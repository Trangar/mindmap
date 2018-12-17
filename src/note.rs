use crate::models::note::Note as DatabaseNote;
use slug::slugify;
use uuid::Uuid;

#[derive(Serialize)]
pub struct Note {
    pub id: Uuid,
    pub seo_name: String,
    pub title: String,
    pub body: String,
}

impl From<DatabaseNote> for Note {
    fn from(n: DatabaseNote) -> Note {
        Note {
            id: n.id,
            seo_name: n.seo_name,
            title: n.title,
            body: n.body,
        }
    }
}

impl Note {
    pub fn load_top_ten(
        conn: &diesel::PgConnection,
        user_id: Uuid,
    ) -> Result<Vec<Note>, failure::Error> {
        Ok(DatabaseNote::load_top_10(conn, user_id)?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    pub fn load_by_seo_name(
        conn: &diesel::PgConnection,
        name: &str,
        user_id: Uuid,
    ) -> Result<Option<Note>, failure::Error> {
        DatabaseNote::load_by_seo_name(conn, name, user_id).map(|o| o.map(Into::into))
    }

    pub fn increase_view_count(
        &mut self,
        conn: &diesel::PgConnection,
    ) -> Result<(), failure::Error> {
        DatabaseNote::increase_view_count(conn, self.id)?;
        Ok(())
    }

    pub fn create(
        conn: &diesel::PgConnection,
        title: &str,
        body: &str,
        user_id: Uuid,
    ) -> Result<Note, failure::Error> {
        let seo_name_base = slugify(title);
        let mut seo_name = seo_name_base.clone();
        let mut counter = 1;
        while let Some(_) = DatabaseNote::load_by_seo_name(conn, &seo_name, user_id)? {
            seo_name = format!("{}_{}", seo_name_base, counter);
            counter += 1;
        }

        Ok(DatabaseNote::create(conn, &seo_name, title, body, user_id)?.into())
    }
}
