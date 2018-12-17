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

pub mod models;
pub mod note;
pub mod schema;
pub mod user;

use failure::bail;
use rocket::http::{Cookie, Cookies, Status};
use rocket::request::{FlashMessage, Form, Request};
use rocket::response::{Flash, Redirect, Responder, Response};
use rocket_contrib::databases::diesel::PgConnection;
use rocket_contrib::templates::Template;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub use crate::note::Note;
pub use crate::user::User;
// TODO: Move this to src/user_token.rs
pub use crate::models::user_token::UserToken;

// Rocket logic

#[database("mindmap_db")]
pub struct MindmapDB(PgConnection);

fn main() {
    rocket::ignite()
        .attach(MindmapDB::fairing())
        .attach(Template::fairing())
        .mount(
            "/",
            routes![
                index,
                index_after_login,
                login_submit,
                register_submit,
                search,
                new_note,
                view_note,
            ],
        )
        .launch();
}

pub enum Either<Left, Right> {
    Left(Left),
    Right(Right),
}

impl<'a, Left, Right> Responder<'a> for Either<Left, Right>
where
    Left: Responder<'a>,
    Right: Responder<'a>,
{
    fn respond_to(self, req: &Request) -> Result<Response<'a>, Status> {
        match self {
            Either::Left(l) => l.respond_to(req),
            Either::Right(r) => r.respond_to(req),
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
fn index_after_login(
    conn: MindmapDB,
    flash: Option<FlashMessage>,
    mut cookies: Cookies,
) -> Result<Template, failure::Error> {
    if let Some(flash) = flash {
        let mut split = flash.msg().split(':').map(|s| s.parse());
        let user_id: Uuid = match split.next() {
            Some(Ok(id)) => id,
            _ => bail!("Hello, hacker man"),
        };
        let token_id: Uuid = match split.next() {
            Some(Ok(id)) => id,
            _ => bail!("Hello, hacker man"),
        };

        cookies.add_private(Cookie::new("UID", user_id.to_string()));
        cookies.add_private(Cookie::new("TID", token_id.to_string()));

        let user = User::load_by_id(&conn, user_id)?;

        index(conn, user)
    } else {
        let map = HashMap::<(), ()>::new();
        Ok(Template::render("index_not_logged_in", &map))
    }
}

// Search logic

// TODO: https://www.compose.com/articles/mastering-postgresql-tools-full-text-search-and-phrase-search/
#[get("/search?<q>")]
fn search(_conn: MindmapDB, _user: User, q: String) -> Result<Template, failure::Error> {
    let results = SearchResults {
        search: q,
        results: Vec::new(),
    };
    Ok(Template::render("search", &results))
}

#[derive(Serialize)]
struct SearchResults {
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

#[get("/n/<seo_name..>")]
fn view_note(
    conn: MindmapDB,
    user: User,
    seo_name: PathBuf,
) -> Result<Either<Template, Redirect>, failure::Error> {
    let mut first: &Path = &seo_name;
    while let Some(parent) = first.parent() {
        if parent.to_str().unwrap().is_empty() {
            break;
        }
        first = parent;
    }
    let seo_name = first.to_str().unwrap();
    println!("Seo name: {:?}", seo_name);
    match Note::load_by_seo_name(&conn, seo_name, user.id)? {
        Some(mut note) => {
            note.increase_view_count(&conn)?;
            let model = ViewNoteModel { note };
            Ok(Either::Left(Template::render("note", model)))
        }
        None => Ok(Either::Right(Redirect::to("/"))),
    }
}

#[derive(Serialize)]
struct ViewNoteModel {
    pub note: Note,
}

// Login logic

#[post("/login", data = "<login>")]
fn login_submit(
    ip: SocketAddr,
    conn: MindmapDB,
    login: Form<LoginSubmitModel>,
) -> Either<Template, Flash<Redirect>> {
    match User::attempt_login(
        &conn,
        login.username.as_str(),
        login.password.as_str(),
        &ip.ip().to_string(),
    ) {
        Ok((user, token)) => {
            // TODO: Set tokens
            Either::Right(Flash::success(
                Redirect::to("/"),
                format!("{}:{}", user.id, token.id),
            ))
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
    register: Form<RegisterSubmitModel>,
) -> Either<Template, Flash<Redirect>> {
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
        Ok((user, token)) => Either::Right(Flash::success(
            Redirect::to("/"),
            format!("{}:{}", user.id, token.id),
        )),
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
