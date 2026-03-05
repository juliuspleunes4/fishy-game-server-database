use crate::{domain::ActiveEffect, entity::player_effects};
use chrono::{DateTime, Utc};
use rocket::async_trait;
use sea_orm::{
    prelude::Expr, sea_query::OnConflict, ActiveValue::Set, ColumnTrait, Condition,
    DatabaseTransaction, DbErr, EntityTrait, ExprTrait, QueryFilter, QueryOrder, QuerySelect,
};
use uuid::Uuid;

#[async_trait]
pub trait EffectsRepository: Send + Sync {
    async fn add_effect(
        &self,
        tx: &DatabaseTransaction,
        user_id: Uuid,
        item_id: i32,
        expiry_time: DateTime<Utc>,
    ) -> Result<(), DbErr>;

    async fn remove_effect(
        &self,
        tx: &DatabaseTransaction,
        user_id: Uuid,
        item_id: i32,
    ) -> Result<(), DbErr>;

    async fn get_active_effects(
        &self,
        tx: &DatabaseTransaction,
        user_id: Uuid,
    ) -> Result<Vec<ActiveEffect>, DbErr>;

    async fn remove_all_expired_effects_global(
        &self,
        tx: &DatabaseTransaction,
    ) -> Result<(), DbErr>;
}

#[derive(Debug, Clone)]
pub struct EffectsRepositoryImpl;

impl EffectsRepositoryImpl {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl EffectsRepository for EffectsRepositoryImpl {
    async fn add_effect(
        &self,
        tx: &DatabaseTransaction,
        user_id: Uuid,
        item_id: i32,
        expiry_time: DateTime<Utc>,
    ) -> Result<(), DbErr> {
        player_effects::Entity::insert(player_effects::ActiveModel {
            user_id: Set(user_id),
            item_id: Set(item_id),
            expiry_time: Set(expiry_time.fixed_offset()),
        })
        .on_conflict(
            OnConflict::columns([
                player_effects::Column::UserId,
                player_effects::Column::ItemId,
            ])
            .update_column(player_effects::Column::ExpiryTime)
            .to_owned(),
        )
        .exec(tx)
        .await?;

        Ok(())
    }

    async fn remove_effect(
        &self,
        tx: &DatabaseTransaction,
        user_id: Uuid,
        item_id: i32,
    ) -> Result<(), DbErr> {
        player_effects::Entity::delete_many()
            .filter(
                Condition::all()
                    .add(player_effects::Column::UserId.eq(user_id))
                    .add(player_effects::Column::ItemId.eq(item_id)),
            )
            .exec(tx)
            .await?;

        Ok(())
    }

    async fn get_active_effects(
        &self,
        tx: &DatabaseTransaction,
        user_id: Uuid,
    ) -> Result<Vec<ActiveEffect>, DbErr> {
        let effects = player_effects::Entity::find()
            .select_only()
            .column(player_effects::Column::ItemId)
            .column(player_effects::Column::ExpiryTime)
            .filter(player_effects::Column::UserId.eq(user_id))
            .filter(player_effects::Column::ExpiryTime.gt(Utc::now()))
            .order_by_asc(player_effects::Column::ExpiryTime)
            .into_model::<ActiveEffect>()
            .all(tx)
            .await?;

        Ok(effects)
    }

    async fn remove_all_expired_effects_global(
        &self,
        tx: &DatabaseTransaction,
    ) -> Result<(), DbErr> {
        player_effects::Entity::delete_many()
            .filter(Expr::col(player_effects::Column::ExpiryTime).lte(Expr::current_timestamp()))
            .exec(tx)
            .await?;
        Ok(())
    }
}
