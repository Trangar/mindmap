use crate::schema::note;
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Queryable)]
pub struct Note {
    pub id: Uuid,
    pub user_id: Uuid,
    pub view_count: i32,
    pub seo_name: String,
    pub title: String,
    pub body: String,
    pub deleted: bool,
}

#[derive(Insertable)]
#[table_name = "note"]
pub struct InsertNote<'a> {
    pub user_id: Uuid,
    pub view_count: i32,
    pub seo_name: &'a str,
    pub title: &'a str,
    pub body: &'a str,
    pub deleted: bool,
}

impl Note {
    pub fn load_top_10(
        conn: &diesel::PgConnection,
        user_id: Uuid,
    ) -> Result<Vec<Note>, failure::Error> {
        note::table
            .filter(
                note::dsl::user_id
                    .eq(user_id)
                    .and(note::dsl::deleted.eq(false)),
            )
            .order(note::dsl::view_count.desc())
            .limit(10)
            .get_results(conn)
            .map_err(Into::into)
    }

    pub fn load_by_seo_name(
        conn: &diesel::PgConnection,
        name: &str,
        user_id: Uuid,
    ) -> Result<Option<Note>, failure::Error> {
        note::table
            .filter(
                note::dsl::user_id
                    .eq(user_id)
                    .and(note::dsl::seo_name.eq(name)),
            )
            .get_result(conn)
            .optional()
            .map_err(Into::into)
    }

    pub fn increase_view_count(
        conn: &diesel::PgConnection,
        id: Uuid,
    ) -> Result<i32, failure::Error> {
        diesel::update(note::table.find(id))
            .set(note::dsl::view_count.eq(note::dsl::view_count + 1))
            .returning(note::dsl::view_count)
            .get_result(conn)
            .map_err(Into::into)
    }

    pub fn create(
        conn: &diesel::PgConnection,
        seo_name: &str,
        title: &str,
        body: &str,
        user_id: Uuid,
    ) -> Result<Note, failure::Error> {
        let note = InsertNote {
            user_id,
            view_count: 0,
            seo_name,
            title,
            body,
            deleted: false,
        };
        diesel::insert_into(note::table)
            .values(note)
            .get_result(conn)
            .map_err(Into::into)
    }
}
