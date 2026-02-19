use crate::domain::{LoginResponse, User};
use crate::repository::user::*;
use crate::utils::jwt::generate_jwt;
use bcrypt::hash;
use chrono::Utc;
use rocket::async_trait;
use uuid::Uuid;

// Here you add your business logic here.
#[async_trait]
pub trait UserService: Send + Sync {
    async fn create(
        &self,
        name: String,
        email: String,
        password: String,
    ) -> Result<LoginResponse, sqlx::Error>;

    async fn retreive_username(&self, email: String) -> Result<bool, sqlx::Error>;

    async fn change_password(
        &self,
        name: String,
        new_password: String,
    ) -> Result<bool, sqlx::Error>;

    async fn from_uuid(&self, user_id: Uuid) -> Result<Option<User>, sqlx::Error>;
}

pub struct UserServiceImpl<T: UserRepository> {
    user_repository: T,
    secret_key: String,
}

impl<R: UserRepository> UserServiceImpl<R> {
    // create a new function for UserServiceImpl.
    pub fn new(user_repository: R, secret_key: String) -> Self {
        Self {
            user_repository,
            secret_key,
        }
    }
}

// Implement UserService trait for UserServiceImpl.
#[async_trait]
impl<R: UserRepository> UserService for UserServiceImpl<R> {
    async fn create(
        &self,
        name: String,
        email: String,
        password: String,
    ) -> Result<LoginResponse, sqlx::Error> {
        let salt = Uuid::new_v4().to_string();
        let user_id = Uuid::new_v4();
        let user = User {
            user_id,
            salt: salt.clone(),
            name,
            email,
            password: hash_password(&password, &salt),
            created: Utc::now(),
        };

        match self.user_repository.create_new_user(user).await {
            Ok(_) => Ok(LoginResponse {
                code: 200,
                jwt: generate_jwt(user_id, &self.secret_key)?,
            }),
            Err(e) => {
                dbg!(&e);
                return Err(sqlx::Error::BeginFailed);
            }
        }
    }

    async fn retreive_username(&self, email: String) -> Result<bool, sqlx::Error> {
        match self.user_repository.get_username_from_email(email).await {
            Ok(_) => Ok(true),
            Err(e) => {
                dbg!(&e);
                return Ok(false);
            }
        }
    }

    async fn change_password(
        &self,
        _name: String,
        _new_password: String,
    ) -> Result<bool, sqlx::Error> {
        Ok(false)
    }

    async fn from_uuid(&self, user_id: Uuid) -> Result<Option<User>, sqlx::Error> {
        // recieve the user from the database given a user_id.
        self.user_repository.from_uuid(user_id).await
    }
}

/// Hashes a password with bcrypt together with a salt.
pub fn hash_password(password: &str, salt: &str) -> String {
    // Generate a hashed password
    hash(format!("{}{}", password, salt), bcrypt::DEFAULT_COST).expect("Failed to hash password")
}
