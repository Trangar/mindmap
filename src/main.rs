#![feature(proc_macro_hygiene, decl_macro)]
#![allow(proc_macro_derive_resolution_fallback)]

#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;

mod either;
pub mod models;
pub mod note;
pub mod schema;
pub mod user;

use rocket::http::{Cookie, Cookies};
use rocket::request::Form;
use rocket::response::Redirect;
use rocket_contrib::databases::diesel::PgConnection;
use rocket_contrib::templates::Template;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub use crate::either::Either;
pub use crate::note::{Link, Note, NoteHistory, NoteLink};
pub use crate::user::User;
// TODO: Move this to src/user_token.rs
pub use crate::models::user_token::UserToken;

// Rocket logic

#[database("mindmap_db")]
pub struct MindmapDB(PgConnection);

fn main() {
    rocket::ignite()
        .attach(MindmapDB::fairing())
        .attach(Template::custom(|engine| {
            engine.tera.register_filter("markdown", markdown_filter);
        }))
        .mount(
            "/",
            routes![
                index,
                index_not_logged_in,
                // index_after_login,
                login_submit,
                logout,
                register_submit,
                search,
                search_note_link,
                create_note_link,
                follow_link,
                new_note,
                edit_note,
                edit_note_submit,
                view_note,
                view_note_history,
            ],
        )
        .launch();
}

use crate::filters::markdown_filter;
mod filters {
    use pulldown_cmark::{html, Parser};
    use rocket_contrib::templates::tera::{Error, ErrorKind};
    use serde_json::Value;
    use std::collections::HashMap;

    pub fn markdown_filter<S: std::hash::BuildHasher>(
        v: Value,
        _data: HashMap<String, Value, S>,
    ) -> Result<Value, Error> {
        if let Some(s) = v.as_str() {
            let parser = Parser::new(s);
            let mut html_buf = String::new();
            html::push_html(&mut html_buf, parser);
            Ok(Value::String(html_buf))
        } else {
            Err(Error::from_kind(ErrorKind::Msg(String::from(
                "Value is not a valid string",
            ))))
        }
    }
}

// Main page

#[get("/", rank = 1)]
fn index(conn: MindmapDB, user: User) -> Result<Template, failure::Error> {
    let notes = Note::load_top_ten(&conn, user.id)?;
    let model = IndexModel {
        top_10_notes: notes,
    };
    Ok(Template::render("index", &model))
}

#[derive(Serialize)]
pub struct IndexModel {
    pub top_10_notes: Vec<Note>,
}

#[get("/", rank = 2)]
fn index_not_logged_in() -> Template {
    let map = HashMap::<(), ()>::new();
    Template::render("index_not_logged_in", &map)
}

#[get("/logout")]
fn logout(mut cookies: Cookies) -> Redirect {
    let names: Vec<String> = cookies.iter().map(|c| c.name().to_owned()).collect();
    for name in names {
        cookies.remove(Cookie::named(name));
    }
    Redirect::to("/")
}

// Search logic

#[get("/search?<q>")]
fn search(conn: MindmapDB, user: User, q: String) -> Result<Template, failure::Error> {
    let mut query = SearchQuery::default();

    for part in q.split(' ') {
        if Some("-") == part.get(..1) {
            query.excludes.push(&part[1..])
        } else {
            query.queries.push(part);
        }
    }

    let results = Note::search(&conn, query, user.id)?;

    let results = SearchResults { search: q, results };
    Ok(Template::render("search", &results))
}

#[derive(Serialize)]
struct SearchResults {
    pub search: String,
    pub results: Vec<Note>,
}

#[get("/create_link/<seo_name>?<q>")]
fn search_note_link(
    conn: MindmapDB,
    user: User,
    seo_name: String,
    q: String,
) -> Result<Either<Template, Redirect>, failure::Error> {
    match Note::load_by_seo_name(&conn, &seo_name, user.id)? {
        Some(note) => {
            let mut query = SearchQuery::default();

            for part in q.split(' ') {
                if Some("-") == part.get(..1) {
                    query.excludes.push(&part[1..])
                } else {
                    query.queries.push(part);
                }
            }

            let results = Note::search(&conn, query, user.id)?;

            let results = SearchLinkResults {
                search: q,
                results,
                note,
            };
            Ok(Either::Left(Template::render("search_link", &results)))
        }
        None => Ok(Either::Right(Redirect::to("/"))),
    }
}
#[get("/create_link/<left_seo_name>/<right_seo_name>")]
fn create_note_link(
    conn: MindmapDB,
    user: User,
    left_seo_name: String,
    right_seo_name: String,
) -> Result<Redirect, failure::Error> {
    match (
        Note::load_by_seo_name(&conn, &left_seo_name, user.id)?,
        Note::load_by_seo_name(&conn, &right_seo_name, user.id)?,
    ) {
        (Some(left), Some(right)) => {
            left.create_link_to(&conn, &right)?;
            Ok(Redirect::to(format!("/n/{}", left.seo_name)))
        }
        (_, _) => Ok(Redirect::to("/")),
    }
}

