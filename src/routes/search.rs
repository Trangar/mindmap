use crate::either::Either;
use crate::note::Note;
use crate::user::User;
use crate::{HtmlSafeString, MindmapDB};
use rocket::response::Redirect;
use rocket_contrib::templates::Template;

#[get("/search?<q>")]
pub fn search(conn: MindmapDB, user: User, q: HtmlSafeString) -> Result<Template, failure::Error> {
    let mut query = SearchQuery::default();

    for part in q.split(' ') {
        if Some("-") == part.get(..1) {
            query.excludes.push(&part[1..])
        } else {
            query.queries.push(part);
        }
    }

    let results = Note::search(&conn, query, user.id)?;

    let results = SearchResults {
        search: q.get(),
        results,
    };
    Ok(Template::render("search", &results))
}

#[get("/create_link/<seo_name>?<q>")]
pub fn search_for_link(
    conn: MindmapDB,
    user: User,
    seo_name: String,
    q: HtmlSafeString,
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
                search: q.get(),
                results,
                note,
            };
            Ok(Either::Left(Template::render("search_link", &results)))
        }
        None => Ok(Either::Right(Redirect::to("/"))),
    }
}

#[derive(Serialize)]
struct SearchResults {
    pub search: String,
    pub results: Vec<Note>,
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
