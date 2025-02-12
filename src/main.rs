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

#[post("/movies", format = "json", data = "<movie>")]
async fn create(mut db: Connection<Db>, movie: Json<Movie>) -> Result<String, String> {
    let result = sqlx::query!(
        "INSERT INTO movies(name, director) VALUES ($1, $2)",
        movie.name,
        movie.director
    )
    .execute(&mut **db)
    .await;

    match result {
        Ok(_) => Ok("Item added successfully!".to_string()),
        Err(err) => Err(format!("Failed to insert item: {}", err)),
    }
}

#[get("/movies/<id>")]
async fn show(mut db: Connection<Db>, id: i32) -> Result<Json<Movie>, String> {
    let movie = sqlx::query_as!(Movie, "SELECT name, director FROM movies WHERE id = $1", id)
        .fetch_one(&mut **db)
        .await;

    match movie {
        Ok(movie) => Ok(Json(movie)),
        Err(err) => Err(format!("Failed to fetch movie: {}", err)),
    }
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
        .mount("/", routes![home, index, create, show])
}
