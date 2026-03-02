use rocket::async_trait;
use sea_orm::{
    sea_query::OnConflict, ActiveValue::Set, ColumnTrait, Condition, DatabaseConnection,
    DatabaseTransaction, DbErr, EntityTrait, QueryFilter,
};
use uuid::Uuid;

use crate::entity::inventory_item;

#[async_trait]
pub trait InventoryRepository: Send + Sync {
    async fn add_or_update(
        &self,
        user_id: Uuid,
        item_uuid: Uuid,
        definition_id: i32,
        state_blob: String,
    ) -> Result<(), DbErr>;

    /// Transactional variant for use inside an existing `DatabaseTransaction`.
    async fn add_or_update_tx(
        &self,
        tx: &DatabaseTransaction,
        user_id: Uuid,
        item_uuid: Uuid,
        definition_id: i32,
        state_blob: String,
    ) -> Result<(), DbErr>;

    async fn destroy(&self, user_id: Uuid, item_uid: Uuid) -> Result<(), DbErr>;
}

#[derive(Debug, Clone)]
pub struct InventoryRepositoryImpl {
    db: DatabaseConnection,
}

impl InventoryRepositoryImpl {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
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
    ) -> Result<(), DbErr> {
        inventory_item::Entity::insert(inventory_item::ActiveModel {
            user_id: Set(user_id),
            item_uuid: Set(item_uuid),
            definition_id: Set(definition_id),
            state_blob: Set(state_blob),
        })
        .on_conflict(
            OnConflict::column(inventory_item::Column::ItemUuid)
                .update_column(inventory_item::Column::StateBlob)
                .to_owned(),
        )
        .exec(&self.db)
        .await?;

        Ok(())
    }

    async fn add_or_update_tx(
        &self,
        tx: &DatabaseTransaction,
        user_id: Uuid,
        item_uuid: Uuid,
        definition_id: i32,
        state_blob: String,
    ) -> Result<(), DbErr> {
        inventory_item::Entity::insert(inventory_item::ActiveModel {
            user_id: Set(user_id),
            item_uuid: Set(item_uuid),
            definition_id: Set(definition_id),
            state_blob: Set(state_blob),
        })
        .on_conflict(
            OnConflict::column(inventory_item::Column::ItemUuid)
                .update_column(inventory_item::Column::StateBlob)
                .to_owned(),
        )
        .exec(tx)
        .await?;

        Ok(())
    }

    async fn destroy(&self, user_id: Uuid, item_uid: Uuid) -> Result<(), DbErr> {
        let result = inventory_item::Entity::delete_many()
            .filter(
                Condition::all()
                    .add(inventory_item::Column::UserId.eq(user_id))
                    .add(inventory_item::Column::ItemUuid.eq(item_uid))
                    .to_owned(),
            )
            .exec(&self.db)
            .await?;

        if result.rows_affected == 0 {
            return Err(DbErr::RecordNotUpdated);
        }

        Ok(())
    }
}
