use crate::domain::User;
use crate::entity::users;
use rocket::async_trait;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, DatabaseTransaction, DbErr, EntityTrait, QuerySelect};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use sqlx::types::Uuid;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn insert_new_user(&self, tx: &DatabaseTransaction, user: &User) -> Result<(), DbErr>;

    async fn from_uuid(
        &self,
        tx: &DatabaseTransaction,
        user_id: Uuid,
    ) -> Result<Option<User>, DbErr>;

    async fn get_username_from_email(
        &self,
        tx: &DatabaseTransaction,
        email: String,
    ) -> Result<Option<Username>, DbErr>;

    async fn from_username(
        &self,
        tx: &DatabaseTransaction,
        username: String,
    ) -> Result<Option<User>, DbErr>;

    // add more functions such as update or delete.
}

#[derive(Debug, Clone)]
pub struct UserRepositoryImpl;

impl UserRepositoryImpl {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, FromRow)]
pub struct Username {
    pub name: String,
}

#[async_trait]
impl UserRepository for UserRepositoryImpl {
    async fn insert_new_user(&self, tx: &DatabaseTransaction, user: &User) -> Result<(), DbErr> {
        users::ActiveModel {
            user_id: Set(user.user_id),
            name: Set(user.name.clone()),
            email: Set(user.email.clone()),
            password: Set(user.password.clone()),
            salt: Set(user.salt.clone()),
            created: Set(user.created.fixed_offset()),
        }
        .insert(tx)
        .await?;

        Ok(())
    }

    async fn from_uuid(
        &self,
        tx: &DatabaseTransaction,
        user_id: Uuid,
    ) -> Result<Option<User>, DbErr> {
        let model = users::Entity::find_by_id(user_id).one(tx).await?;

        Ok(model.map(|m| User {
            user_id: m.user_id,
            name: m.name,
            email: m.email,
            password: m.password,
            salt: m.salt,
            created: m.created.with_timezone(&chrono::Utc),
        }))
    }

    async fn get_username_from_email(
        &self,
        tx: &DatabaseTransaction,
        email: String,
    ) -> Result<Option<Username>, DbErr> {
        let model = users::Entity::find_by_email(&email)
            .select_only()
            .column(users::Column::Name)
            .into_tuple::<String>()
            .one(tx)
            .await?;

        Ok(model.map(|name| Username { name }))
    }

    async fn from_username(
        &self,
        tx: &DatabaseTransaction,
        username: String,
    ) -> Result<Option<User>, DbErr> {
        let model = users::Entity::find_by_name(&username).one(tx).await?;

        Ok(model.map(|m| User {
            user_id: m.user_id,
            name: m.name,
            email: m.email,
            password: m.password,
            salt: m.salt,
            created: m.created.with_timezone(&chrono::Utc),
        }))
    }
}
