use bcrypt::{hash, verify, DEFAULT_COST};
use rocket::{fairing::AdHoc, form::Form, get, http::CookieJar, response::Redirect, routes};
use rocket_db_pools::Connection;
use rocket_dyn_templates::{context, Template};
use serde::{Deserialize, Serialize};
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

#[get("/sign_up")]
async fn sign_up() -> Template {
    Template::render("users/sign_up", context! {})
}

#[put("/sign_up", data = "<form>")]
async fn register(mut db: Connection<Db>, form: Form<UserRequest>) -> Redirect {
    let id = Uuid::new_v4();
    let hashed_password = hash(&form.password, DEFAULT_COST).unwrap();

    let query = r#"
        INSERT INTO users (id, email, password_hash)
        VALUES ($1, $2, $3)
        RETURNING id
    "#;

    let result = sqlx::query(query)
        .bind(id)
        .bind(&form.email)
        .bind(&hashed_password)
        .fetch_one(&mut **db)
        .await;

    match result {
        Ok(_) => Redirect::temporary(uri!("/users/login")),
        Err(err) => {
            println!("{}", err);
            Redirect::to(uri!("/users/sign_up"))
        }
    }
}

#[get("/sign_in")]
async fn sign_in() -> Template {
    Template::render("users/sign_in", context! {})
}

#[put("/sign_in", data = "<form>")]
async fn login(
    mut db: Connection<Db>,
    form: Form<UserRequest>,
    cookies: &CookieJar<'_>,
) -> Redirect {
    let user = sqlx::query_as!(User, "SELECT * FROM users WHERE email = $1", form.email,)
        .fetch_one(&mut **db)
        .await
        .unwrap();

    println!("Login with: {}", form.email);
    if let Ok(true) = verify(&form.password, &user.password_hash) {
        let token = generate_jwt(&user.email);
        cookies.add_private(("jwt", token));
        cookies.add(("notice", "User logged in correctly"));
        Redirect::to(uri!("/movies"))
    } else {
        cookies.add(("notice", "Invalid user"));
        Redirect::to(uri!("/users/sign_in"))
    }
}

// #[get("/log_out")]
// fn logout() {}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Users Stage", |rocket| async {
        rocket.mount("/users", routes![sign_up, sign_in, register, login])
    })
}
