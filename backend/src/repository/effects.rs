use crate::domain::ActiveEffect;
use chrono::{DateTime, Utc};
use rocket::async_trait;
use sqlx::{Error, PgPool};
use uuid::Uuid;

#[async_trait]
pub trait EffectsRepository: Send + Sync {
    async fn add_effect(
        &self,
        user_id: Uuid,
        item_id: i32,
        expiry_time: DateTime<Utc>,
    ) -> Result<(), sqlx::Error>;

    async fn remove_effect(&self, user_id: Uuid, item_id: i32) -> Result<(), sqlx::Error>;

    async fn get_active_effects(&self, user_id: Uuid) -> Result<Vec<ActiveEffect>, sqlx::Error>;

    async fn remove_all_expired_effects_global(&self) -> Result<(), sqlx::Error>;
}

#[derive(Debug, Clone)]
pub struct EffectsRepositoryImpl {
    pool: PgPool,
}

impl EffectsRepositoryImpl {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EffectsRepository for EffectsRepositoryImpl {
    async fn add_effect(
        &self,
        user_id: Uuid,
        item_id: i32,
        expiry_time: DateTime<Utc>,
    ) -> Result<(), sqlx::Error> {
        let result = sqlx::query!(
            "INSERT INTO player_effects (user_id, item_id, expiry_time)
            VALUES ($1, $2, $3)
            ON CONFLICT (user_id, item_id) 
            DO UPDATE SET expiry_time = EXCLUDED.expiry_time",
            user_id,
            item_id,
            expiry_time,
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(Error::RowNotFound);
        }

        Ok(())
    }

    async fn remove_effect(&self, user_id: Uuid, item_id: i32) -> Result<(), sqlx::Error> {
        let result = sqlx::query!(
            "DELETE FROM player_effects
            WHERE user_id = $1 AND item_id = $2",
            user_id,
            item_id,
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(Error::RowNotFound);
        }

        Ok(())
    }

    async fn get_active_effects(&self, user_id: Uuid) -> Result<Vec<ActiveEffect>, sqlx::Error> {
        let effects = sqlx::query_as!(
            ActiveEffect,
            "SELECT item_id, expiry_time
            FROM player_effects
            WHERE user_id = $1 AND expiry_time > NOW()
            ORDER BY expiry_time ASC",
            user_id,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(effects)
    }

    async fn remove_all_expired_effects_global(&self) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "DELETE FROM player_effects
            WHERE expiry_time <= NOW()",
        )
        .execute(&self.pool)
        .await
        .map(|_| ())
        .inspect_err(|e| {
            dbg!(e);
        })
    }
}
