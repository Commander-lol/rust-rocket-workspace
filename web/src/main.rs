use rocket::{get, Rocket};

use rocket_contrib::serve::{StaticFiles, Options};

pub(crate) mod app;
pub(crate) mod http;

fn main() {
    let settings = app::Settings::new().unwrap();
    Rocket::custom(settings.clone().into())
        .mount(&settings.static_route, StaticFiles::new(settings.static_dir, Options::None))
        .launch();
}
