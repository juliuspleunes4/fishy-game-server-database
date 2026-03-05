use rocket::async_trait;
use sea_orm::{DatabaseConnection, DbErr, TransactionError, TransactionTrait};
use uuid::Uuid;

use crate::{
    domain::{ItemType, SelectItemRequest, StatFish},
    repository::stats::StatsRepository,
};

#[async_trait]
pub trait StatsService: Send + Sync {
    async fn add_xp(&self, user_id: Uuid, amount: i32) -> Result<(), DbErr>;

    async fn add_playtime(&self, user_id: Uuid, amount: i32) -> Result<(), DbErr>;

    async fn add_fish(&self, user_caught: Uuid, fish: StatFish, xp_earned: i32) -> Result<(), DbErr>;

    async fn select_item(&self, select_item: SelectItemRequest) -> Result<(), DbErr>;
}

pub struct StatsServiceImpl<T: StatsRepository> {
    db: DatabaseConnection,
    stats_repository: T,
}

impl<R: StatsRepository + Clone> StatsServiceImpl<R> {
    // create a new function for StatsServiceImpl.
    pub fn new(db: DatabaseConnection, stats_repository: R) -> Self {
        Self { db, stats_repository }
    }
}

// Implement StatsService trait for StatsServiceImpl.
#[async_trait]
impl<R: StatsRepository + Clone + 'static> StatsService for StatsServiceImpl<R> {
    async fn add_xp(&self, user_id: Uuid, amount: i32) -> Result<(), DbErr> {
        let stats_repo = self.stats_repository.clone();

        self.db
            .transaction::<_, (), DbErr>(move |tx| {
                Box::pin(async move { stats_repo.add_xp(tx, user_id, amount).await })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Connection(e) => e,
                TransactionError::Transaction(e) => e,
            })
    }

    async fn add_playtime(&self, user_id: Uuid, amount: i32) -> Result<(), DbErr> {
        let stats_repo = self.stats_repository.clone();

        self.db
            .transaction::<_, (), DbErr>(move |tx| {
                Box::pin(async move { stats_repo.add_playtime(tx, user_id, amount).await })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Connection(e) => e,
                TransactionError::Transaction(e) => e,
            })
    }

    async fn add_fish(&self, user_caught: Uuid, fish: StatFish, xp_earned: i32) -> Result<(), DbErr> {
        let stats_repo = self.stats_repository.clone();

        self.db
            .transaction::<_, (), DbErr>(move |tx| {
                Box::pin(async move {
                    stats_repo.add_fish_tx(tx, user_caught, fish).await?;
                    stats_repo.add_xp(tx, user_caught, xp_earned as i32).await
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Connection(e) => e,
                TransactionError::Transaction(e) => e,
            })
    }

    async fn select_item(&self, item_request: SelectItemRequest) -> Result<(), DbErr> {
        match item_request.item_type {
            ItemType::Rod => {
                let stats_repo = self.stats_repository.clone();
                let user_id = item_request.user_id;
                let item_uid = item_request.item_uid;

                self.db
                    .transaction::<_, (), DbErr>(move |tx| {
                        Box::pin(async move { stats_repo.select_rod(tx, user_id, item_uid).await })
                    })
                    .await
                    .map_err(|e| match e {
                        TransactionError::Connection(e) => e,
                        TransactionError::Transaction(e) => e,
                    })
            }
            ItemType::Bait => {
                let stats_repo = self.stats_repository.clone();
                let user_id = item_request.user_id;
                let item_uid = item_request.item_uid;

                self.db
                    .transaction::<_, (), DbErr>(move |tx| {
                        Box::pin(async move {
                            stats_repo.select_bait(tx, user_id, item_uid).await
                        })
                    })
                    .await
                    .map_err(|e| match e {
                        TransactionError::Connection(e) => e,
                        TransactionError::Transaction(e) => e,
                    })
            }
            ItemType::Extra => unimplemented!(),
        }
    }
}
