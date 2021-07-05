use actix_web::{guard, middleware::Logger, web, App, HttpRequest, HttpResponse, HttpServer};
use async_graphql::{
    http::{playground_source, GraphQLPlaygroundConfig},
    EmptySubscription, Schema, ServerError,
};
use async_graphql_actix_web::{Request, Response};
use dotenv;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use log::{debug, error, info, warn};
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::sync::{Arc, RwLock};

use lazy_static::lazy_static;

mod graphql_schema;
mod tables;

lazy_static! {
    // Unfortunately, there is no better solution than this (other than option, which will just be a pain later)
    static ref PRIVATE_KEY: RwLock<String> = RwLock::new("".to_string());
    static ref PUBLIC_KEY: RwLock<String> = RwLock::new("".to_string());
    static ref FIRST_RUN: RwLock<bool> = RwLock::new(false);
}

fn read_keyfile_from_envvar(var: &str) -> String {
    match dotenv::var(var) {
        Ok(var_val) => match std::fs::read_to_string(var_val) {
            Ok(keyfile_data) => keyfile_data,
            Err(e) => error_exit(&format!(
                "Failed to read file from env var {}. Error: {}",
                var, e
            )),
        },
        Err(_) => {
            error_exit(&format!("Expected a valid {} variable", var));
        }
    }
}
// Stolen from https://github.com/async-graphql/examples/blob/master/actix-web/token-from-header/src/main.rs
async fn graphql_playground() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(playground_source(
            GraphQLPlaygroundConfig::new("/").subscription_endpoint("/"),
        ))
}

async fn graphql_request(
    pool: web::Data<Arc<PgPool>>,
    schema: web::Data<Schema<graphql_schema::Query, graphql_schema::Mutation, EmptySubscription>>,
    request: HttpRequest,
    graphql_request: Request,
) -> Response {
    let err_msg_response = |msg: &str| -> Response {
        async_graphql::Response::from_errors(vec![ServerError::new(msg, None)]).into()
    };

    // Authentication bypassed if this is the first run. We will have to do something to get past async_graphql's guards when we implement them
    if *FIRST_RUN.read().unwrap() {
        // *FIRST_RUN.write().unwrap() = false;

        let graphql_request = graphql_request.into_inner();
        // Nope (TODO only disable first run mode AFTER they issue a token, and not after the first request? I'm undecided.)
        let graphql_response = schema.execute(graphql_request).await.into();

        // Check if we should continue first-run mode
        match check_first_run(&*pool).await {
            Ok(continue_first_run) if !continue_first_run => {
                info!("NOTICE: Disabling first-run mode. A token has been generated.");
                *FIRST_RUN.write().unwrap() = false;
            }
            Err(e) => {
                warn!("WARNING: First-run check returned error {}", e);
            }
            _ => {} // Continue first-run mode
        }

        return graphql_response;
    }

    if let Some(token) = request.headers().get("Token") {
        if let Ok(token_str) = token.to_str() {
            // Fetch token from SQL and check if it's valid
            // TODO potential inefficiency here since it has to do this every time
            let public_key_read = PUBLIC_KEY.read().unwrap();
            let public_key_as_bytes = public_key_read.as_ref();
            let decoding_key = match DecodingKey::from_ec_pem(public_key_as_bytes) {
                Ok(decoding_key) => decoding_key,
                Err(e) => return err_msg_response(&format!("Expected a valid public key. Please check your server configuration. Error: {}", e)),
            };

            match decode::<tables::JWTClaims>(
                token_str,
                &decoding_key,
                &Validation::new(Algorithm::ES256),
            ) {
                Ok(claim_data) => {
                    debug!("{:#?} details", claim_data);

                    // TODO x-verify the claim data with the database.
                    // If the data doesn't match with the data in the db,
                    // **notify everything and everyone immediately, since someone stole the signing key**
                    // (on second thought, you're not supposed to do this since this defeats the point of a JWT)

                    let graphql_request = graphql_request.into_inner().data(claim_data.claims.cap);
                    return schema.execute(graphql_request).await.into();
                }
                Err(e) => {
                    return err_msg_response(&format!("There was an error with your token: {}", e))
                }
            }
        }
    }

    err_msg_response("A valid token is missing. Please provide one in the HTTP header.")
}

async fn check_first_run(pool: &PgPool) -> Result<bool, String> {
    // Checks if first-run mode should be enabled or disabled
    let number_of_tokens = match sqlx::query!("SELECT COUNT(*) FROM tokens")
        .fetch_one(pool)
        .await
    {
        Ok(count_result) => match count_result.count {
            Some(count_result_unwrapped) => count_result_unwrapped,
            None => {
                return Err("Could not check whether or not to run in first-run mode. Exiting (also this should never happen).
                            SELECT COUNT(*) returned None. Maybe migrate the database?".to_string())
            }
        },
        Err(e) => {
            return Err(format!("Could not check whether or not to run in first-run mode. Exiting. Error details: {:#?}",e))
        }
    };
    debug!("Checking number of tokens: {}", number_of_tokens);

    Ok(if number_of_tokens == 0 { true } else { false })
}

fn error_exit(error_message: &str) -> ! {
    error!("{}", error_message);
    std::process::exit(1);
}

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    env_logger::init();
    // We use the env var that SQLx uses to make our lives easier
    let pg_connection_str = match dotenv::var("DATABASE_URL") {
        Ok(data) => data,
        Err(_) => error_exit("You must provide a valid PostgreSQL connection string!"),
    };

    let http_host_str = dotenv::var("AR_PG_HTTP_HOST_STR").unwrap_or("127.0.0.1:8080".to_string());

    let pool = Arc::new(match PgPoolOptions::new()
        .max_connections(5)
        .connect(&pg_connection_str)
        .await
    {
        Ok(conn) => conn,
        Err(err) => {
                error_exit(&format!(
                "An error occurred with opening the PostgreSQL connection.
Please check your connection string and try again.\nError details (check the SQLx library for details): {:#?}", err
            ))
        }
    });

    // If there are zero tokens in the database, we will remove authentication so someone can create a token (and then immediately turn off "first run" mode)
    *FIRST_RUN.write().unwrap() = match check_first_run(&*pool).await {
        Ok(should_first_run) => should_first_run,
        Err(e) => error_exit(&e),
    };
    info!("First run status is {}\n", FIRST_RUN.read().unwrap());

    // Load public and private keys
    // TODO maybe read as bytes instead of strings since that's what the library wants? It would be more efficient.
    *PRIVATE_KEY.write().unwrap() = read_keyfile_from_envvar("AR_PG_PRIVATE_KEY");
    *PUBLIC_KEY.write().unwrap() = read_keyfile_from_envvar("AR_PG_PUBLIC_KEY");

    let schema = Schema::build(
        graphql_schema::Query,
        graphql_schema::Mutation,
        EmptySubscription,
    )
    .data(Arc::clone(&pool))
    .finish();

    info!("GraphQL API is listening at {}", http_host_str);

    HttpServer::new(move || {
        App::new()
            .data(Arc::clone(&pool))
            .data(schema.clone())
            .wrap(Logger::default())
            .service(
                web::resource("/graphql")
                    .guard(guard::Post())
                    .to(graphql_request),
            )
            .service(
                web::resource("/")
                    .guard(guard::Get())
                    .to(graphql_playground),
            )
    })
    .bind(http_host_str)?
    .run()
    .await
}
