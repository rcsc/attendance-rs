// Database magic happens HERE

use crate::tables::*;
use crate::PRIVATE_KEY;
use async_graphql::*;
use async_graphql::{guard::Guard, validators::Email, Context, Result};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use log::debug;
use sqlx::{
    postgres::PgPool,
    types::{
        chrono::{DateTime, Utc},
        Uuid,
    },
};
use std::sync::Arc;

pub struct Query;
pub struct Mutation;

#[Object]
impl Query {
    // TODO consolidate user stuff into one findUser and attendance stuff into one findAttendance
    #[graphql(guard(CapabilityGuard(capability = "TokenCapability::Viewer")))]
    async fn users(&self, ctx: &Context<'_>) -> Result<Vec<User>> {
        let pool = ctx.data::<Arc<PgPool>>()?;

        Ok(sqlx::query_as!(User, "SELECT * FROM users")
            .fetch_all(&**pool)
            .await?)
    }

    #[graphql(guard(CapabilityGuard(capability = "TokenCapability::Viewer")))]
    async fn user_by_full_name_search(
        &self,
        ctx: &Context<'_>,
        full_name: String,
    ) -> Result<Vec<User>> {
        let pool = ctx.data::<Arc<PgPool>>()?;

        Ok(sqlx::query_as!(
            User,
            "SELECT * FROM users where full_name LIKE $1",
            "%".to_string() + &full_name + "%"
        )
        .fetch_all(&**pool)
        .await?)
    }

    #[graphql(guard(CapabilityGuard(capability = "TokenCapability::Viewer")))]
    async fn user_by_full_name_match(
        &self,
        ctx: &Context<'_>,
        full_name: String,
    ) -> Result<Vec<User>> {
        let pool = ctx.data::<Arc<PgPool>>()?;

        Ok(
            sqlx::query_as!(User, "SELECT * FROM users where full_name=$1", &full_name)
                .fetch_all(&**pool)
                .await?,
        )
    }

    #[graphql(guard(or(
        CapabilityGuard(capability = "TokenCapability::Collector"),
        CapabilityGuard(capability = "TokenCapability::Viewer")
    )))]
    async fn user_by_uuid(&self, ctx: &Context<'_>, uuid: String) -> Result<Option<User>> {
        let pool = ctx.data::<Arc<PgPool>>()?;
        let uuid = match Uuid::parse_str(&uuid) {
            Ok(uuid) => uuid,
            Err(_) => return Ok(None), // I'll do this since there would be no results for an invalid UUID
        };

        Ok(Some(
            sqlx::query_as!(User, "SELECT * FROM users where uuid=$1", uuid)
                .fetch_one(&**pool)
                .await?,
        ))
    }

    // This seems really wasteful (less code by consolidating each "user_by" into one thing?)
    #[graphql(guard(or(
        CapabilityGuard(capability = "TokenCapability::Collector"),
        CapabilityGuard(capability = "TokenCapability::Viewer")
    )))]
    async fn user_by_email(&self, ctx: &Context<'_>, email: String) -> Result<Option<User>> {
        let pool = ctx.data::<Arc<PgPool>>()?;

        Ok(Some(
            sqlx::query_as!(User, "SELECT * FROM users where email=$1", email)
                .fetch_one(&**pool)
                .await?,
        ))
    }

    #[graphql(guard(CapabilityGuard(capability = "TokenCapability::Viewer")))]
    async fn attendance(&self, ctx: &Context<'_>) -> Result<Vec<Attendance>> {
        let pool = ctx.data::<Arc<PgPool>>()?;
        Ok(sqlx::query_as!(Attendance, "SELECT * FROM attendance")
            .fetch_all(&**pool)
            .await?)
    }

    #[graphql(guard(CapabilityGuard(capability = "TokenCapability::Viewer")))]
    async fn attendance_by_date(
        &self,
        ctx: &Context<'_>,
        date: DateTime<Utc>,
    ) -> Result<Vec<Attendance>> {
        let pool = ctx.data::<Arc<PgPool>>()?;
        Ok(sqlx::query_as!(
            Attendance,
            "SELECT * FROM attendance WHERE in_time >= $1 AND in_time <= $1",
            date
        )
        .fetch_all(&**pool)
        .await?)
    }
}

#[Object]
impl Mutation {
    #[graphql(guard(CapabilityGuard(capability = "TokenCapability::Collector")))]
    async fn create_user(
        &self,
        ctx: &Context<'_>,
        full_name: String,
        #[graphql(validator(Email))] email: String,
        #[graphql(validator(PhoneNumber))] phone_number: Option<String>,
    ) -> Result<User> {
        let pool = ctx.data::<Arc<PgPool>>()?;
        let mut new_user = User {
            full_name,
            email,
            phone_number,
            uuid: Uuid::nil(),
            create_time: Utc::now(),
            update_time: None,
        };

        // formatter won't format this for some reason
        new_user.uuid = sqlx::query!(
            "INSERT INTO users (full_name, email, phone_number, create_time) VALUES ($1, $2, $3, $4) RETURNING uuid",
            new_user.full_name, new_user.email, new_user.phone_number, new_user.create_time)
            .fetch_one(&**pool)
            .await?
            .uuid;

        Ok(new_user)
    }

