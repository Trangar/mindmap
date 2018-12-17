use crate::models::user::DatabaseUser;
use crate::models::user_token::UserToken;
use crate::MindmapDB;
use crypto::pbkdf2::pbkdf2_check;
use crypto::pbkdf2::pbkdf2_simple;
use failure::{bail, format_err};
use rocket::http::Status;
use rocket::request::FromRequest;
use rocket::{request, Outcome, Request};
use std::str::FromStr;
use uuid::Uuid;

pub struct User {
    pub id: Uuid,
    pub name: String,
}

impl From<DatabaseUser> for User {
    fn from(u: DatabaseUser) -> User {
        User {
            id: u.id,
            name: u.name,
        }
    }
}

impl<'a, 'b> FromRequest<'a, 'b> for User {
    type Error = failure::Error;

    fn from_request(req: &'a Request<'b>) -> request::Outcome<Self, Self::Error> {
        let uid: Uuid = match req
            .cookies()
            .get_private("UID")
            .map(|c| Uuid::from_str(c.value()))
        {
            Some(Ok(c)) => c,
            _ => return Outcome::Forward(()),
        };
        let tid: Uuid = match req
            .cookies()
            .get_private("TID")
            .map(|c| Uuid::from_str(c.value()))
        {
            Some(Ok(c)) => c,
            _ => return Outcome::Forward(()),
        };
        let connection = MindmapDB::from_request(req).unwrap();
        let mut token = match UserToken::load_by_user_and_token_id(&connection, uid, tid) {
            Ok(Some(t)) => t,
            Ok(None) => return Outcome::Forward(()),
            Err(e) => return Outcome::Failure((Status::InternalServerError, e)),
        };

        if !token.active {
            return Outcome::Forward(());
        }

        if let Err(e) = token.update_last_used(&connection) {
            return Outcome::Failure((Status::InternalServerError, e));
        }

        match DatabaseUser::load_by_id(&connection, uid) {
            Ok(Some(u)) => Outcome::Success(u.into()),
            Ok(None) => Outcome::Forward(()),
            Err(e) => Outcome::Failure((Status::InternalServerError, e)),
        }
    }
}

impl User {
    pub fn attempt_login(
        conn: &MindmapDB,
        name: &str,
        password: &str,
        ip: &str,
    ) -> Result<(User, UserToken), failure::Error> {
        let user = match DatabaseUser::load_by_name(conn, name)? {
            Some(u) => u,
            None => {
                // TODO: Validate password anyway to prevent timing attacks
                bail!("Login credentials are invalid");
            }
        };
        let result = pbkdf2_check(password, &user.password)
            .map_err(|e| format_err!("Could not validate password: {}", e))?;

        if !result {
            bail!("Login credentials are invalid");
        }
        let token = UserToken::create(conn, user.id, ip)?;
        Ok((user.into(), token))
    }

    pub fn attempt_register(
        conn: &MindmapDB,
        name: &str,
        password: &str,
        ip: &str,
    ) -> Result<(User, UserToken), failure::Error> {
        if DatabaseUser::load_by_name(conn, name)?.is_some() {
            bail!("Username already in use");
        }

        let password = pbkdf2_simple(password, 10_000)?;

        let user = DatabaseUser::create(conn, name, &password)?;
        let token = UserToken::create(conn, user.id, ip)?;

        Ok((user.into(), token))
    }

    pub fn load_by_id(conn: &MindmapDB, id: Uuid) -> Result<User, failure::Error> {
        match DatabaseUser::load_by_id(conn, id)? {
            Some(u) => Ok(u.into()),
            None => bail!("User not found"),
        }
    }
}
