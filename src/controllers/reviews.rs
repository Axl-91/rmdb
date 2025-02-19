use rocket::{fairing::AdHoc, form::Form};
use rocket_db_pools::Connection;
use rocket_dyn_templates::{context, Template};
use serde::{Deserialize, Serialize};
use sqlx::PgConnection;
use uuid::Uuid;

use crate::{middleware::auth_check::AuthUser, Db};

use super::users::get_user;

#[derive(Debug, Serialize, Deserialize)]
struct Review {
    score: i32,
    review: Option<String>,
}

#[derive(FromForm, Debug, Serialize, Deserialize)]
struct FormReview {
    user_id: String,
    movie_id: String,
    score: String,
    review: Option<String>,
}

// DB FUNCTIONS

async fn get_review(db: &mut PgConnection, id: Uuid) -> Review {
    sqlx::query_as!(
        Review,
        "SELECT score, review FROM reviews WHERE id = $1",
        id
    )
    .fetch_one(db)
    .await
    .unwrap()
}

// REQUEST FUNCTIONS

#[get("/new/<movie_id>")]
async fn new(mut db: Connection<Db>, movie_id: String, auth_user: AuthUser) -> Template {
    let pg_connection = &mut **db;
    let user = get_user(pg_connection, &auth_user.email).await.unwrap();

    Template::render(
        "reviews/new",
        context! {movie_id: movie_id, user_id: user.id, user_email: user.email},
    )
}

#[put("/new", data = "<form>")]
async fn create(form: Form<FormReview>) {
    println!("{}", form.user_id);
}

#[get("/edit/<id>")]
async fn edit(mut db: Connection<Db>, id: String) -> Template {
    let pg_connection = &mut **db;
    let uuid = Uuid::parse_str(&id).unwrap();

    let review = get_review(pg_connection, uuid).await;
    Template::render("reviews/edit", context! {review: review})
}

#[put("/edit/<id>")]
async fn update(id: String) {
    println!("{}", id);
}

#[delete("/delete/<id>")]
async fn delete(id: String) {
    println!("{}", id);
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Reviews Stage", |rocket| async {
        rocket.mount("/reviews", routes![new, create, edit, update, delete])
    })
}
