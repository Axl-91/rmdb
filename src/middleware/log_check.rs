use std::env;

use jsonwebtoken::{decode, Algorithm};
use jsonwebtoken::{DecodingKey, Validation};
use rocket::outcome::Outcome;
use rocket::request::FromRequest;
use serde::Serialize;

use crate::auth::jwt::TokenClaims;

#[derive(Serialize)]
pub struct LoggedUser {
    pub email: Option<String>,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for LoggedUser {
    type Error = ();

    async fn from_request(
        req: &'r rocket::Request<'_>,
    ) -> rocket::request::Outcome<Self, Self::Error> {
        if let Some(token) = req
            .cookies()
            .get_private("jwt")
            .map(|n| n.value().to_string())
        {
            let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET is required");

            let decoding_key = DecodingKey::from_secret(jwt_secret.as_ref());
            let validation = Validation::new(Algorithm::HS256);

            match decode::<TokenClaims>(&token, &decoding_key, &validation) {
                Ok(token_data) => Outcome::Success(LoggedUser {
                    email: Some(token_data.claims.sub),
                }),
                Err(_) => Outcome::Success(LoggedUser { email: None }),
            }
        } else {
            Outcome::Success(LoggedUser { email: None })
        }
    }
}
