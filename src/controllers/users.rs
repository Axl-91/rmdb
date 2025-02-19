use bcrypt::{hash, verify, DEFAULT_COST};
use rocket::{fairing::AdHoc, form::Form, get, http::CookieJar, response::Redirect, routes};
use rocket_db_pools::Connection;
use rocket_dyn_templates::{context, Template};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, PgConnection};

use crate::{auth::jwt::generate_jwt, Db};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub password_hash: String,
}

#[derive(FromForm)]
pub struct UserRequest {
    pub email: String,
    pub password: String,
}

// DB FUNCTIONS

async fn create_user(
    db: &mut PgConnection,
    email: &str,
    password: &str,
) -> Result<PgRow, sqlx::Error> {
    let hashed_password = hash(password, DEFAULT_COST).unwrap();

    let query = r#"
        INSERT INTO users (email, password_hash)
        VALUES ($1, $2)
        RETURNING id
    "#;

    sqlx::query(query)
        .bind(email)
        .bind(&hashed_password)
        .fetch_one(db)
        .await
}

pub async fn get_user(db: &mut PgConnection, email: &str) -> Result<User, sqlx::Error> {
    sqlx::query_as!(
        User,
        "SELECT id, email, password_hash FROM users WHERE email = $1",
        email,
    )
    .fetch_one(db)
    .await
}

// REQUESTS FUNCTIONS

#[get("/sign_up")]
async fn sign_up(cookies: &CookieJar<'_>) -> Template {
    let notice = cookies.get("notice").map(|n| n.value().to_string());
    cookies.remove("notice");

    Template::render("users/sign_up", context! {notice: notice})
}

#[put("/sign_up", data = "<form>")]
async fn register(mut db: Connection<Db>, form: Form<UserRequest>) -> Redirect {
    let pg_connection = &mut **db;
    let result = create_user(pg_connection, &form.email, &form.password).await;

    match result {
        Ok(_) => Redirect::temporary(uri!("/users/sign_in")),
        Err(err) => {
            println!("{}", err);
            Redirect::to(uri!("/users/sign_up"))
        }
    }
}

#[get("/sign_in")]
async fn sign_in(cookies: &CookieJar<'_>) -> Template {
    let notice = cookies.get("notice").map(|n| n.value().to_string());
    cookies.remove("notice");

    Template::render("users/sign_in", context! {notice: notice})
}

#[put("/sign_in", data = "<form>")]
async fn login(
    mut db: Connection<Db>,
    form: Form<UserRequest>,
    cookies: &CookieJar<'_>,
) -> Redirect {
    let pg_connection = &mut **db;

    if let Ok(user) = get_user(pg_connection, &form.email).await {
        if let Ok(true) = verify(&form.password, &user.password_hash) {
            let token = generate_jwt(&user.email);
            cookies.add_private(("jwt", token));
            cookies.add(("notice", "User logged in correctly"));
            Redirect::to(uri!("/movies"))
        } else {
            cookies.add(("notice", "Password is incorrect"));
            Redirect::to(uri!("/users/sign_in"))
        }
    } else {
        cookies.add(("notice", "Invalid user"));
        Redirect::to(uri!("/users/sign_in"))
    }
}

#[delete("/logout")]
fn logout(cookies: &CookieJar<'_>) -> Redirect {
    cookies.remove_private("jwt");
    Redirect::to(uri!("/"))
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Users Stage", |rocket| async {
        rocket.mount("/users", routes![sign_up, sign_in, register, login, logout])
    })
}
