#[macro_use]
extern crate rocket;

mod movies;
use std::env;

#[get("/")]
async fn home() -> &'static str {
    "Hello, world"
}

#[launch]
fn rocket() -> _ {
    dotenv::dotenv().ok();

    let figment = rocket::Config::figment().merge((
        "databases.postgres_db.url",
        env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
    ));

    rocket::custom(figment)
        .mount("/", routes![home])
        .attach(movies::stage())
}
