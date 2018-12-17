use crate::schema::user_token;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Queryable)]
pub struct UserToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub ip: String,
    pub last_used: DateTime<Utc>,
    pub active: bool,
}

#[derive(Insertable)]
#[table_name = "user_token"]
pub struct InsertToken<'a> {
    pub user_id: Uuid,
    pub ip: &'a str,
    pub last_used: DateTime<Utc>,
    pub active: bool,
}

impl UserToken {
    pub fn load_by_user_and_token_id(
        conn: &diesel::PgConnection,
        user_id: Uuid,
        token_id: Uuid,
    ) -> Result<Option<UserToken>, failure::Error> {
        user_token::table
            .filter(
                user_token::dsl::user_id
                    .eq(user_id)
                    .and(user_token::dsl::id.eq(token_id)),
            )
            .get_result(conn)
            .optional()
            .map_err(Into::into)
    }

    pub fn update_last_used(&mut self, conn: &diesel::PgConnection) -> Result<(), failure::Error> {
        let result: DateTime<Utc> =
            diesel::update(user_token::table.filter(user_token::dsl::id.eq(self.id)))
                .set(user_token::dsl::last_used.eq(Utc::now()))
                .returning(user_token::dsl::last_used)
                .get_result(conn)?;
        self.last_used = result;
        Ok(())
    }

    pub fn create(
        conn: &diesel::PgConnection,
        user_id: Uuid,
        ip: &str,
    ) -> Result<UserToken, failure::Error> {
        diesel::insert_into(user_token::table)
            .values(InsertToken {
                user_id,
                ip,
                last_used: Utc::now(),
                active: true,
            })
            .get_result(conn)
            .map_err(Into::into)
    }
}
