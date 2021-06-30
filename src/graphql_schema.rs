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
#[Object]
impl Query {
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
