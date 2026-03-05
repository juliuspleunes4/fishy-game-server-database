use crate::domain::{LoginResponse, User};
use crate::repository::inventory::InventoryRepository;
use crate::repository::stats::StatsRepository;
use crate::repository::user::{UserRepository, Username};
use crate::utils::jwt::generate_jwt;
use bcrypt::hash;
use chrono::Utc;
use rocket::async_trait;
use sea_orm::{DatabaseConnection, DbErr, TransactionError, TransactionTrait};
use uuid::Uuid;

// Here you add your business logic here.
#[async_trait]
pub trait UserService: Send + Sync {
    async fn create(
        &self,
        name: String,
        email: String,
        password: String,
    ) -> Result<LoginResponse, DbErr>;

    async fn retreive_username(&self, email: String) -> Result<bool, DbErr>;

    async fn change_password(
        &self,
        name: String,
        new_password: String,
    ) -> Result<bool, DbErr>;

    async fn from_uuid(&self, user_id: Uuid) -> Result<Option<User>, DbErr>;
}

pub struct UserServiceImpl<U: UserRepository, S: StatsRepository, I: InventoryRepository> {
    db: DatabaseConnection,
    user_repository: U,
    stats_repository: S,
    inventory_repository: I,
    secret_key: String,
}

impl<U: UserRepository, S: StatsRepository, I: InventoryRepository> UserServiceImpl<U, S, I> {
    // create a new function for UserServiceImpl.
    pub fn new(
        db: DatabaseConnection,
        user_repository: U,
        stats_repository: S,
        inventory_repository: I,
        secret_key: String,
    ) -> Self {
        Self {
            db,
            user_repository,
            stats_repository,
            inventory_repository,
            secret_key,
        }
    }
}

// Implement UserService trait for UserServiceImpl.
#[async_trait]
impl<
        U: UserRepository + Clone + 'static,
        S: StatsRepository + Clone + 'static,
        I: InventoryRepository + Clone + 'static,
    > UserService for UserServiceImpl<U, S, I>
{
    async fn create(
        &self,
        name: String,
        email: String,
        password: String,
    ) -> Result<LoginResponse, DbErr> {
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

        let user_repo = self.user_repository.clone();
        let stats_repo = self.stats_repository.clone();
        let inventory_repo = self.inventory_repository.clone();
        let secret_key = self.secret_key.clone();

        self.db
            .transaction::<_, LoginResponse, DbErr>(|tx| {
                Box::pin(async move {
                    user_repo.insert_new_user(tx, &user).await?;
                    stats_repo.insert_new_stats(tx, user.user_id, 25, 5000).await?;
                    inventory_repo.insert_new_inventory(tx, user.user_id, 1000, String::from("AQABAAX2////"), 0, String::from("AQABAAX2////")).await?;
                    Ok(LoginResponse {
                        code: 200,
                        jwt: generate_jwt(user_id, &secret_key).map_err(|e| DbErr::Custom(e))?,
                    })
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Connection(e) => e,
                TransactionError::Transaction(e) => e,
            })
    }

    async fn retreive_username(&self, email: String) -> Result<bool, DbErr> {
        let user_repo = self.user_repository.clone();
        let email_cloned = email.clone();

        let result = self
            .db
            .transaction::<_, Option<Username>, DbErr>(move |tx| {
                Box::pin(async move { user_repo.get_username_from_email(tx, email_cloned).await })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Connection(e) => e,
                TransactionError::Transaction(e) => e,
            })?;

        Ok(result.is_some())
    }

    async fn change_password(
        &self,
        _name: String,
        _new_password: String,
    ) -> Result<bool, DbErr> {
        Ok(false)
    }

    async fn from_uuid(&self, user_id: Uuid) -> Result<Option<User>, DbErr> {
        let user_repo = self.user_repository.clone();

        self.db
            .transaction::<_, Option<User>, DbErr>(move |tx| {
                Box::pin(async move { user_repo.from_uuid(tx, user_id).await })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Connection(e) => e,
                TransactionError::Transaction(e) => e,
            })
    }
}

/// Hashes a password with bcrypt together with a salt.
pub fn hash_password(password: &str, salt: &str) -> String {
    // Generate a hashed password
    hash(format!("{}{}", password, salt), bcrypt::DEFAULT_COST).expect("Failed to hash password")
}
