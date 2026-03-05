use crate::domain::{LoginResponse, User};
use crate::repository::user::UserRepository;
use crate::utils::jwt::{generate_jwt, Claims};
use bcrypt::verify;
use jsonwebtoken::{decode, DecodingKey, Validation};
use rocket::async_trait;
use sea_orm::DbErr;
use std::str::FromStr;
use uuid::Uuid;

/// business logic for authorisation.
#[async_trait]
pub trait AuthenticationService: Send + Sync {
    /// Returns a JWT when credentials are valid.
    async fn login(
        &self,
        username: String,
        password: String,
    ) -> Result<Option<LoginResponse>, DbErr>;

    /// Checks if a JWT is valid given credentials.
    async fn verify_jwt(&self, token: &str) -> Result<Option<User>, DbErr>;
}

/// AuthentcationServiceImpl requires:
/// User repository for recieving information about users.
/// The secret key for signing JWT's.
pub struct AuthenticationServiceImpl<U: UserRepository> {
    user_repository: U,
    secret_key: String,
}

impl<U: UserRepository> AuthenticationServiceImpl<U> {
    pub fn new(user_repository: U, secret_key: String) -> Self {
        Self {
            user_repository,
            secret_key,
        }
    }
}

// Implement the authentication service trait for AuthenticationServiceImpl.
#[async_trait]
impl<U: UserRepository> AuthenticationService for AuthenticationServiceImpl<U> {
    async fn login(
        &self,
        username: String,
        password: String,
    ) -> Result<Option<LoginResponse>, DbErr> {
        let user = match self.user_repository.from_username(username).await.map_err(|e| DbErr::Custom(e.to_string()))? {
            Some(user) => user,
            None => {
                return Ok(None);
            }
        };
        match verify_password(&password, &user.salt, &user.password) {
            true => Ok(Some(LoginResponse {
                code: 200,
                jwt: generate_jwt(user.user_id, &self.secret_key).map_err(|e| DbErr::Custom(e))?,
            })),
            false => Ok(None),
        }
    }

    async fn verify_jwt(&self, token: &str) -> Result<Option<User>, DbErr> {
        let claims = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret_key.clone().as_bytes()),
            &Validation::default(),
        )
        .map(|data| data.claims);

        match claims {
            Ok(claims) => Ok(self
                .user_repository
                .from_uuid(Uuid::from_str(&claims.user_id).expect("Failed to generate uuid."))
                .await.map_err(|e| DbErr::Custom(e.to_string()))?),
            Err(_) => Ok(None),
        }
    }
}

pub fn verify_password(password: &str, salt: &str, hashed_password: &str) -> bool {
    // Verify the password against the hashed password
    let s = format!("{}{}", password, salt);
    verify(s, hashed_password).unwrap_or(false)
}
