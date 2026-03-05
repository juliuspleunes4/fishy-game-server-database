use crate::domain::{LoginResponse, User};
use crate::repository::user::UserRepository;
use crate::utils::jwt::{generate_jwt, Claims};
use bcrypt::verify;
use jsonwebtoken::{decode, DecodingKey, Validation};
use rocket::async_trait;
use sea_orm::{DatabaseConnection, DbErr, TransactionError, TransactionTrait};
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
/// Database connection to access user data,
/// user repository for recieving information about users,
/// and the secret key for signing JWT's.
pub struct AuthenticationServiceImpl<U: UserRepository + Clone> {
    db: DatabaseConnection,
    user_repository: U,
    secret_key: String,
}

impl<U: UserRepository + Clone> AuthenticationServiceImpl<U> {
    pub fn new(db: DatabaseConnection, user_repository: U, secret_key: String) -> Self {
        Self {
            db,
            user_repository,
            secret_key,
        }
    }
}

// Implement the authentication service trait for AuthenticationServiceImpl.
#[async_trait]
impl<U: UserRepository + Clone + 'static> AuthenticationService for AuthenticationServiceImpl<U> {
    async fn login(
        &self,
        username: String,
        password: String,
    ) -> Result<Option<LoginResponse>, DbErr> {
        let user_repo = self.user_repository.clone();
        let username_cloned = username.clone();

        let user = self
            .db
            .transaction::<_, Option<User>, DbErr>(move |tx| {
                Box::pin(async move { user_repo.from_username(tx, username_cloned).await })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Connection(e) => e,
                TransactionError::Transaction(e) => e,
            })?;

        let user = match user {
            Some(user) => user,
            None => return Ok(None),
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
            Ok(claims) => {
                let user_repo = self.user_repository.clone();
                let user_id =
                    Uuid::from_str(&claims.user_id).expect("Failed to generate uuid.");

                let user = self
                    .db
                    .transaction::<_, Option<User>, DbErr>(move |tx| {
                        Box::pin(async move { user_repo.from_uuid(tx, user_id).await })
                    })
                    .await
                    .map_err(|e| match e {
                        TransactionError::Connection(e) => e,
                        TransactionError::Transaction(e) => e,
                    })?;

                Ok(user)
            }
            Err(_) => Ok(None),
        }
    }
}

pub fn verify_password(password: &str, salt: &str, hashed_password: &str) -> bool {
    // Verify the password against the hashed password
    let s = format!("{}{}", password, salt);
    verify(s, hashed_password).unwrap_or(false)
}
