use rocket::{fairing::AdHoc, http::Status, serde::json::Json};
use rocket_db_pools::{sqlx, Connection, Database};
use serde::{Deserialize, Serialize};

type ErrorResp = (Status, String);

#[derive(Database)]
#[database("postgres_db")]
struct Db(sqlx::PgPool);

#[derive(Serialize, Deserialize)]
struct ErrorMessage {
    message: String,
    error: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Movie {
    name: Option<String>,
    director: Option<String>,
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
async fn show(mut db: Connection<Db>, id: i32) -> Result<Json<Movie>, ErrorResp> {
    let movie = sqlx::query_as!(Movie, "SELECT name, director FROM movies WHERE id = $1", id)
        .fetch_one(&mut **db)
        .await;

    match movie {
        Ok(movie) => Ok(Json(movie)),
        Err(err) => Err((
            Status::NotFound,
            format!("Failed to fetch movie: {:?}", err),
        )),
    }
}

#[delete("/movies/<id>")]
async fn delete(mut db: Connection<Db>, id: i32) -> Result<String, ErrorResp> {
    let result = sqlx::query!("DELETE FROM movies WHERE id = $1", id)
        .execute(&mut **db)
        .await;

    match result {
        Ok(_) => Ok("Movie register deleted successfully".to_string()),
        Err(err) => Err((
            Status::NotFound,
            format!("Failed to delete movie: {:?}", err),
        )),
    }
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Movies Stage", |rocket| async {
        rocket
            .attach(Db::init())
            .mount("/", routes![index, create, show, delete])
    })
}
