use crate::models::note::{Note as DatabaseNote, NoteHistory as DatabaseNoteHistory};
use crate::models::note_link::NoteLink as DatabaseNoteLink;
use crate::routes::SearchQuery;
use chrono::{DateTime, Utc};
use failure::format_err;
use slug::slugify;
use uuid::Uuid;

#[derive(Serialize)]
pub struct Note {
    pub id: Uuid,
    pub user_id: Uuid,
    pub seo_name: String,
    pub title: String,
    pub body: String,
}

impl From<DatabaseNote> for Note {
    fn from(n: DatabaseNote) -> Note {
        Note {
            id: n.id,
            user_id: n.user_id,
            seo_name: n.seo_name,
            title: n.title,
            body: n.body,
        }
    }
}

#[derive(Serialize)]
pub struct NoteHistory {
    pub created: DateTime<Utc>,
    pub title: String,
    pub body: String,
}

impl From<DatabaseNoteHistory> for NoteHistory {
    fn from(n: DatabaseNoteHistory) -> NoteHistory {
        NoteHistory {
            created: n.created,
            title: n.title,
            body: n.body,
        }
    }
}

#[derive(Serialize)]
pub struct NoteLink {
    pub note: Note,
    pub link: Link,
}

impl From<DatabaseNoteLink> for NoteLink {
    fn from(l: DatabaseNoteLink) -> NoteLink {
        NoteLink {
            note: l.other.into(),
            link: Link { id: l.id },
        }
    }
}

#[derive(Serialize)]
pub struct Link {
    pub id: Uuid,
}

impl Link {
    pub fn increase_click_count(&self, conn: &diesel::PgConnection) -> Result<(), failure::Error> {
        DatabaseNoteLink::increase_click_count(conn, self.id)
    }
}

impl Note {
    pub fn load_paged(
        conn: &diesel::PgConnection,
        user_id: Uuid,
        start_index: u64,
        count: u64,
    ) -> Result<Vec<Note>, failure::Error> {
        Ok(
            DatabaseNote::load_paged(conn, user_id, start_index as i64, count as i64)?
                .into_iter()
                .map(Into::into)
                .collect(),
        )
    }

    pub fn count_all(conn: &diesel::PgConnection, user_id: Uuid) -> Result<u64, failure::Error> {
        DatabaseNote::count_by_user(conn, user_id).map(|count| count as u64)
    }

    pub fn search(
        conn: &diesel::PgConnection,
        search: SearchQuery,
        user_id: Uuid,
    ) -> Result<Vec<Note>, failure::Error> {
        Ok(DatabaseNote::search(conn, search, user_id)?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    pub fn delete_by_seo_name(
        conn: &diesel::PgConnection,
        name: &str,
        user_id: Uuid,
    ) -> Result<(), failure::Error> {
        let note = Note::load_by_seo_name(conn, name, user_id)?
            .ok_or_else(|| format_err!("Note not found"))?;
        DatabaseNoteLink::delete_by_note(conn, note.id)?;
        DatabaseNoteHistory::delete_by_note(conn, note.id)?;
        DatabaseNote::delete(conn, note.id)?;
        Ok(())
    }

    pub fn load_by_id(
        conn: &diesel::PgConnection,
        id: Uuid,
    ) -> Result<Option<Note>, failure::Error> {
        DatabaseNote::load_by_id(conn, id).map(|o| o.map(Into::into))
    }

    pub fn load_by_seo_name(
        conn: &diesel::PgConnection,
        name: &str,
        user_id: Uuid,
    ) -> Result<Option<Note>, failure::Error> {
        DatabaseNote::load_by_seo_name(conn, name, user_id).map(|o| o.map(Into::into))
    }

    pub fn create(
        conn: &diesel::PgConnection,
        title: &str,
        body: &str,
        user_id: Uuid,
    ) -> Result<Note, failure::Error> {
        let mut seo_name_base = slugify(title);
        if seo_name_base.is_empty() {
            seo_name_base = "-".to_owned();
        }
        let mut seo_name = seo_name_base.clone();
        let mut counter = 1;
        while let Some(_) = DatabaseNote::load_by_seo_name(conn, &seo_name, user_id)? {
            seo_name = format!("{}_{}", seo_name_base, counter);
            counter += 1;
        }

        Ok(DatabaseNote::create(conn, &seo_name, title, body, user_id)?.into())
    }

    pub fn update(
        &mut self,
        conn: &diesel::PgConnection,
        new_title: &str,
        new_body: &str,
    ) -> Result<(), failure::Error> {
        let mut seo_name_base = slugify(new_title);
        if seo_name_base.is_empty() {
            seo_name_base = "-".to_owned();
        }
        let mut seo_name = seo_name_base.clone();
        let mut counter = 1;
        loop {
            if seo_name == self.seo_name {
                break;
            }
            if DatabaseNote::load_by_seo_name(conn, &seo_name, self.user_id)?.is_none() {
                break;
            }
            seo_name = format!("{}_{}", seo_name_base, counter);
            counter += 1;
        }

        let result = DatabaseNote::update(conn, self.id, &seo_name, new_title, new_body)?;
        *self = result.into();
        Ok(())
    }

    pub fn load_history(
        &self,
        conn: &diesel::PgConnection,
    ) -> Result<Vec<NoteHistory>, failure::Error> {
        Ok(DatabaseNote::load_history(conn, self.id)?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    pub fn create_link_to(
        &self,
        conn: &diesel::PgConnection,
        other: &Note,
    ) -> Result<(), failure::Error> {
        DatabaseNoteLink::create(conn, self.id, other.id)?;
        Ok(())
    }

    pub fn increase_view_count(
        &mut self,
        conn: &diesel::PgConnection,
    ) -> Result<(), failure::Error> {
        DatabaseNote::increase_view_count(conn, self.id)?;
        Ok(())
    }

    pub fn load_links(&self, conn: &diesel::PgConnection) -> Result<Vec<NoteLink>, failure::Error> {
        let links = DatabaseNoteLink::load_by_note(conn, self.id)?;
        Ok(links.into_iter().map(Into::into).collect())
    }
}
