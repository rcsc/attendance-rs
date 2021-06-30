// Database magic happens HERE

use crate::tables::*;
use async_graphql::*;
use async_graphql::{Context, Result};
use sqlx::{
    postgres::PgPool,
    types::{
        chrono::{DateTime, Utc},
        Uuid,
    },
};

pub struct Query;
pub struct Mutation;

#[Object]
impl Query {
    // TODO consolidate user stuff into one findUser and attendance stuff into one findAttendance
    async fn users(&self, ctx: &Context<'_>) -> Result<Vec<User>> {
        let pool = ctx.data::<PgPool>()?;

        Ok(sqlx::query_as!(User, "SELECT * FROM users")
            .fetch_all(pool)
            .await?)
    }

    async fn user_by_full_name_search(
        &self,
        ctx: &Context<'_>,
        full_name: String,
    ) -> Result<Vec<User>> {
        let pool = ctx.data::<PgPool>()?;

        Ok(sqlx::query_as!(
            User,
            "SELECT * FROM users where full_name LIKE $1",
            "%".to_string() + &full_name + "%"
        )
        .fetch_all(pool)
        .await?)
    }

    async fn user_by_full_name_match(
        &self,
        ctx: &Context<'_>,
        full_name: String,
    ) -> Result<Vec<User>> {
        let pool = ctx.data::<PgPool>()?;

        Ok(
            sqlx::query_as!(User, "SELECT * FROM users where full_name=$1", &full_name)
                .fetch_all(pool)
                .await?,
        )
    }

    async fn user_by_uuid(&self, ctx: &Context<'_>, uuid: String) -> Result<Option<User>> {
        let pool = ctx.data::<PgPool>()?;
        let uuid = match Uuid::parse_str(&uuid) {
            Ok(uuid) => uuid,
            Err(_) => return Ok(None), // I'll do this since there would be no results for an invalid UUID
        };

        Ok(Some(
            sqlx::query_as!(User, "SELECT * FROM users where uuid=$1", uuid)
                .fetch_one(pool)
                .await?,
        ))
    }

    // This seems really wasteful
    async fn user_by_email(&self, ctx: &Context<'_>, email: String) -> Result<Option<User>> {
        let pool = ctx.data::<PgPool>()?;

        Ok(Some(
            sqlx::query_as!(User, "SELECT * FROM users where email=$1", email)
                .fetch_one(pool)
                .await?,
        ))
    }

    async fn attendance(&self, ctx: &Context<'_>) -> Result<Vec<Attendance>> {
        let pool = ctx.data::<PgPool>()?;
        Ok(sqlx::query_as!(Attendance, "SELECT * FROM attendance")
            .fetch_all(pool)
            .await?)
    }

    async fn attendance_by_date(
        &self,
        ctx: &Context<'_>,
        date: DateTime<Utc>,
    ) -> Result<Vec<Attendance>> {
        let pool = ctx.data::<PgPool>()?;
        Ok(sqlx::query_as!(
            Attendance,
            "SELECT * FROM attendance WHERE in_time >= $1 AND in_time <= $1",
            date
        )
        .fetch_all(pool)
        .await?)
    }
}

#[Object]
impl Mutation {
    async fn log_attendance(
        &self,
        ctx: &Context<'_>,
        uuid: Option<String>,
        email: Option<String>,
    ) -> Result<Attendance> {
        let pool = ctx.data::<PgPool>()?;

        // If both uuid and email are valid, then uuid will be chosen.
        // If only email/uuid, then email/uuid will be chosen

        let uuid_parsed = if let Some(uuid_unwrapped) = uuid {
            Uuid::parse_str(&uuid_unwrapped)?
        } else if let Some(email_unwrapped) = email {
            // Query the server to find the uuid
            sqlx::query!("SELECT uuid FROM users WHERE email=$1", email_unwrapped)
                .fetch_one(pool)
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
        .fetch_optional(pool)
        .await?
        {
            if attendance.out_time.is_none() {
                // Run an update query, as the user checked in, but not out

                attendance.out_time = sqlx::query!(
                    "UPDATE attendance SET out_time=$1 WHERE id=$2 RETURNING out_time",
                    Utc::now(),
                    attendance.id
                )
                .fetch_one(pool)
                .await?
                .out_time;

                return Ok(attendance);
            }
        }

        println!("uuid_parsed was {:?}", uuid_parsed);

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
        .fetch_one(pool)
        .await?;

        attendance.id = record.id;

        Ok(attendance)
    }
}
