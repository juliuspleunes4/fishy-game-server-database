use rocket::async_trait;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, Condition, DatabaseTransaction, DbErr, EntityTrait, QueryFilter, sea_query::OnConflict
};
use uuid::Uuid;

use crate::entity::inventory_item;

#[async_trait]
pub trait InventoryRepository: Send + Sync {
    async fn add_or_update_tx(
        &self,
        tx: &DatabaseTransaction,
        user_id: Uuid,
        item_uuid: Uuid,
        definition_id: i32,
        state_blob: String,
    ) -> Result<(), DbErr>;

    async fn destroy(
        &self,
        tx: &DatabaseTransaction,
        user_id: Uuid,
        item_uid: Uuid,
    ) -> Result<(), DbErr>;

    async fn insert_new_inventory(&self, tx: &DatabaseTransaction, user_id: Uuid, rod_id: i32, rod_state: String, bait_id: i32, bait_state: String) -> Result<(), DbErr>;
}

#[derive(Debug, Clone)]
pub struct InventoryRepositoryImpl;

impl InventoryRepositoryImpl {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl InventoryRepository for InventoryRepositoryImpl {
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

    async fn destroy(
        &self,
        tx: &DatabaseTransaction,
        user_id: Uuid,
        item_uid: Uuid,
    ) -> Result<(), DbErr> {
        let result = inventory_item::Entity::delete_many()
            .filter(
                Condition::all()
                    .add(inventory_item::Column::UserId.eq(user_id))
                    .add(inventory_item::Column::ItemUuid.eq(item_uid))
                    .to_owned(),
            )
            .exec(tx)
            .await?;

        if result.rows_affected == 0 {
            return Err(DbErr::RecordNotUpdated);
        }

        Ok(())
    }

    async fn insert_new_inventory(&self, tx: &DatabaseTransaction, user_id: Uuid, rod_id: i32, rod_state: String, bait_id: i32, bait_state: String) -> Result<(), DbErr> {
        let default_items = [
            (rod_id, rod_state),
            (bait_id, bait_state),
        ];

        for (definition_id, state_blob) in default_items {
            inventory_item::ActiveModel {
                user_id: Set(user_id),
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
