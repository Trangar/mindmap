use rocket::http::Status;
use rocket::request::Request;
use rocket::response::{Responder, Response};

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
