use async_graphql::*;
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
