use rocket::async_trait;
use uuid::Uuid;

use crate::repository::inventory::InventoryRepository;

// Here you add your business logic here.
#[async_trait]
pub trait InventoryService: Send + Sync {
    async fn add_or_update(
        &self,
        user_id: Uuid,
        item_uuid: Uuid,
        definition_id: i32,
        state_blob: String,
    ) -> Result<(), sqlx::Error>;

    async fn destroy(&self, user_id: Uuid, item_uid: Uuid) -> Result<(), sqlx::Error>;
}

pub struct InventoryServiceImpl<T: InventoryRepository> {
    inventory_repository: T,
}

impl<R: InventoryRepository> InventoryServiceImpl<R> {
    // create a new function for InventoryServiceImpl.
    pub fn new(inventory_repository: R) -> Self {
        Self {
            inventory_repository,
        }
    }
}

// Implement InventoryService trait for InventoryServiceImpl.
#[async_trait]
impl<R: InventoryRepository> InventoryService for InventoryServiceImpl<R> {
    async fn add_or_update(
        &self,
        user_id: Uuid,
        item_uuid: Uuid,
        definition_id: i32,
        state_blob: String,
    ) -> Result<(), sqlx::Error> {
        self.inventory_repository
            .add_or_update(user_id, item_uuid, definition_id, state_blob)
            .await
    }

    async fn destroy(&self, user_id: Uuid, item_uid: Uuid) -> Result<(), sqlx::Error> {
        self.inventory_repository.destroy(user_id, item_uid).await
    }
}
