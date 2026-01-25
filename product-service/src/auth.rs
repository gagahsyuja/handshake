use jsonwebtoken::{decode, DecodingKey, Validation};
use rocket::http::Status;
use rocket::request::{self, FromRequest, Request};
use rocket::outcome::Outcome;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32, // user id
    pub email: String,
    pub exp: usize,
}

pub struct AuthenticatedUser {
    pub user_id: i32,
    pub email: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthenticatedUser {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let token = request.headers().get_one("Authorization");
        
        match token {
            Some(token) => {
                let token = token.replace("Bearer ", "");
                
                // Validate JWT locally (same secret as auth service)
                let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret-key-change-in-production".to_string());
                
                match decode::<Claims>(
                    &token,
                    &DecodingKey::from_secret(secret.as_ref()),
                    &Validation::default(),
                ) {
                    Ok(token_data) => Outcome::Success(AuthenticatedUser {
                        user_id: token_data.claims.sub,
                        email: token_data.claims.email,
                    }),
                    Err(_) => Outcome::Error((Status::Unauthorized, ())),
                }
            }
            None => Outcome::Error((Status::Unauthorized, ())),
        }
    }
}
