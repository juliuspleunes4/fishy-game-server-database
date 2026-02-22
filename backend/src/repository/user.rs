use crate::domain::User;
use crate::entity::{inventory_item, stats, users};
use rocket::async_trait;
use sea_orm::ActiveValue::Set;
use sea_orm::{
    ActiveModelTrait, DatabaseConnection, DatabaseTransaction, DbErr, EntityTrait, QuerySelect,
    TransactionError, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use sqlx::types::Uuid;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create_new_user(&self, user: User) -> Result<(), DbErr>;

    async fn from_uuid(&self, user_id: Uuid) -> Result<Option<User>, sqlx::Error>;

    async fn get_username_from_email(&self, email: String)
        -> Result<Option<Username>, sqlx::Error>;

    async fn from_username(&self, email: String) -> Result<Option<User>, sqlx::Error>;

    // add more functions such as update or delete.
}

#[derive(Debug, Clone)]
pub struct UserRepositoryImpl {
    db: DatabaseConnection,
}

impl UserRepositoryImpl {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    async fn insert_new_user(tx: &DatabaseTransaction, user: &User) -> Result<(), DbErr> {
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

    async fn insert_default_stats(tx: &DatabaseTransaction, user_id: &Uuid) -> Result<(), DbErr> {
        stats::ActiveModel {
            user_id: Set(*user_id),
            xp: Set(0),
            coins: Set(25),
            bucks: Set(5000),
            total_playtime: Set(0),
            ..Default::default()
        }
        .insert(tx)
        .await?;

        Ok(())
    }

    async fn insert_default_inventory(
        tx: &DatabaseTransaction,
        user_id: &Uuid,
    ) -> Result<(), DbErr> {
        let default_items = [
            (1000, String::from("AQABAAX2////")), // bamboo rod
            (0, String::from("AQABAAX2////")),    // hook
        ];

        for (definition_id, state_blob) in default_items {
            inventory_item::ActiveModel {
                user_id: Set(*user_id),
                item_uuid: Set(Uuid::new_v4()),
                definition_id: Set(definition_id),
                state_blob: Set(state_blob),
                ..Default::default()
            }
            .insert(tx)
            .await?;
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, FromRow)]
pub struct Username {
    pub name: String,
}

#[async_trait]
impl UserRepository for UserRepositoryImpl {
    async fn create_new_user(&self, user: User) -> Result<(), DbErr> {
        self.db
            .transaction::<_, (), DbErr>(|tx| {
                Box::pin(async move {
                    Self::insert_new_user(tx, &user).await?;
                    Self::insert_default_stats(tx, &user.user_id).await?;
                    Self::insert_default_inventory(tx, &user.user_id).await?;
                    Ok(())
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Connection(e) => e,
                TransactionError::Transaction(e) => e,
            })
    }

    async fn from_uuid(&self, user_id: Uuid) -> Result<Option<User>, sqlx::Error> {
        let model = users::Entity::find_by_id(user_id)
            .one(&self.db)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

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
        email: String,
    ) -> Result<Option<Username>, sqlx::Error> {
        let model = users::Entity::find_by_email(&email)
            .select_only()
            .column(users::Column::Name)
            .into_tuple::<String>()
            .one(&self.db)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        Ok(model.map(|name| Username { name }))
    }

    async fn from_username(&self, username: String) -> Result<Option<User>, sqlx::Error> {
        let model = users::Entity::find_by_name(&username)
            .one(&self.db)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

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
