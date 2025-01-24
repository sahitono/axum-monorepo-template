use serde::{Deserialize, Serialize};
use validator::Validate;
use entity::user_account;
use crate::infrastructure::errors::AppResult;

#[derive(Clone, Serialize, Deserialize, Debug, Validate, Default)]
pub struct UserNewDto {
    // #[validate(required, length(min = 1))]
    // pub name: Option<String>,
    #[validate(required, length(min = 1), email(message = "email is invalid"))]
    pub username: Option<String>,
    #[validate(required, length(min = 6))]
    pub password: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UserReadResponse {
    // #[validate(required, length(min = 1))]
    // pub name: Option<String>,
    pub username: String,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
}

impl UserReadResponse {
    pub fn from_model(model: user_account::Model) -> AppResult<Self> {
        Ok(
            UserReadResponse {
                username: model.username,
                created_at: model.created_at,
            }
        )
    }
}