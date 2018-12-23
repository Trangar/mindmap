use rocket::http::{Cookie, Cookies};
use rocket::request::Form;
use rocket::response::Redirect;
use rocket_contrib::templates::Template;
use std::collections::HashMap;
use std::net::SocketAddr;

use crate::either::Either;
use crate::user::User;
use crate::{replace_html_tags, MindmapDB};

#[get("/", rank = 2)]
pub fn index_not_logged_in() -> Template {
    let map = HashMap::<(), ()>::new();
    Template::render("index_not_logged_in", &map)
}

#[get("/logout")]
pub fn logout(mut cookies: Cookies) -> Redirect {
    let names: Vec<String> = cookies.iter().map(|c| c.name().to_owned()).collect();
    for name in names {
        cookies.remove(Cookie::named(name));
    }
    Redirect::to("/")
}

#[post("/login", data = "<login>")]
pub fn login_submit(
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

#[post("/register", data = "<register>")]
pub fn register_submit(
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
        &replace_html_tags(&register.username),
        &replace_html_tags(&register.password),
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
pub struct LoginSubmitModel {
    pub username: String,
    pub password: String,
}

#[derive(Default, Serialize)]
pub struct LoginRenderModel {
    pub username: String,
    pub error: String,
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
