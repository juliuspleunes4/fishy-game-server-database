use chrono::Utc;
use rocket::async_trait;
use sea_orm::{DatabaseConnection, DbErr, TransactionError, TransactionTrait};
use uuid::Uuid;

use crate::{
    domain::{ActiveEffect, AddActiveEffectRequest},
    repository::effects::EffectsRepository,
};

// Here you add your business logic here.
#[async_trait]
pub trait EffectsService: Send + Sync {
    async fn add_effect(&self, request: AddActiveEffectRequest) -> Result<(), DbErr>;

    async fn remove_effect(&self, user_id: Uuid, item_id: i32) -> Result<(), DbErr>;

    async fn get_active_effects(&self, user_id: Uuid) -> Result<Vec<ActiveEffect>, DbErr>;

    async fn cleanup_all_expired_effects(&self) -> Result<(), DbErr>;
}

pub struct EffectsServiceImpl<T: EffectsRepository> {
    db: DatabaseConnection,
    effects_repository: T,
}

impl<R: EffectsRepository> EffectsServiceImpl<R> {
    // create a new function for EffectsServiceImpl.
    pub fn new(db: DatabaseConnection, effects_repository: R) -> Self {
        Self {
            db,
            effects_repository,
        }
    }
}

// Implement EffectsService trait for EffectsServiceImpl.
#[async_trait]
impl<R: EffectsRepository + Clone + 'static> EffectsService for EffectsServiceImpl<R> {
    async fn add_effect(&self, request: AddActiveEffectRequest) -> Result<(), DbErr> {
        // Validate that the expiry time is in the future
        if request.expiry_time <= Utc::now() {
            return Err(DbErr::Custom("Expiry time must be in the future".into()));
        }

        let effects_repo = self.effects_repository.clone();

        self.db
            .transaction::<_, (), DbErr>(move |tx| {
                Box::pin(async move {
                    effects_repo
                        .add_effect(tx, request.user_id, request.item_id, request.expiry_time)
                        .await
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Connection(e) => e,
                TransactionError::Transaction(e) => e,
            })
    }

    async fn remove_effect(&self, user_id: Uuid, item_id: i32) -> Result<(), DbErr> {
        let effects_repo = self.effects_repository.clone();

        self.db
            .transaction::<_, (), DbErr>(move |tx| {
                Box::pin(async move { effects_repo.remove_effect(tx, user_id, item_id).await })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Connection(e) => e,
                TransactionError::Transaction(e) => e,
            })
    }

    async fn get_active_effects(&self, user_id: Uuid) -> Result<Vec<ActiveEffect>, DbErr> {
        let effects_repo = self.effects_repository.clone();

        self.db
            .transaction::<_, Vec<ActiveEffect>, DbErr>(move |tx| {
                Box::pin(async move { effects_repo.get_active_effects(tx, user_id).await })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Connection(e) => e,
                TransactionError::Transaction(e) => e,
            })
    }

    async fn cleanup_all_expired_effects(&self) -> Result<(), DbErr> {
        let effects_repo = self.effects_repository.clone();

        self.db
            .transaction::<_, (), DbErr>(move |tx| {
                Box::pin(async move { effects_repo.remove_all_expired_effects_global(tx).await })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Connection(e) => e,
                TransactionError::Transaction(e) => e,
            })
    }
}
