use rocket::http::{Cookie, Cookies};
use rocket::request::Form;
use rocket::response::Redirect;
use rocket_contrib::templates::Template;
use std::collections::HashMap;
use std::net::SocketAddr;

use crate::either::Either;
use crate::user::User;
use crate::{HtmlSafeString, MindmapDB};

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
    let login = login.into_inner();
    match User::attempt_login(
        &conn,
        &login.username,
        &login.password,
        &ip.ip().to_string(),
    ) {
        Ok((user, token)) => {
            cookies.add_private(Cookie::new("UID", user.id.to_string()));
            cookies.add_private(Cookie::new("TID", token.id.to_string()));
            Either::Right(Redirect::to("/"))
        }
        Err(e) => {
            let render_model = LoginRenderModel {
                username: login.username.get(),
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
    let register = register.into_inner();

    if register.password != register.repeat_password {
        let render_model = RegisterRenderModel {
            username: register.username.get(),
            error: String::from("Passwords don't match"),
        };
        return Either::Left(Template::render("register", &render_model));
    }
    match User::attempt_register(
        &conn,
        &register.username,
        &register.password,
        &ip.ip().to_string(),
    ) {
        Ok((user, token)) => {
            cookies.add_private(Cookie::new("UID", user.id.to_string()));
            cookies.add_private(Cookie::new("TID", token.id.to_string()));
            Either::Right(Redirect::to("/"))
        }
        Err(e) => {
            let render_model = RegisterRenderModel {
                username: register.username.get(),
                error: e.to_string(),
            };
            Either::Left(Template::render("register", &render_model))
        }
    }
}

#[derive(FromForm)]
pub struct LoginSubmitModel {
    pub username: HtmlSafeString,
    pub password: String,
}

#[derive(Default, Serialize)]
pub struct LoginRenderModel {
    pub username: String,
    pub error: String,
}

#[derive(FromForm)]
pub struct RegisterSubmitModel {
    pub username: HtmlSafeString,
    pub password: String,
    pub repeat_password: String,
}

#[derive(Default, Serialize)]
pub struct RegisterRenderModel {
    pub username: String,
    pub error: String,
}
