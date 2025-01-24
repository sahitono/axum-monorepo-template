use chrono::TimeDelta;
use serde::{Deserialize, Serialize};
use validator::Validate;

// Define a structure for holding sign-in data
#[derive(Clone, Serialize, Deserialize, Debug, Validate, Default)]
pub struct SignInPayload {
    #[validate(required, length(min = 1), email(message = "email is invalid"))]
    pub username: Option<String>,
    #[validate(required, length(min = 6))]
    pub password: Option<String>,
}

#[derive(Serialize, Deserialize)]
// Define a structure for holding claims data used in JWT tokens
pub struct Claims {
    pub exp: usize,  // Expiry time of the token
    pub iat: usize,  // Issued at time of the token
    pub sub: String,  // user id
}

#[derive(Serialize)]
pub struct OAuth2Response {
    pub token_type: String,
    pub access_token: String,
    pub expires_in: TimeDelta,
}

impl OAuth2Response {
    pub fn new_bearer(token: String, expires_in: TimeDelta) -> Self {
        Self {
            token_type: String::from("Bearer"),
            access_token: token,
            expires_in
        }
    }
}
