use rocket::async_trait;
use sea_orm::{DatabaseConnection, DbErr, TransactionError, TransactionTrait};
use uuid::Uuid;

use crate::repository::inventory::InventoryRepository;

// Here you add your business logic here.
#[async_trait]
pub trait InventoryService: Send + Sync {
    async fn use_item(
        &self,
        user_uuid: Uuid,
        item_uuid: Uuid,
        item_def_id: i32,
        state_blob: String,
    ) -> Result<(), DbErr>;

    async fn destroy(&self, user_id: Uuid, item_uid: Uuid) -> Result<(), DbErr>;
}

pub struct InventoryServiceImpl<T: InventoryRepository> {
    db: DatabaseConnection,
    inventory_repository: T,
}

impl<R: InventoryRepository + Clone> InventoryServiceImpl<R> {
    // create a new function for InventoryServiceImpl.
    pub fn new(db: DatabaseConnection, inventory_repository: R) -> Self {
        Self {
            db,
            inventory_repository,
        }
    }
}

// Implement InventoryService trait for InventoryServiceImpl.
#[async_trait]
impl<R: InventoryRepository + Clone + 'static> InventoryService for InventoryServiceImpl<R> {
    async fn use_item(
        &self,
        user_uuid: Uuid,
        item_uuid: Uuid,
        item_def_id: i32,
        state_blob: String,
    ) -> Result<(), DbErr> {
        let inv_repo = self.inventory_repository.clone();

        self.db
            .transaction::<_, (), DbErr>(move |tx| {
                Box::pin(async move {
                    inv_repo
                        .add_or_update_tx(tx, user_uuid, item_uuid, item_def_id, state_blob)
                        .await?;

                    Ok(())
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Connection(e) => e,
                TransactionError::Transaction(e) => e,
            })
    }

    async fn destroy(&self, user_id: Uuid, item_uid: Uuid) -> Result<(), DbErr> {
        let inv_repo = self.inventory_repository.clone();

        self.db
            .transaction::<_, (), DbErr>(move |tx| {
                Box::pin(async move {
                    inv_repo.destroy(tx, user_id, item_uid).await?;

                    Ok(())
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Connection(e) => e,
                TransactionError::Transaction(e) => e,
            })
    }
}
