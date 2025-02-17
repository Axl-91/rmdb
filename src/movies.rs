use rocket::{
    fairing::AdHoc,
    form::Form,
    http::{CookieJar, Status},
    response::Redirect,
    serde::json::Json,
};
use rocket_db_pools::{sqlx, Connection};
use rocket_dyn_templates::{context, tera, Template};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgQueryResult, types::Uuid};

use crate::{middleware::AuthenticatedUser, Db};

type ErrorResp = (Status, String);

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

#[derive(FromForm)]
struct FormMovie {
    name: String,
    director: String,
}

async fn get_movies(mut db: Connection<Db>) -> Vec<Movie> {
    sqlx::query_as!(Movie, "SELECT id, name, director FROM movies ORDER BY name")
        .fetch_all(&mut **db)
        .await
        .unwrap()
}

async fn get_movie(mut db: Connection<Db>, id: Uuid) -> Result<Movie, sqlx::Error> {
    sqlx::query_as!(
        Movie,
        "SELECT id, name, director FROM movies WHERE id = $1",
        id
    )
    .fetch_one(&mut **db)
    .await
}

async fn create_movie(
    mut db: Connection<Db>,
    name: &str,
    director: &str,
) -> Result<PgQueryResult, sqlx::Error> {
    let id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO movies(id, name, director) VALUES ($1, $2, $3)",
        id,
        name,
        director
    )
    .execute(&mut **db)
    .await
}

// async fn update_movie() {}

// async fn delete_movie() {}

#[get("/")]
async fn index(
    db: Connection<Db>,
    cookies: &CookieJar<'_>,
    auth_user: AuthenticatedUser,
) -> Template {
    let movies = get_movies(db).await;

    let mut context = tera::Context::new();
    context.insert("movies", &movies);

    let notice = cookies.get("notice").map(|n| n.value().to_string());

    cookies.remove("notice");

    Template::render(
        "movies/index",
        context! { movies: movies, notice: notice, user_email: auth_user.email},
    )
}

#[get("/new")]
async fn new() -> Template {
    Template::render("movies/new", context! {})
}

#[put("/create", data = "<form>")]
async fn create(db: Connection<Db>, form: Form<FormMovie>, cookies: &CookieJar<'_>) -> Redirect {
    let result = create_movie(db, &form.name, &form.director).await;

    match result {
        Ok(_) => cookies.add(("notice", "Movie created successfully")),
        Err(err) => cookies.add(("notice", format!("Movie couldn't be created: {}", err))),
    }

    Redirect::to(uri!("/movies"))
}

#[get("/<id>")]
async fn show(db: Connection<Db>, id: String) -> Result<Json<Movie>, ErrorResp> {
    let uuid = Uuid::parse_str(&id).unwrap();
    let movie = get_movie(db, uuid).await;

    match movie {
        Ok(movie) => Ok(Json(movie)),
        Err(err) => Err((
            Status::NotFound,
            format!("Failed to fetch movie: {:?}", err),
        )),
    }
}

#[get("/edit/<id>")]
async fn edit(db: Connection<Db>, id: &str) -> Template {
    let uuid = Uuid::parse_str(id).unwrap();
    let movie = get_movie(db, uuid).await.unwrap();

    Template::render("movies/edit", context! {movie: movie})
}

#[put("/<id>", data = "<form>")]
async fn update(
    mut db: Connection<Db>,
    id: &str,
    form: Form<FormMovie>,
    cookies: &CookieJar<'_>,
) -> Redirect {
    let uuid = Uuid::parse_str(id).unwrap();

    let result = sqlx::query!(
        "UPDATE movies
        SET name = $1, director = $2
        WHERE id = $3",
        form.name,
        form.director,
        uuid
    )
    .execute(&mut **db)
    .await;

    match result {
        Ok(_) => cookies.add(("notice", "Movie edited successfully")),
        Err(err) => cookies.add(("notice", format!("Movie couldn't get updated: {}", err))),
    }

    Redirect::to(uri!("/movies"))
}

#[delete("/<id>")]
async fn delete(mut db: Connection<Db>, id: String) -> Template {
    let uuid = Uuid::parse_str(&id).unwrap();
    let result = sqlx::query!("DELETE FROM movies WHERE id = $1", uuid)
        .execute(&mut **db)
        .await;

    let movies = get_movies(db).await;

    match result {
        Ok(_) => Template::render(
            "movies/index",
            context! {movies: movies, notice: "Movie deleted successfully"},
        ),
        Err(err) => Template::render(
            "movies/index",
            context! {movies: movies, notice: format!("Failed to delete movie: {}", err)},
        ),
    }
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Movies Stage", |rocket| async {
        rocket.mount(
            "/movies",
            routes![index, new, create, show, edit, update, delete],
        )
    })
}
