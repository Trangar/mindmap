use crate::schema::{note, note_history};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Queryable, QueryableByName)]
#[table_name = "note"]
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

#[derive(Queryable)]
pub struct NoteHistory {
    pub id: Uuid,
    pub note_id: Uuid,
    pub created: DateTime<Utc>,
    pub title: String,
    pub body: String,
}

impl NoteHistory {
    pub fn delete_by_note(
        conn: &diesel::PgConnection,
        note_id: Uuid,
    ) -> Result<(), failure::Error> {
        diesel::delete(note_history::table.filter(note_history::dsl::note_id.eq(note_id)))
            .execute(conn)?;
        Ok(())
    }
}

#[derive(Insertable)]
#[table_name = "note_history"]
pub struct InsertNoteHistory<'a> {
    pub note_id: Uuid,
    pub created: DateTime<Utc>,
    pub title: &'a str,
    pub body: &'a str,
}

impl<'a> InsertNoteHistory<'a> {
    fn create(conn: &diesel::PgConnection, note: &Note) -> Result<(), failure::Error> {
        diesel::insert_into(note_history::table)
            .values(InsertNoteHistory {
                note_id: note.id,
                created: Utc::now(),
                title: note.title.as_str(),
                body: note.body.as_str(),
            })
            .execute(conn)?;
        Ok(())
    }
}

impl Note {
    pub fn load_paged(
        conn: &diesel::PgConnection,
        user_id: Uuid,
        start: i64,
        count: i64,
    ) -> Result<Vec<Note>, failure::Error> {
        note::table
            .filter(
                note::dsl::user_id
                    .eq(user_id)
                    .and(note::dsl::deleted.eq(false)),
            )
            .order(note::dsl::view_count.desc())
            .offset(start)
            .limit(count)
            .get_results(conn)
            .map_err(Into::into)
    }

    pub fn count_by_user(
        conn: &diesel::PgConnection,
        user_id: Uuid,
    ) -> Result<i64, failure::Error> {
        note::table
            .filter(note::dsl::user_id.eq(user_id))
            .count()
            .get_result(conn)
            .map_err(Into::into)
    }

    pub fn delete(conn: &diesel::PgConnection, id: Uuid) -> Result<(), failure::Error> {
        diesel::delete(note::table.find(id)).execute(conn)?;
        Ok(())
    }

    pub fn load_by_id(
        conn: &diesel::PgConnection,
        id: Uuid,
    ) -> Result<Option<Note>, failure::Error> {
        note::table
            .find(id)
            .get_result(conn)
            .optional()
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

    pub fn search(
        conn: &diesel::PgConnection,
        search_query: crate::SearchQuery,
        user_id: Uuid,
    ) -> Result<Vec<Note>, failure::Error> {
        fn sanitize(s: String) -> String {
            let mut result = String::with_capacity(s.len());
            for c in s.chars() {
                if c.is_alphanumeric() {
                    result.push(c);
                }
            }
            result
        }
        let search_words =
            search_query
                .queries
                .into_iter()
                .map(sanitize)
                .fold(String::new(), |acc, s| {
                    if s.trim().is_empty() {
                        acc
                    } else if acc.is_empty() {
                        s
                    } else {
                        format!("{} & {}", acc, s)
                    }
                });
        let mut query = String::from(r#"
SELECT
    note_search.id,
    note_search.user_id,
    note_search.view_count,
    note_search.seo_name,
    note_search.title,
    note_search.body,
    note_search.deleted
FROM (
    SELECT
        note.id,
        note.user_id,
        note.view_count,
        note.seo_name,
        note.title,
        note.body,
        note.deleted,
        Setweight(To_tsvector('english', note.title), 'A') || Setweight(To_tsvector('english', note.body), 'B') AS document
    FROM note
    WHERE note.user_id = $1
) note_search
WHERE note_search.document @@ to_tsquery('english', $2)"#);

        for exclude in search_query.excludes {
            let exclude = sanitize(exclude);
            if exclude.trim().is_empty() {
                continue;
            }
            query += "\nAND note_search.title NOT LIKE '%";
            query += exclude.as_str();
            query += "%' AND note_search.body NOT LIKE '%";
            query += exclude.as_str();
            query += "%'";
        }
        query += r#"
ORDER BY ts_rank(note_search.document, to_tsquery('english', $2)) DESC"#;

        diesel::sql_query(query)
            .bind::<diesel::sql_types::Uuid, _>(user_id)
            .bind::<diesel::sql_types::Text, _>(&search_words)
            .load(conn)
            .map_err(Into::into)
    }

    pub fn load_history(
        conn: &diesel::PgConnection,
        note_id: Uuid,
    ) -> Result<Vec<NoteHistory>, failure::Error> {
        note_history::table
            .filter(note_history::dsl::note_id.eq(note_id))
            .order(note_history::dsl::created.desc())
            .get_results(conn)
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

    pub fn update(
        conn: &diesel::PgConnection,
        id: Uuid,
        seo_name: &str,
        title: &str,
        body: &str,
    ) -> Result<Note, failure::Error> {
        let note: Note = diesel::update(note::table.find(id))
            .set((
                note::dsl::seo_name.eq(seo_name),
                note::dsl::title.eq(title),
                note::dsl::body.eq(body),
            ))
            .get_result(conn)?;
        InsertNoteHistory::create(conn, &note)?;
        Ok(note)
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
        let note: Note = diesel::insert_into(note::table)
            .values(note)
            .get_result(conn)?;

        InsertNoteHistory::create(conn, &note)?;

        Ok(note)
    }
}
