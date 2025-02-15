use rocket::{fairing::AdHoc, http::Status, serde::json::Json};
use rocket_db_pools::{sqlx, Connection, Database};
use rocket_dyn_templates::{context, Template};
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;

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
    id: String,
    name: Option<String>,
    director: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct NewMovie {
    name: String,
    director: String,
}

async fn get_all_movies(mut db: Connection<Db>) -> Vec<Movie> {
    sqlx::query_as!(Movie, "SELECT id, name, director FROM movies")
        .fetch_all(&mut **db)
        .await
        .unwrap()
}

#[get("/movies")]
async fn index(db: Connection<Db>) -> Template {
    let movies = get_all_movies(db).await;
    Template::render("movies/index", context! { movies: movies })
}

#[post("/movies", format = "json", data = "<movie>")]
async fn create(mut db: Connection<Db>, movie: Json<NewMovie>) -> Result<String, String> {
    let id = Uuid::new_v4();
    let result = sqlx::query!(
        "INSERT INTO movies(id, name, director) VALUES ($1, $2, $3)",
        id,
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
async fn show(mut db: Connection<Db>, id: String) -> Result<Json<Movie>, ErrorResp> {
    let uuid = Uuid::parse_str(&id).unwrap();
    let movie = sqlx::query_as!(
        Movie,
        "SELECT id, name, director FROM movies WHERE id = $1",
        uuid
    )
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
async fn delete(mut db: Connection<Db>, id: String) -> Template {
    let uuid = Uuid::parse_str(&id).unwrap();
    let result = sqlx::query!("DELETE FROM movies WHERE id = $1", uuid)
        .execute(&mut **db)
        .await;

    let movies = get_all_movies(db).await;

    match result {
        Ok(_) => Template::render(
            "movies/index",
            context! {movies: movies, alert: "Movie deleted successfully"},
        ),
        Err(err) => Template::render(
            "movies/index",
            context! {movies: movies, alert: format!("Failed to delete movie: {}", err)},
        ),
    }
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Movies Stage", |rocket| async {
        rocket
            .attach(Db::init())
            .mount("/", routes![index, create, show, delete])
    })
}
