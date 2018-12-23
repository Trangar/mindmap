use super::note::Note;
use crate::schema::{note, note_link};
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Queryable)]
pub struct NoteLink {
    pub id: Uuid,
    pub other: Note,
    pub click_count: i32,
}

#[derive(Insertable)]
#[table_name = "note_link"]
pub struct InsertNoteLink {
    left: Uuid,
    right: Uuid,
    click_count: i32,
}

impl NoteLink {
    pub fn load_by_note(
        conn: &diesel::PgConnection,
        note_id: Uuid,
    ) -> Result<Vec<NoteLink>, failure::Error> {
        let mut first: Vec<NoteLink> = note_link::table
            .filter(note_link::dsl::left.eq(note_id))
            .inner_join(note::table.on(note::dsl::id.eq(note_link::dsl::right)))
            .select((
                note_link::dsl::id,
                (
                    note::dsl::id,
                    note::dsl::user_id,
                    note::dsl::view_count,
                    note::dsl::seo_name,
                    note::dsl::title,
                    note::dsl::body,
                    note::dsl::deleted,
                ),
                note_link::dsl::click_count,
            ))
            .get_results(conn)?;
        let second: Vec<NoteLink> = note_link::table
            .filter(note_link::dsl::right.eq(note_id))
            .inner_join(note::table.on(note::dsl::id.eq(note_link::dsl::left)))
            .select((
                note_link::dsl::id,
                (
                    note::dsl::id,
                    note::dsl::user_id,
                    note::dsl::view_count,
                    note::dsl::seo_name,
                    note::dsl::title,
                    note::dsl::body,
                    note::dsl::deleted,
                ),
                note_link::dsl::click_count,
            ))
            .get_results(conn)?;

        first.extend(second.into_iter());
        first.sort_by_key(|f| -f.click_count);
        Ok(first)
    }

    pub fn delete_by_note(
        conn: &diesel::PgConnection,
        note_id: Uuid,
    ) -> Result<(), failure::Error> {
        diesel::delete(
            note_link::table.filter(
                note_link::dsl::left
                    .eq(note_id)
                    .or(note_link::dsl::right.eq(note_id)),
            ),
        )
        .execute(conn)?;
        Ok(())
    }

    pub fn create(
        conn: &diesel::PgConnection,
        left: Uuid,
        right: Uuid,
    ) -> Result<(), failure::Error> {
        diesel::insert_into(note_link::table)
            .values(InsertNoteLink {
                left,
                right,
                click_count: 0,
            })
            .execute(conn)?;
        Ok(())
    }

    pub fn increase_click_count(
        conn: &diesel::PgConnection,
        id: Uuid,
    ) -> Result<(), failure::Error> {
        diesel::update(note_link::table.find(id))
            .set(note_link::dsl::click_count.eq(note_link::dsl::click_count + 1))
            .execute(conn)?;
        Ok(())
    }
}
