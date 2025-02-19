use chrono::Utc;
use jsonwebtoken::{
    decode, encode, Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation,
};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,
    pub exp: usize,
}

pub fn generate_jwt(user_email: &str) -> String {
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET is required");

    // Set token to expire in 1 hour
    let expiration = Utc::now()
        .checked_add_signed(chrono::Duration::hours(1))
        .expect("Invalid timestamp")
        .timestamp();

    let claims = TokenClaims {
        sub: user_email.to_string(),
        exp: expiration as usize,
    };

    let encoding_key = EncodingKey::from_secret(jwt_secret.as_ref());

    encode(&Header::new(Algorithm::HS256), &claims, &encoding_key).expect("Error generating JWT")
}

pub fn decode_jwt(token: String) -> Result<TokenData<TokenClaims>, jsonwebtoken::errors::Error> {
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET is required");

    let decoding_key = DecodingKey::from_secret(jwt_secret.as_ref());
    let validation = Validation::new(Algorithm::HS256);

    decode::<TokenClaims>(&token, &decoding_key, &validation)
}
