use crate::schema::user;
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Queryable)]
pub struct DatabaseUser {
    pub id: Uuid,
    pub name: String,
    pub password: String,
}

#[derive(Insertable)]
#[table_name = "user"]
struct InsertUser<'a> {
    pub name: &'a str,
    pub password: &'a str,
}

impl DatabaseUser {
    pub fn load_by_id(
        conn: &diesel::PgConnection,
        id: Uuid,
    ) -> Result<Option<DatabaseUser>, failure::Error> {
        user::table
            .find(id)
            .get_result(conn)
            .optional()
            .map_err(Into::into)
    }
    pub fn load_by_name(
        conn: &diesel::PgConnection,
        name: &str,
    ) -> Result<Option<DatabaseUser>, failure::Error> {
        user::table
            .filter(user::dsl::name.eq(name))
            .get_result(conn)
            .optional()
            .map_err(Into::into)
    }

    pub fn create(
        conn: &diesel::PgConnection,
        name: &str,
        password: &str,
    ) -> Result<DatabaseUser, failure::Error> {
        diesel::insert_into(user::table)
            .values(InsertUser { name, password })
            .get_result(conn)
            .map_err(Into::into)
    }
}
