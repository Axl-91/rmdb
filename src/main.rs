#[macro_use]
extern crate rocket;

mod movies;
use rocket::fs::{relative, FileServer};
use rocket_dyn_templates::{context, Template};
use std::env;

#[get("/")]
async fn home() -> Template {
    let hello = "Hello, world".to_string();
    Template::render("home", context! { message: hello })
}

#[launch]
fn rocket() -> _ {
    dotenv::dotenv().ok();

    let figment = rocket::Config::figment().merge((
        "databases.postgres_db.url",
        env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
    ));

    rocket::custom(figment)
        .attach(Template::fairing())
        .attach(movies::stage())
        .mount("/", FileServer::from(relative!("templates")))
        .mount("/", routes![home])
}
