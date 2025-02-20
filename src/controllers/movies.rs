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

use super::reviews::get_reviews_from_movie;

#[derive(Debug, Serialize, Deserialize)]
pub struct Movie {
    id: String,
    name: String,
    director: String,
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

pub async fn get_movie(db: &mut PgConnection, id: Uuid) -> Result<Movie, sqlx::Error> {
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
    form: Form<FormMovie>,
) -> Result<PgQueryResult, sqlx::Error> {
    sqlx::query!(
        "INSERT INTO movies(name, director, synopsis) VALUES ($1, $2, $3)",
        &form.name,
        &form.director,
        form.synopsis.as_deref()
    )
    .execute(db)
    .await
}

async fn update_movie(
    db: &mut PgConnection,
    id: Uuid,
    form: Form<FormMovie>,
) -> Result<PgQueryResult, sqlx::Error> {
    let now = chrono::Utc::now().naive_utc();

    sqlx::query!(
        "UPDATE movies
        SET name = $1, director = $2, synopsis=$3, updated_at = $4
        WHERE id = $5",
        &form.name,
        &form.director,
        form.synopsis.as_deref(),
        now,
        id
    )
    .execute(db)
    .await
}

async fn delete_movie(db: &mut PgConnection, id: Uuid) -> Result<PgQueryResult, sqlx::Error> {
    sqlx::query!("DELETE FROM movies WHERE id = $1", id)
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

    match create_movie(pg_connection, form).await {
        Ok(_) => cookies.add(("notice", "Movie created successfully")),
        Err(err) => cookies.add(("notice", format!("Failed to create movie: {:?}", err))),
    }

    Redirect::to(uri!("/movies"))
}

#[get("/<id>")]
async fn show(mut db: Connection<Db>, id: String, logged_user: LoggedUser) -> Template {
    let pg_connection = &mut **db;
    let uuid = Uuid::parse_str(&id).unwrap();

    let movie = get_movie(pg_connection, uuid).await.unwrap();
    let reviews = get_reviews_from_movie(pg_connection, uuid).await;
    let has_review = if let Some(user_email) = logged_user.email.clone() {
        reviews.clone().into_iter().any(|r| r.email == user_email)
    } else {
        false
    };

    Template::render(
        "movies/show",
        context! {movie: movie, reviews: reviews, has_review: has_review, user_email: logged_user.email},
    )
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
    let uuid = Uuid::parse_str(id).unwrap();

    match update_movie(pg_connection, uuid, form).await {
        Ok(_) => cookies.add(("notice", "Movie edited successfully")),
        Err(err) => cookies.add(("notice", format!("Failed to update movie: {:?}", err))),
    }

    Redirect::to(uri!("/movies"))
}

#[delete("/<id>")]
async fn delete(mut db: Connection<Db>, id: &str, cookies: &CookieJar<'_>) -> Redirect {
    let pg_connection = &mut **db;
    let uuid = Uuid::parse_str(id).unwrap();

    match delete_movie(pg_connection, uuid).await {
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
