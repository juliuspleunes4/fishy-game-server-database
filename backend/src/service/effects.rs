use chrono::Utc;
use rocket::async_trait;
use uuid::Uuid;

use crate::{
    domain::{ActiveEffect, AddActiveEffectRequest},
    repository::effects::EffectsRepository,
};

// Here you add your business logic here.
#[async_trait]
pub trait EffectsService: Send + Sync {
    async fn add_effect(&self, request: AddActiveEffectRequest) -> Result<(), sqlx::Error>;

    async fn remove_effect(&self, user_id: Uuid, item_id: i32) -> Result<(), sqlx::Error>;

    async fn get_active_effects(&self, user_id: Uuid) -> Result<Vec<ActiveEffect>, sqlx::Error>;

    async fn cleanup_all_expired_effects(&self) -> Result<(), sqlx::Error>;
}

pub struct EffectsServiceImpl<T: EffectsRepository> {
    effects_repository: T,
}

impl<R: EffectsRepository> EffectsServiceImpl<R> {
    // create a new function for EffectsServiceImpl.
    pub fn new(effects_repository: R) -> Self {
        Self { effects_repository }
    }
}

// Implement EffectsService trait for EffectsServiceImpl.
#[async_trait]
impl<R: EffectsRepository> EffectsService for EffectsServiceImpl<R> {
    async fn add_effect(&self, request: AddActiveEffectRequest) -> Result<(), sqlx::Error> {
        // Validate that the expiry time is in the future
        if request.expiry_time <= Utc::now() {
            return Err(sqlx::Error::Protocol(
                "Expiry time must be in the future".into(),
            ));
        }

        self.effects_repository
            .add_effect(request.user_id, request.item_id, request.expiry_time)
            .await
    }

    async fn remove_effect(&self, user_id: Uuid, item_id: i32) -> Result<(), sqlx::Error> {
        self.effects_repository
            .remove_effect(user_id, item_id)
            .await
    }

    async fn get_active_effects(&self, user_id: Uuid) -> Result<Vec<ActiveEffect>, sqlx::Error> {
        self.effects_repository.get_active_effects(user_id).await
    }

    async fn cleanup_all_expired_effects(&self) -> Result<(), sqlx::Error> {
        self.effects_repository
            .remove_all_expired_effects_global()
            .await
    }
}
