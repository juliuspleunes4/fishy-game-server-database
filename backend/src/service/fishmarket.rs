use rocket::async_trait;
use sea_orm::{DatabaseConnection, DbErr, TransactionError, TransactionTrait};
use uuid::Uuid;

use crate::{controller::fishmarket::FishToSell, repository::{inventory::InventoryRepository, stats::StatsRepository}};

// Here you add your business logic here.
#[async_trait]
pub trait FishmarketService: Send + Sync {
    async fn sell_fishes(&self, seller_id: Uuid, fishes: Vec<FishToSell>, earned_money: i32) -> Result<(), DbErr>;
}

pub struct FishmarketServiceImpl<S: StatsRepository, I: InventoryRepository> {
    db: DatabaseConnection,
    stats_repository: S,
    inventory_repository: I,
}

impl<S: StatsRepository, I: InventoryRepository> FishmarketServiceImpl<S, I> {
    pub fn new(db: DatabaseConnection, stats_repository: S, inventory_repository: I) -> Self {
        Self {
            db,
            stats_repository,
            inventory_repository,
        }
    }
}

#[async_trait]
impl<S: StatsRepository + Clone + 'static, I: InventoryRepository + Clone + 'static> FishmarketService for FishmarketServiceImpl<S, I> {
    async fn sell_fishes(&self, seller_id: Uuid, fishes: Vec<FishToSell>, earned_money: i32) -> Result<(), DbErr> {
        let inventory_repo = self.inventory_repository.clone();
        let stats_repo = self.stats_repository.clone();
        self.db.transaction::<_, (), DbErr>(move |tx| {
            Box::pin(async move {
                for fish in fishes {
                    match fish.new_state_blob {
                        Some(state_blob) => inventory_repo.add_or_update_tx(tx, seller_id, fish.fish_uid, fish.fish_id, state_blob).await,
                        None => inventory_repo.destroy(tx, seller_id, fish.fish_uid).await,
                    }?;
                }
                stats_repo.change_bucks_tx(tx, seller_id, earned_money).await
            })
        })
        .await
        .map_err(|e| match e {
            TransactionError::Connection(e) => e,
            TransactionError::Transaction(e) => e,
        })
    }
}
