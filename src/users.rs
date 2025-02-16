use bcrypt::{hash, DEFAULT_COST};
use rocket::{fairing::AdHoc, form::Form, get, routes};
use rocket_db_pools::Connection;
use rocket_dyn_templates::{context, Template};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::Db;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub password_hash: String,
}

#[derive(FromForm)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

#[get("/sign_up")]
async fn sign_up() -> Template {
    Template::render("users/sign_up", context! {})
}

#[post("/sign_up", data = "<form>")]
async fn register(mut db: Connection<Db>, form: Form<RegisterRequest>) -> Result<String, String> {
    let id = Uuid::new_v4();
    let hashed_password = match hash(&form.password, DEFAULT_COST) {
        Ok(hashed) => hashed,
        Err(_) => return Err("Error hashing password".into()),
    };

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
        Ok(_) => Ok("User registered successfully".into()),
        Err(err) => Err(format!("Error: {}", err)),
    }
}

// #[post("/login", data = "<form>")]
// async fn login(form: Json<User>) {}

// #[get("/log_out")]
// fn logout() {}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Users Stage", |rocket| async {
        rocket.mount("/users", routes![sign_up, register])
    })
}
