use bcrypt::{hash, verify, DEFAULT_COST};
use rocket::{fairing::AdHoc, form::Form, get, http::CookieJar, response::Redirect, routes};
use rocket_db_pools::Connection;
use rocket_dyn_templates::{context, Template};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
use uuid::Uuid;

use crate::{auth::generate_jwt, Db};

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
    mut db: Connection<Db>,
    email: &str,
    password: &str,
) -> Result<PgRow, sqlx::Error> {
    let id = Uuid::new_v4();
    let hashed_password = hash(password, DEFAULT_COST).unwrap();

    let query = r#"
        INSERT INTO users (id, email, password_hash)
        VALUES ($1, $2, $3)
        RETURNING id
    "#;

    sqlx::query(query)
        .bind(id)
        .bind(email)
        .bind(&hashed_password)
        .fetch_one(&mut **db)
        .await
}

async fn get_user(mut db: Connection<Db>, email: &str) -> Result<User, sqlx::Error> {
    sqlx::query_as!(User, "SELECT * FROM users WHERE email = $1", email,)
        .fetch_one(&mut **db)
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
async fn register(db: Connection<Db>, form: Form<UserRequest>) -> Redirect {
    let result = create_user(db, &form.email, &form.password).await;

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
async fn login(db: Connection<Db>, form: Form<UserRequest>, cookies: &CookieJar<'_>) -> Redirect {
    if let Ok(user) = get_user(db, &form.email).await {
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
