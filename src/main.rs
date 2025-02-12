#[macro_use]
extern crate rocket;

use rocket::serde::json::Json;
use rocket_db_pools::{sqlx, Connection, Database};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Database)]
#[database("postgres_db")]
struct Db(sqlx::PgPool);

#[derive(Debug, Serialize, Deserialize)]
struct Movie {
    name: Option<String>,
    director: Option<String>,
}

#[get("/")]
async fn home() -> &'static str {
    "Hello, world"
}

#[get("/movies")]
async fn index(mut db: Connection<Db>) -> Json<Vec<Movie>> {
    let movies = sqlx::query_as!(Movie, "SELECT name, director FROM movies")
        .fetch_all(&mut **db)
        .await
        .unwrap();

    Json(movies)
}

#[launch]
fn rocket() -> _ {
    dotenv::dotenv().ok();

    let figment = rocket::Config::figment().merge((
        "databases.postgres_db.url",
        env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
    ));

    rocket::custom(figment)
        .attach(Db::init())
        .mount("/", routes![home, index])
}
