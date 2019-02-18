use rocket::{get, Rocket};

pub(crate) mod app;
pub(crate) mod http;

fn main() {
    let settings = app::Settings::new().unwrap().into();
    Rocket::custom(settings).launch();
}
