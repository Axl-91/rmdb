use rocket::{fairing::AdHoc, form::Form, http::CookieJar, response::Redirect};
use rocket_db_pools::{sqlx, Connection};
use rocket_dyn_templates::{context, Template};
use serde::{Deserialize, Serialize};
use sqlx::{
    postgres::PgQueryResult,
    types::{chrono, Uuid},
};

use crate::{middleware::AuthenticatedUser, Db};

#[derive(Debug, Serialize, Deserialize)]
struct Movie {
    id: String,
    name: Option<String>,
    director: Option<String>,
}

#[derive(FromForm, Debug, Serialize, Deserialize)]
struct FormMovie {
    name: String,
    director: String,
}

// DB FUNCTIONS

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
    let now = chrono::Utc::now().naive_utc();

    sqlx::query!(
        "INSERT INTO movies(id, name, director, created_at, updated_at) VALUES ($1, $2, $3, $4, $5)",
        id,
        name,
        director,
        now,
        now
    )
    .execute(&mut **db)
    .await
}

async fn update_movie(
    mut db: Connection<Db>,
    id: &str,
    name: &str,
    director: &str,
) -> Result<PgQueryResult, sqlx::Error> {
    let uuid = Uuid::parse_str(id).unwrap();
    let now = chrono::Utc::now().naive_utc();

    sqlx::query!(
        "UPDATE movies
        SET name = $1, director = $2, updated_at = $3
        WHERE id = $4",
        name,
        director,
        now,
        uuid
    )
    .execute(&mut **db)
    .await
}

async fn delete_movie(mut db: Connection<Db>, id: &str) -> Result<PgQueryResult, sqlx::Error> {
    let uuid = Uuid::parse_str(id).unwrap();

    sqlx::query!("DELETE FROM movies WHERE id = $1", uuid)
        .execute(&mut **db)
        .await
}

// REQUEST FUNCTIONS

#[get("/")]
async fn index(
    db: Connection<Db>,
    cookies: &CookieJar<'_>,
    auth_user: AuthenticatedUser,
) -> Template {
    let movies = get_movies(db).await;

    // I'll check for notice (alerts) to show, as create/edit/delete all redirect to index with a notice
    let notice = cookies.get("notice").map(|n| n.value().to_string());
    // After I store the notice to show I'll delete it from the cookies so it shows only one time
    cookies.remove("notice");

    Template::render(
        "movies/index",
        context! { movies: movies, notice: notice, user_email: auth_user.email},
    )
}

#[get("/new")]
async fn new(auth_user: AuthenticatedUser) -> Template {
    Template::render("movies/new", context! {user_email: auth_user.email})
}

#[put("/create", data = "<form>")]
async fn create(db: Connection<Db>, form: Form<FormMovie>, cookies: &CookieJar<'_>) -> Redirect {
    let result = create_movie(db, &form.name, &form.director).await;

    match result {
        Ok(_) => cookies.add(("notice", "Movie created successfully")),
        Err(err) => cookies.add(("notice", format!("Failed to create movie: {:?}", err))),
    }

    Redirect::to(uri!("/movies"))
}

#[get("/<id>")]
async fn show(db: Connection<Db>, id: String) -> Template {
    let uuid = Uuid::parse_str(&id).unwrap();
    let movie = get_movie(db, uuid).await.unwrap();

    Template::render("movies/show", context! {movie: movie})
}

#[get("/edit/<id>")]
async fn edit(db: Connection<Db>, id: &str, auth_user: AuthenticatedUser) -> Template {
    let uuid = Uuid::parse_str(id).unwrap();
    let movie = get_movie(db, uuid).await.unwrap();

    Template::render(
        "movies/edit",
        context! {movie: movie, user_email: auth_user.email},
    )
}

#[put("/<id>", data = "<form>")]
async fn update(
    db: Connection<Db>,
    id: &str,
    form: Form<FormMovie>,
    cookies: &CookieJar<'_>,
) -> Redirect {
    match update_movie(db, id, &form.name, &form.director).await {
        Ok(_) => cookies.add(("notice", "Movie edited successfully")),
        Err(err) => cookies.add(("notice", format!("Failed to update movie: {:?}", err))),
    }

    Redirect::to(uri!("/movies"))
}

#[delete("/<id>")]
async fn delete(db: Connection<Db>, id: &str, cookies: &CookieJar<'_>) -> Redirect {
    match delete_movie(db, id).await {
        Ok(_) => cookies.add(("notice", "Movie deleted successfully")),
        Err(err) => cookies.add(("notice", format!("Failed to delete movie: {:?}", err))),
    }

    Redirect::to("/movies")
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Movies Stage", |rocket| async {
        rocket.mount(
            "/movies",
            routes![index, new, create, show, edit, update, delete],
        )
    })
}
