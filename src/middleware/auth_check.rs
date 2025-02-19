use rocket::http::Status;
use rocket::outcome::Outcome;
use rocket::request::FromRequest;
use serde::Serialize;

use crate::auth::jwt::decode_jwt;

#[derive(Serialize)]
pub struct AuthUser {
    pub email: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthUser {
    type Error = ();

    async fn from_request(
        req: &'r rocket::Request<'_>,
    ) -> rocket::request::Outcome<Self, Self::Error> {
        let cookie = req.cookies().get_private("jwt");

        if let Some(token) = cookie.map(|n| n.value().to_string()) {
            match decode_jwt(token) {
                Ok(token_data) => Outcome::Success(AuthUser {
                    email: token_data.claims.sub,
                }),
                Err(_) => Outcome::Error((Status::Unauthorized, ())),
            }
        } else {
            Outcome::Error((Status::Unauthorized, ()))
        }
    }
}
