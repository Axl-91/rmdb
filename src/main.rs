#[macro_use]
extern crate rocket;

mod auth;
mod controllers;
mod middleware;
use middleware::log_check::LoggedUser;
use rocket::fs::{relative, FileServer};
use rocket_db_pools::Database;
use rocket_dyn_templates::{context, Template};
use std::env;

#[derive(Database)]
#[database("postgres_db")]
struct Db(sqlx::PgPool);

#[get("/")]
async fn home(logged_user: LoggedUser) -> Template {
    Template::render("home", context! {user_email: logged_user.email})
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
        .attach(Db::init())
        .attach(controllers::movies::stage())
        .attach(controllers::users::stage())
        .mount("/", FileServer::from(relative!("templates")))
        .mount("/", routes![home])
}
