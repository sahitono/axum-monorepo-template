use crate::dto::user::UserNewDto;
use crate::infrastructure::errors::{AppError, AppResult};
use crate::infrastructure::uuid::generate_uuid;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use entity::prelude::UserAccount;
use entity::user_account;
use sea_orm::ActiveValue::Set;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use std::sync::Arc;
use uuid::Uuid;

pub struct UserRepository {
    db: Arc<DatabaseConnection>,
}

impl UserRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> UserRepository {
        Self { db }
    }

    pub async fn get_by_id(&self, user_id: &str) -> AppResult<user_account::Model> {
        let users = UserAccount::find_by_id(Uuid::parse_str(user_id)?).one(&*self.db).await?;
        match users {
            Some(user) => Ok(user),
            None => Err(AppError::BadRequest("user not found".to_string())),
        }
    }

    pub async fn get_by_username(&self, username: &str) -> AppResult<user_account::Model> {
        let users = UserAccount::find()
            .filter(user_account::Column::Username.eq(username))
            .one(&*self.db)
            .await?;
        match users {
            Some(user) => Ok(user),
            None => Err(AppError::BadRequest("user not found".to_string())),
        }
    }

    pub async fn create(&self, dto: &UserNewDto) -> AppResult<String> {
        let id = generate_uuid();

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(dto.username.clone().unwrap().as_ref(), &salt)
            .unwrap()
            .to_string();

        let user = user_account::ActiveModel {
            id: Set(id),
            username: Set(dto.username.clone().unwrap()),
            password: Set(password_hash),
            ..Default::default()
        };
        let res = UserAccount::insert(user).exec(&*self.db).await?;
        Ok(res.last_insert_id.to_string())
    }
}
