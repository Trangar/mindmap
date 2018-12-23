use crate::either::Either;
use crate::note::{Note, NoteHistory};
use crate::user::User;
use crate::MindmapDB;
use rocket::response::Redirect;
use rocket_contrib::templates::Template;
use std::path::PathBuf;

#[get("/history/<seo_name..>")]
pub fn view(
    conn: MindmapDB,
    user: User,
    seo_name: PathBuf,
) -> Result<Either<Template, Redirect>, failure::Error> {
    let seo_name = super::note::get_seo_name_from_path(&seo_name);
    match Note::load_by_seo_name(&conn, seo_name, user.id)? {
        Some(note) => {
            let history = note.load_history(&conn)?;
            let model = ViewNoteHistoryModel { note, history };
            Ok(Either::Left(Template::render("note_history", &model)))
        }
        None => Ok(Either::Right(Redirect::to("/"))),
    }
}

#[derive(Serialize)]
pub struct ViewNoteHistoryModel {
    pub note: Note,
    pub history: Vec<NoteHistory>,
}
