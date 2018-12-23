use crate::note::Note;
use crate::user::User;
use crate::MindmapDB;
use rocket::Route;
use rocket_contrib::templates::Template;

mod auth;
mod link;
mod note;
mod note_history;
mod search;

pub use self::search::SearchQuery;

pub fn get() -> Vec<Route> {
    routes![
        index,
        auth::index_not_logged_in,
        auth::login_submit,
        auth::logout,
        auth::register_submit,
        link::create,
        link::follow,
        note_history::view,
        note::new,
        note::edit,
        note::edit_submit,
        note::view,
        note::delete_preview,
        note::delete_submit,
        search::search,
        search::search_for_link,
    ]
}

#[get("/?<page>&<count>", rank = 1)]
fn index(
    conn: MindmapDB,
    user: User,
    page: Option<u64>,
    count: Option<u64>,
) -> Result<Template, failure::Error> {
    let page = page.unwrap_or(1);
    let count = count.unwrap_or(100);

    let notes = Note::load_paged(&conn, user.id, (page - 1) * count, count)?;
    let total_notes = Note::count_all(&conn, user.id)?;
    let model = IndexModel {
        notes: notes,
        page,
        total_pages: (total_notes / count) + 1,
        notes_per_page: count,
    };
    Ok(Template::render("index", &model))
}

#[derive(Serialize)]
pub struct IndexModel {
    pub notes: Vec<Note>,
    pub page: u64,
    pub total_pages: u64,
    pub notes_per_page: u64,
}
