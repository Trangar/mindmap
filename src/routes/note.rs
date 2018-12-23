use rocket::http::RawStr;
use rocket::request::Form;
use rocket::request::FromFormValue;
use rocket::response::Redirect;
use rocket_contrib::templates::Template;
use std::path::{Path, PathBuf};

use crate::either::Either;
use crate::note::{Note, NoteLink};
use crate::user::User;
use crate::{HtmlSafeString, MindmapDB};

pub fn get_seo_name_from_path(p: &Path) -> &str {
    let mut first = p;
    while let Some(parent) = first.parent() {
        if parent.to_str().unwrap().is_empty() {
            break;
        }
        first = parent;
    }
    first.to_str().unwrap()
}

#[post("/new_note", data = "<data>")]
pub fn new(conn: MindmapDB, user: User, data: Form<NewNote>) -> Result<Redirect, failure::Error> {
    let note = Note::create(&conn, &data.title, &data.body, user.id)?;
    Ok(Redirect::to(format!("/n/{}", note.seo_name)))
}

#[get("/n/<seo_name..>")]
pub fn view(
    conn: MindmapDB,
    user: User,
    seo_name: PathBuf,
) -> Result<Either<Template, Redirect>, failure::Error> {
    let seo_name = get_seo_name_from_path(&seo_name);
    match Note::load_by_seo_name(&conn, seo_name, user.id)? {
        Some(mut note) => {
            note.increase_view_count(&conn)?;
            let links = note.load_links(&conn)?;
            let model = ViewNoteModel { note, links };
            Ok(Either::Left(Template::render("note", model)))
        }
        None => Ok(Either::Right(Redirect::to("/"))),
    }
}

#[get("/delete/<seo_name..>")]
pub fn delete_preview(
    conn: MindmapDB,
    user: User,
    seo_name: PathBuf,
) -> Result<Either<Template, Redirect>, failure::Error> {
    let seo_name = get_seo_name_from_path(&seo_name);
    match Note::load_by_seo_name(&conn, seo_name, user.id)? {
        Some(note) => {
            let model = DeletePreviewModel { note };
            Ok(Either::Left(Template::render("delete_preview", model)))
        }
        None => Ok(Either::Right(Redirect::to("/"))),
    }
}

#[post("/delete/<seo_name..>", data = "<data>")]
pub fn delete_submit(
    conn: MindmapDB,
    user: User,
    seo_name: PathBuf,
    data: Form<DeleteSubmitModel>,
) -> Result<Redirect, failure::Error> {
    match data.action {
        DeleteActionType::Cancel => Ok(Redirect::to(format!("/n/{}", seo_name.to_str().unwrap()))),
        DeleteActionType::Delete => {
            let seo_name = get_seo_name_from_path(&seo_name);
            Note::delete_by_seo_name(&conn, seo_name, user.id)?;
            Ok(Redirect::to("/"))
        }
    }
}
#[get("/edit/<seo_name..>")]
pub fn edit(
    conn: MindmapDB,
    user: User,
    seo_name: PathBuf,
) -> Result<Either<Template, Redirect>, failure::Error> {
    let seo_name = get_seo_name_from_path(&seo_name);
    match Note::load_by_seo_name(&conn, seo_name, user.id)? {
        Some(note) => {
            let model = EditNoteModel { note };
            Ok(Either::Left(Template::render("edit_note", model)))
        }
        None => Ok(Either::Right(Redirect::to("/"))),
    }
}

#[post("/edit/<seo_name..>", data = "<data>")]
pub fn edit_submit(
    conn: MindmapDB,
    user: User,
    seo_name: PathBuf,
    data: Form<SaveNoteModel>,
) -> Result<Either<Template, Redirect>, failure::Error> {
    let seo_name = get_seo_name_from_path(&seo_name);
    match Note::load_by_seo_name(&conn, seo_name, user.id)? {
        Some(mut note) => {
            note.update(&conn, &data.title, &data.body)?;
            Ok(Either::Right(Redirect::to(format!("/n/{}", note.seo_name))))
        }
        None => Ok(Either::Right(Redirect::to("/"))),
    }
}

#[derive(Serialize)]
pub struct DeletePreviewModel {
    pub note: Note,
}

#[derive(Serialize)]
pub struct EditNoteModel {
    pub note: Note,
}

#[derive(FromForm, Debug)]
pub struct SaveNoteModel {
    pub title: HtmlSafeString,
    pub body: HtmlSafeString,
}

#[derive(FromForm)]
pub struct NewNote {
    pub title: HtmlSafeString,
    pub body: HtmlSafeString,
}
#[derive(FromForm, Debug)]
pub struct DeleteSubmitModel {
    pub action: DeleteActionType,
}

#[derive(Debug)]
pub enum DeleteActionType {
    Cancel,
    Delete,
}

impl<'v> FromFormValue<'v> for DeleteActionType {
    type Error = &'v RawStr;

    fn from_form_value(form_value: &'v RawStr) -> Result<DeleteActionType, &'v RawStr> {
        match form_value.as_str() {
            "cancel" => Ok(DeleteActionType::Cancel),
            "delete" => Ok(DeleteActionType::Delete),
            _ => Err(form_value),
        }
    }
}

#[derive(Serialize)]
pub struct ViewNoteModel {
    pub note: Note,
    pub links: Vec<NoteLink>,
}
