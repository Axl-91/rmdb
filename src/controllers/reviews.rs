use rocket::{fairing::AdHoc, form::Form, http::CookieJar, response::Redirect};
use rocket_db_pools::Connection;
use rocket_dyn_templates::{context, Template};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgQueryResult, PgConnection};
use uuid::Uuid;

use crate::{middleware::auth_check::AuthUser, Db};

use super::{movies::get_movie, users::get_user};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserReview {
    id: String,
    pub email: String,
    pub score: i32,
    review: Option<String>,
}

#[derive(FromForm, Debug, Serialize, Deserialize)]
struct FormReview {
    user_id: String,
    movie_id: String,
    score: i32,
    review: Option<String>,
}

// DB FUNCTIONS

pub async fn get_reviews_from_movie(db: &mut PgConnection, id: Uuid) -> Vec<UserReview> {
    sqlx::query_as!(
        UserReview,
        "SELECT r.id, u.email, r.score, r.review FROM reviews r JOIN users u ON u.id = r.user_id WHERE r.movie_id = $1",
        id
    )
    .fetch_all(db)
    .await
    .unwrap()
}

async fn delete_review(db: &mut PgConnection, id: Uuid) -> Result<PgQueryResult, sqlx::Error> {
    sqlx::query!("DELETE FROM reviews WHERE id = $1", id)
        .execute(db)
        .await
}

async fn create_review(
    db: &mut PgConnection,
    form: Form<FormReview>,
) -> Result<PgQueryResult, sqlx::Error> {
    let uuid_movie = Uuid::parse_str(&form.movie_id).unwrap();
    let uuid_user = Uuid::parse_str(&form.user_id).unwrap();

    sqlx::query!(
        "INSERT INTO reviews(score, review, user_id, movie_id)
            VALUES ($1, $2, $3, $4)",
        form.score,
        form.review.as_deref(),
        uuid_user,
        uuid_movie
    )
    .execute(db)
    .await
}

// REQUEST FUNCTIONS

#[get("/new/<movie_id>")]
async fn new(mut db: Connection<Db>, movie_id: String, auth_user: AuthUser) -> Template {
    let pg_connection = &mut **db;
    let uuid = Uuid::parse_str(&movie_id).unwrap();

    let user = get_user(pg_connection, &auth_user.email).await.unwrap();
    let movie = get_movie(pg_connection, uuid).await.unwrap();

    Template::render(
        "reviews/new",
        context! {movie: movie, user_id: user.id, user_email: user.email},
    )
}

#[put("/new", data = "<form>")]
async fn create(
    mut db: Connection<Db>,
    form: Form<FormReview>,
    cookies: &CookieJar<'_>,
) -> Redirect {
    let pg_connection = &mut **db;

    match create_review(pg_connection, form).await {
        Ok(_) => cookies.add(("notice", "Review submitted successfully")),
        Err(err) => cookies.add(("notice", format!("Failed to submit review: {:?}", err))),
    }

    Redirect::to(uri!("/movies"))
}

// #[get("/edit/<id>")]
// async fn edit(mut db: Connection<Db>, id: String) -> Template {
//     let pg_connection = &mut **db;
//     let uuid = Uuid::parse_str(&id).unwrap();

//     let review = get_review(pg_connection, uuid).await;
//     Template::render("reviews/edit", context! {review: review})
// }

// #[put("/edit/<id>")]
// async fn update(id: String) {
//     println!("{}", id);
// }

#[delete("/delete/<id>")]
async fn delete(mut db: Connection<Db>, id: String, cookies: &CookieJar<'_>) -> Redirect {
    let pg_connection = &mut **db;
    let uuid = Uuid::parse_str(&id).unwrap();

    match delete_review(pg_connection, uuid).await {
        Ok(_) => cookies.add(("notice", "Review deleted successfully")),
        Err(err) => cookies.add(("notice", format!("Failed to delete review: {:?}", err))),
    }

    Redirect::to("/movies")
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Reviews Stage", |rocket| async {
        rocket.mount("/reviews", routes![new, create, delete])
    })
}
