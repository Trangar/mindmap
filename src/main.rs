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

#[deprecated(note = "Use HtmlSafeString instead")]
fn replace_html_tags(s: &str) -> String {
    let mut output = String::with_capacity(s.len());
    for c in s.chars() {
        // Taken from https://www.owasp.org/index.php/XSS_(Cross_Site_Scripting)_Prevention_Cheat_Sheet#RULE_.231_-_HTML_Escape_Before_Inserting_Untrusted_Data_into_HTML_Element_Content
        match c {
            '&' => output += "&amp;",
            '<' => output += "&lt;",
            '>' => output += "&gt;",
            '"' => output += "&quot;",
            '\'' => output += "&#x27;",
            '/' => output += "&#x2F;",
            c => output.push(c),
        }
    }
    output
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
