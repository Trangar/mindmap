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

pub mod either;
pub mod models;
pub mod note;
pub mod routes;
pub mod schema;
pub mod tera_utils;
pub mod user;

use rocket::http::RawStr;
use rocket::request::FromFormValue;
use rocket_contrib::databases::diesel::PgConnection;
use rocket_contrib::serve::StaticFiles;
use rocket_contrib::templates::Template;

// Rocket logic

#[database("mindmap_db")]
pub struct MindmapDB(PgConnection);

#[derive(Debug)]
pub struct HtmlSafeString(String);

impl HtmlSafeString {
    pub fn get(self) -> String {
        self.0
    }
}

impl std::ops::Deref for HtmlSafeString {
    type Target = str;
    fn deref(&self) -> &str {
        self.0.as_str()
    }
}

impl<'v> FromFormValue<'v> for HtmlSafeString {
    type Error = &'v RawStr;

    fn from_form_value(form_value: &'v RawStr) -> Result<HtmlSafeString, &'v RawStr> {
        Ok(HtmlSafeString(form_value.html_escape().to_string()))
    }
}

fn main() {
    rocket::ignite()
        .attach(MindmapDB::fairing())
        .attach(Template::custom(|engine| {
            crate::tera_utils::register(&mut engine.tera);
        }))
        .mount("/", crate::routes::get())
        .mount("/", StaticFiles::from("static"))
        .launch();
}
