use rocket::response::Redirect;
use uuid::Uuid;

use crate::note::{Link, Note};
use crate::user::User;
use crate::MindmapDB;

#[get("/create_link/<left_seo_name>/<right_seo_name>")]
pub fn create(
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
pub fn follow(
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
