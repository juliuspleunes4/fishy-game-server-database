use rocket::async_trait;
use sqlx::{Error, PgPool};
use uuid::Uuid;

#[async_trait]
pub trait InventoryRepository: Send + Sync {
    async fn add_or_update(
        &self,
        user_id: Uuid,
        item_uuid: Uuid,
        definition_id: i32,
        state_blob: String,
    ) -> Result<(), sqlx::Error>;

    async fn destroy(&self, user_id: Uuid, item_uid: Uuid) -> Result<(), sqlx::Error>;
}

#[derive(Debug, Clone)]
pub struct InventoryRepositoryImpl {
    pool: PgPool,
}

impl InventoryRepositoryImpl {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl InventoryRepository for InventoryRepositoryImpl {
    async fn add_or_update(
        &self,
        user_id: Uuid,
        item_uuid: Uuid,
        definition_id: i32,
        state_blob: String,
    ) -> Result<(), sqlx::Error> {
        match sqlx::query!(
            "INSERT INTO inventory_item (user_id, item_uuid, definition_id, state_blob)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (item_uuid)
            DO UPDATE SET
                state_blob = EXCLUDED.state_blob",
            user_id,
            item_uuid,
            definition_id,
            state_blob,
        )
        .fetch_optional(&self.pool)
        .await
        {
            Ok(o) => o,
            Err(e) => {
                dbg!(&e);
                return Err(e);
            }
        };
        Ok(())
    }

    async fn destroy(&self, user_id: Uuid, item_uid: Uuid) -> Result<(), sqlx::Error> {
        let result = match sqlx::query!(
            "DELETE FROM inventory_item WHERE
            user_id = $1 AND item_uuid = $2",
            user_id,
            item_uid,
        )
        .execute(&self.pool)
        .await
        {
            Ok(o) => o,
            Err(e) => {
                dbg!(&e);
                return Err(e);
            }
        };

        if result.rows_affected() == 0 {
            return Err(Error::RowNotFound);
        }

        Ok(())
    }
}
