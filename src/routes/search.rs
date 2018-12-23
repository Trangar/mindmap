use crate::either::Either;
use crate::note::Note;
use crate::user::User;
use crate::{replace_html_tags, MindmapDB};
use rocket::response::Redirect;
use rocket_contrib::templates::Template;

#[get("/search?<q>")]
pub fn search(conn: MindmapDB, user: User, q: String) -> Result<Template, failure::Error> {
    let mut query = SearchQuery::default();

    for part in q.split(' ') {
        if Some("-") == part.get(..1) {
            query.excludes.push(replace_html_tags(&part[1..]))
        } else {
            query.queries.push(replace_html_tags(part));
        }
    }

    let results = Note::search(&conn, query, user.id)?;

    let results = SearchResults {
        search: replace_html_tags(q.as_str()),
        results,
    };
    Ok(Template::render("search", &results))
}

#[derive(Serialize)]
struct SearchResults {
    pub search: String,
    pub results: Vec<Note>,
}

#[get("/create_link/<seo_name>?<q>")]
pub fn search_note_link(
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
                    query.excludes.push(replace_html_tags(&part[1..]))
                } else {
                    query.queries.push(replace_html_tags(part));
                }
            }

            let results = Note::search(&conn, query, user.id)?;

            let results = SearchLinkResults {
                search: replace_html_tags(q.as_str()),
                results,
                note,
            };
            Ok(Either::Left(Template::render("search_link", &results)))
        }
        None => Ok(Either::Right(Redirect::to("/"))),
    }
}

#[derive(Debug, Default)]
pub struct SearchQuery {
    pub queries: Vec<String>,
    pub excludes: Vec<String>,
}

#[derive(Serialize)]
struct SearchLinkResults {
    pub note: Note,
    pub search: String,
    pub results: Vec<Note>,
}
