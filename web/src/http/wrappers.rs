use rocket_contrib::templates::Template;

use rocket::http::Status;
use rocket::request::Request;
use rocket::response::{Flash, NamedFile, Redirect, Responder, Response};

pub enum VaryingResponse {
    Template(Template),
    File(NamedFile),
    Redirect(Redirect),
    Flash(Flash<Redirect>),
}

impl<'r> Responder<'r> for VaryingResponse {
    fn respond_to(self, request: &Request) -> Result<Response<'r>, Status> {
        use self::VaryingResponse::*;

        match self {
            Template(r) => r.respond_to(request),
            File(r) => r.respond_to(request),
            Redirect(r) => r.respond_to(request),
            Flash(r) => r.respond_to(request),
        }
    }
}