#[get("/link/<id>/<seo_name>")]
fn follow_link(
    conn: MindmapDB,
    _user: User,
    id: String,
    seo_name: String,
) -> Result<Redirect, failure::Error> {
    let id = Uuid::parse_str(&id)?;
    let link = Link { id };
    link.increase_click_count(&conn)?;
    Ok(Redirect::to(format!("/n/{}", seo_name)))
}

#[derive(Debug, Default)]
pub struct SearchQuery<'a> {
    pub queries: Vec<&'a str>,
    pub excludes: Vec<&'a str>,
}

#[derive(Serialize)]
struct SearchLinkResults {
    pub note: Note,
    pub search: String,
    pub results: Vec<Note>,
}

// Note insert

#[post("/new_note", data = "<data>")]
fn new_note(conn: MindmapDB, user: User, data: Form<NewNote>) -> Result<Redirect, failure::Error> {
    let note = Note::create(&conn, &data.title, &data.body, user.id)?;
    Ok(Redirect::to(format!("/n/{}", note.seo_name)))
}

#[derive(FromForm)]
struct NewNote {
    pub title: String,
    pub body: String,
}

// View note

fn get_seo_name_from_path(p: &Path) -> &str {
    let mut first = p;
    while let Some(parent) = first.parent() {
        if parent.to_str().unwrap().is_empty() {
            break;
        }
        first = parent;
    }
    first.to_str().unwrap()
}

#[get("/n/<seo_name..>")]
fn view_note(
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

#[derive(Serialize)]
struct ViewNoteModel {
    pub note: Note,
    pub links: Vec<NoteLink>,
}

#[get("/edit/<seo_name..>")]
fn edit_note(
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
fn edit_note_submit(
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
struct EditNoteModel {
    pub note: Note,
}

#[derive(FromForm, Debug)]
struct SaveNoteModel {
    pub title: String,
    pub body: String,
}

#[get("/history/<seo_name..>")]
fn view_note_history(
    conn: MindmapDB,
    user: User,
    seo_name: PathBuf,
) -> Result<Either<Template, Redirect>, failure::Error> {
    let seo_name = get_seo_name_from_path(&seo_name);
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

// Login logic

#[post("/login", data = "<login>")]
fn login_submit(
    ip: SocketAddr,
    conn: MindmapDB,
    mut cookies: Cookies,
    login: Form<LoginSubmitModel>,
) -> Either<Template, Redirect> {
    match User::attempt_login(
        &conn,
        login.username.as_str(),
        login.password.as_str(),
        &ip.ip().to_string(),
    ) {
        Ok((user, token)) => {
            cookies.add_private(Cookie::new("UID", user.id.to_string()));
            cookies.add_private(Cookie::new("TID", token.id.to_string()));
            Either::Right(Redirect::to("/"))
        }
        Err(e) => {
            let render_model = LoginRenderModel {
                username: login.username.clone(),
                error: e.to_string(),
            };
            Either::Left(Template::render("login", &render_model))
        }
    }
}

#[derive(FromForm)]
pub struct LoginSubmitModel {
    pub username: String,
    pub password: String,
}

#[derive(Default, Serialize)]
pub struct LoginRenderModel {
    pub username: String,
    pub error: String,
}

// Register logic

#[post("/register", data = "<register>")]
fn register_submit(
    ip: SocketAddr,
    conn: MindmapDB,
    mut cookies: Cookies,
    register: Form<RegisterSubmitModel>,
) -> Either<Template, Redirect> {
    if register.password != register.repeat_password {
        let render_model = RegisterRenderModel {
            username: register.username.clone(),
            error: String::from("Passwords don't match"),
        };
        return Either::Left(Template::render("register", &render_model));
    }
    match User::attempt_register(
        &conn,
        register.username.as_str(),
        register.password.as_str(),
        &ip.ip().to_string(),
    ) {
        Ok((user, token)) => {
            cookies.add_private(Cookie::new("UID", user.id.to_string()));
            cookies.add_private(Cookie::new("TID", token.id.to_string()));
            Either::Right(Redirect::to("/"))
        }
        Err(e) => {
            let render_model = RegisterRenderModel {
                username: register.username.clone(),
                error: e.to_string(),
            };
            Either::Left(Template::render("register", &render_model))
        }
    }
}

#[derive(FromForm)]
pub struct RegisterSubmitModel {
    pub username: String,
    pub password: String,
    pub repeat_password: String,
}

#[derive(Default, Serialize)]
pub struct RegisterRenderModel {
    pub username: String,
    pub error: String,
}