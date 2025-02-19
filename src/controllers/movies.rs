use rocket::{fairing::AdHoc, form::Form, http::CookieJar, response::Redirect};
use rocket_db_pools::{sqlx, Connection};
use rocket_dyn_templates::{context, Template};
use serde::{Deserialize, Serialize};
use sqlx::{
    postgres::PgQueryResult,
    types::{chrono, Uuid},
    PgConnection,
};

use crate::{middleware::log_check::LoggedUser, Db};

#[derive(Debug, Serialize, Deserialize)]
struct Movie {
    id: String,
    name: Option<String>,
    director: Option<String>,
    synopsis: Option<String>,
}

#[derive(FromForm, Debug, Serialize, Deserialize)]
struct FormMovie {
    name: String,
    director: String,
    synopsis: Option<String>,
}

// DB FUNCTIONS

async fn get_movies(db: &mut PgConnection) -> Vec<Movie> {
    sqlx::query_as!(
        Movie,
        "SELECT id, name, director, synopsis FROM movies ORDER BY name"
    )
    .fetch_all(db)
    .await
    .unwrap()
}

async fn get_movie(db: &mut PgConnection, id: Uuid) -> Result<Movie, sqlx::Error> {
    sqlx::query_as!(
        Movie,
        "SELECT id, name, director, synopsis FROM movies WHERE id = $1",
        id
    )
    .fetch_one(db)
    .await
}

async fn create_movie(
    db: &mut PgConnection,
    name: &str,
    director: &str,
    synopsis: &Option<String>,
) -> Result<PgQueryResult, sqlx::Error> {
    sqlx::query!(
        "INSERT INTO movies(name, director, synopsis) VALUES ($1, $2, $3)",
        name,
        director,
        synopsis.as_deref()
    )
    .execute(db)
    .await
}

async fn update_movie(
    db: &mut PgConnection,
    id: &str,
    name: &str,
    director: &str,
    synopsis: &Option<String>,
) -> Result<PgQueryResult, sqlx::Error> {
    let uuid = Uuid::parse_str(id).unwrap();
    let now = chrono::Utc::now().naive_utc();

    sqlx::query!(
        "UPDATE movies
        SET name = $1, director = $2, synopsis=$3, updated_at = $4
        WHERE id = $5",
        name,
        director,
        synopsis.as_deref(),
        now,
        uuid
    )
    .execute(db)
    .await
}

async fn delete_movie(db: &mut PgConnection, id: &str) -> Result<PgQueryResult, sqlx::Error> {
    let uuid = Uuid::parse_str(id).unwrap();

    sqlx::query!("DELETE FROM movies WHERE id = $1", uuid)
        .execute(db)
        .await
}

// REQUEST FUNCTIONS

#[get("/")]
async fn index(
    mut db: Connection<Db>,
    cookies: &CookieJar<'_>,
    logged_user: LoggedUser,
) -> Template {
    let pg_connection = &mut **db;
    let movies = get_movies(pg_connection).await;

    // I'll check for notice (alerts) to show, as create/edit/delete all redirect to index with a notice
    let notice = cookies.get("notice").map(|n| n.value().to_string());
    // After I store the notice to show I'll delete it from the cookies so it shows only one time
    cookies.remove("notice");

    Template::render(
        "movies/index",
        context! { movies: movies, notice: notice, user_email: logged_user.email},
    )
}

#[get("/new")]
async fn new(logged_user: LoggedUser) -> Template {
    Template::render("movies/new", context! {user_email: logged_user.email})
}

#[put("/create", data = "<form>")]
async fn create(
    mut db: Connection<Db>,
    form: Form<FormMovie>,
    cookies: &CookieJar<'_>,
) -> Redirect {
    let pg_connection = &mut **db;
    let result = create_movie(pg_connection, &form.name, &form.director, &form.synopsis).await;

    match result {
        Ok(_) => cookies.add(("notice", "Movie created successfully")),
        Err(err) => cookies.add(("notice", format!("Failed to create movie: {:?}", err))),
    }

    Redirect::to(uri!("/movies"))
}

#[get("/<id>")]
async fn show(mut db: Connection<Db>, id: String) -> Template {
    let pg_connection = &mut **db;
    let uuid = Uuid::parse_str(&id).unwrap();

    let movie = get_movie(pg_connection, uuid).await.unwrap();

    Template::render("movies/show", context! {movie: movie})
}

#[get("/edit/<id>")]
async fn edit(mut db: Connection<Db>, id: &str, logged_user: LoggedUser) -> Template {
    let pg_connection = &mut **db;
    let uuid = Uuid::parse_str(id).unwrap();

    let movie = get_movie(pg_connection, uuid).await.unwrap();

    Template::render(
        "movies/edit",
        context! {movie: movie, user_email: logged_user.email},
    )
}

#[put("/<id>", data = "<form>")]
async fn update(
    mut db: Connection<Db>,
    id: &str,
    form: Form<FormMovie>,
    cookies: &CookieJar<'_>,
) -> Redirect {
    let pg_connection = &mut **db;

    match update_movie(
        pg_connection,
        id,
        &form.name,
        &form.director,
        &form.synopsis,
    )
    .await
    {
        Ok(_) => cookies.add(("notice", "Movie edited successfully")),
        Err(err) => cookies.add(("notice", format!("Failed to update movie: {:?}", err))),
    }

    Redirect::to(uri!("/movies"))
}

#[delete("/<id>")]
async fn delete(mut db: Connection<Db>, id: &str, cookies: &CookieJar<'_>) -> Redirect {
    let pg_connection = &mut **db;

    match delete_movie(pg_connection, id).await {
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
