use async_graphql::{
    http::{playground_source, GraphQLPlaygroundConfig},
    EmptyMutation, EmptySubscription, Schema,
};
use sqlx::postgres::PgPoolOptions;
use std::env;
use tide::{http::mime, Body, Response, StatusCode};
mod graphql_schema;
mod tables;

#[async_std::main]
async fn main() -> Result<(), sqlx::Error> {
    let pg_connection_str = match env::var("AR_PG_CONNECTION_STR") {
        Ok(data) => data,
        Err(_) => {
            println!("You must provide a valid PostgreSQL connection string!");
            std::process::exit(1)
        }
    };

    let http_host_str = env::var("AR_PG_HTTP_HOST_STR").unwrap_or("127.0.0.1:8080".to_string());

    let pool = match PgPoolOptions::new()
        .max_connections(5)
        .connect(&pg_connection_str)
        .await
    {
        Ok(conn) => conn,
        Err(err) => {
            println!(
                "An error occurred with opening the PostgreSQL connection.
Please check your connection string and try again."
            );
            println!(
                "\nError details (check the SQLx library for details): {:#?}",
                err
            );
            std::process::exit(1)
        }
    };

    // TODO migrations

    let schema = Schema::build(graphql_schema::Query, EmptyMutation, EmptySubscription)
        .data(pool)
        .finish();

    let mut app = tide::new();
    app.at("/graphql")
        .post(async_graphql_tide::endpoint(schema));
    app.at("/").get(|_| async move {
        let mut resp = Response::new(StatusCode::Ok);
        resp.set_body(Body::from_string(playground_source(
            GraphQLPlaygroundConfig::new("/graphql"),
        )));
        resp.set_content_type(mime::HTML);
        Ok(resp)
    });

    println!("GraphQL API is listening at {}", http_host_str);
    app.listen(http_host_str).await?;

    Ok(())
}
