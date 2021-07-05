use async_graphql::guard::Guard;
use async_graphql::*;
use serde::{Deserialize, Serialize};
use sqlx::{
    postgres::PgPool,
    types::{
        chrono::{DateTime, Utc},
        Uuid,
    },
};

#[derive(Debug, SimpleObject)]
#[graphql(complex)]
pub struct User {
    #[graphql(skip)]
    pub uuid: Uuid, // Won't let me use option, so I guess we call Uuid::null() at this point to create the struct
    pub full_name: String,
    pub email: String,
    pub phone_number: Option<String>,
    pub create_time: DateTime<Utc>,
    pub update_time: Option<DateTime<Utc>>,
}

#[ComplexObject]
impl User {
    async fn uuid(&self) -> String {
        let hyphenated = self.uuid.to_hyphenated().clone();
        hyphenated.to_string()
    }
    async fn attendance(&self, ctx: &Context<'_>) -> Result<Vec<Attendance>> {
        let pool = ctx.data::<PgPool>()?;
        Ok(sqlx::query_as!(
            Attendance,
            "SELECT * FROM attendance WHERE user_uuid=$1",
            self.uuid
        )
        .fetch_all(pool)
        .await?)
    }
}

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct Attendance {
    pub id: i32,
    #[graphql(skip)]
    pub user_uuid: Uuid,
    pub in_time: DateTime<Utc>,
    pub out_time: Option<DateTime<Utc>>,
}
#[ComplexObject]
impl Attendance {
    async fn user_uuid(&self) -> String {
        let hyphenated = self.user_uuid.to_hyphenated().clone();
        hyphenated.to_string()
    }
}

#[derive(sqlx::Type, Enum, Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Debug)]
#[sqlx(type_name = "token_capability", rename_all = "lowercase")]
pub enum TokenCapability {
    Collector,
    Viewer,
    Administrator,
}

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct Token {
    #[graphql(skip)]
    pub uuid: Uuid,
    pub description: String,
    pub initial_valid_time: Option<DateTime<Utc>>,
    pub expiration_time: DateTime<Utc>,
    pub create_time: DateTime<Utc>,
    pub capability: TokenCapability,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JWTClaims {
    pub uuid: String,
    // Capability (shorthand)
    pub cap: TokenCapability,
    // For validation with the JWT library
    pub exp: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nbf: Option<i64>,
    // We will want to validate these pieces of data in the JWT **and** in the database
}

pub struct CapabilityGuard {
    pub capability: TokenCapability,
}

// Stolen from https://async-graphql.github.io/async-graphql/en/field_guard.html
#[async_trait::async_trait]
impl Guard for CapabilityGuard {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        // Administrators get access to everything, since they are administrators
        if ctx.data_opt::<TokenCapability>() == Some(&self.capability)
            || ctx.data_opt::<TokenCapability>() == Some(&TokenCapability::Administrator)
        {
            Ok(())
        } else {
            Err("You are not allowed to access this resource.".into())
        }
    }
}

// TODO custom async_graphql implementation for uuid
#[ComplexObject]
impl Token {}