    #[graphql(guard(CapabilityGuard(capability = "TokenCapability::Collector")))]
    async fn update_user(
        &self,
        ctx: &Context<'_>,
        uuid: String,
        full_name: Option<String>,
        #[graphql(validator(Email))] email: Option<String>,
        #[graphql(validator(PhoneNumber))] phone_number: Option<String>,
    ) -> Result<User> {
        let pool = ctx.data::<Arc<PgPool>>()?;

        let mut user = match sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE uuid=$1",
            Uuid::parse_str(&uuid)?
        )
        .fetch_optional(&**pool)
        .await?
        {
            Some(user) => user,
            None => return Err(async_graphql::Error::new("User to modify not found!")),
        };

        if let Some(full_name) = full_name {
            user.full_name = full_name;
        }
        if let Some(email) = email {
            user.email = email;
        }
        if let Some(phone_number_data) = phone_number {
            user.phone_number = Some(phone_number_data);
        }

        sqlx::query!(
            "UPDATE users SET (full_name, email, phone_number) = ($1, $2, $3) WHERE uuid=$4",
            user.full_name,
            user.email,
            user.phone_number,
            user.uuid
        )
        .execute(&**pool)
        .await?;

        Ok(user)
    }

    #[graphql(guard(CapabilityGuard(capability = "TokenCapability::Collector")))]
    async fn log_attendance(
        &self,
        ctx: &Context<'_>,
        uuid: Option<String>,
        email: Option<String>,
    ) -> Result<Attendance> {
        let pool = ctx.data::<Arc<PgPool>>()?;

        // If both uuid and email are valid, then uuid will be chosen.
        // If only email/uuid, then email/uuid will be chosen

        let uuid_parsed = if let Some(uuid_unwrapped) = uuid {
            Uuid::parse_str(&uuid_unwrapped)?
        } else if let Some(email_unwrapped) = email {
            // Query the server to find the uuid
            sqlx::query!("SELECT uuid FROM users WHERE email=$1", email_unwrapped)
                .fetch_one(&**pool)
                .await?
                .uuid
        } else {
            // I'm pretty sure this code will never get executed
            return Err(async_graphql::Error::new(
                "You must specify either a UUID or a user's e-mail",
            ));
        };

        // Check if the user has an entry without an out time

        if let Some(mut attendance) = sqlx::query_as!(
            Attendance,
            "SELECT * FROM attendance WHERE user_uuid=$1 ORDER BY in_time DESC LIMIT 1",
            uuid_parsed
        )
        .fetch_optional(&**pool)
        .await?
        {
            if attendance.out_time.is_none() {
                // Run an update query, as the user checked in, but not out

                attendance.out_time = sqlx::query!(
                    "UPDATE attendance SET out_time=$1 WHERE id=$2 RETURNING out_time",
                    Utc::now(),
                    attendance.id
                )
                .fetch_one(&**pool)
                .await?
                .out_time;

                return Ok(attendance);
            }
        }

        debug!("uuid_parsed was {:?}", uuid_parsed);

        let mut attendance = Attendance {
            id: -1,
            user_uuid: uuid_parsed,
            in_time: Utc::now(),
            out_time: None,
        };
        let record = sqlx::query!(
            "INSERT INTO attendance (user_uuid, in_time) VALUES ($1, $2) RETURNING id",
            attendance.user_uuid,
            attendance.in_time,
        )
        .fetch_one(&**pool)
        .await?;

        attendance.id = record.id;

        Ok(attendance)
    }

    // Only administrators
    #[graphql(guard(or(
        CapabilityGuard(capability = "TokenCapability::Administrator"),
        FirstRunGuard()
    )))]
    async fn generate_token(
        &self,
        ctx: &Context<'_>,
        description: String,
        capability: TokenCapability,
        initial_valid_time: Option<DateTime<Utc>>,
        expiration_time: DateTime<Utc>,
    ) -> Result<String> {
        // Generate a JWT
        let pool = ctx.data::<Arc<PgPool>>()?;
        let mut token_struct = Token {
            description,
            capability,
            initial_valid_time,
            expiration_time,
            uuid: Uuid::nil(),
            create_time: Utc::now(),
        };

        token_struct.uuid = sqlx::query!(
            "INSERT INTO tokens (description, expiration_time, create_time, capability) VALUES ($1, $2, $3, $4) RETURNING uuid",
            token_struct.description, token_struct.expiration_time, token_struct.create_time, token_struct.capability as TokenCapability
        ).fetch_one(&**pool).await?.uuid;

        let claims = JWTClaims {
            uuid: token_struct.uuid.to_string(),
            cap: token_struct.capability,
            exp: token_struct.expiration_time.timestamp(),
            nbf: token_struct.initial_valid_time.map(|item| item.timestamp()),
        };

        let private_key_read = PRIVATE_KEY.read().unwrap();
        let private_key_as_bytes = private_key_read.as_ref();
        match jsonwebtoken::encode(
            &Header::new(Algorithm::ES256),
            &claims,
            &EncodingKey::from_ec_pem(private_key_as_bytes).expect("Expected a valid private key"),
        ) {
            Ok(key) => Ok(key),
            Err(error) => Err(async_graphql::Error::new(format!("{}", error))),
        }
    }
}
