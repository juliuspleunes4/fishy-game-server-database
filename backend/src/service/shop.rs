use rocket::async_trait;
use sea_orm::{DatabaseConnection, DbErr, TransactionError, TransactionTrait};
use uuid::Uuid;

use crate::{
    controller::shop::MoneyType,
    repository::{inventory::InventoryRepository, stats::StatsRepository},
};

#[async_trait]
pub trait ShopService: Send + Sync {
    async fn buy_item(
        &self,
        player_id: Uuid,
        item_def_id: i32,
        item_uuid: Uuid,
        item_state_blob: String,
        item_price: i32,
        bought_using: MoneyType,
    ) -> Result<(), DbErr>;
}

pub struct ShopServiceImpl<R: StatsRepository + Clone, T: InventoryRepository + Clone> {
    db: DatabaseConnection,
    stats_repository: R,
    inventory_repository: T,
}

impl<R: StatsRepository + Clone, T: InventoryRepository + Clone> ShopServiceImpl<R, T> {
    // create a new function for StatsServiceImpl.
    pub fn new(db: DatabaseConnection, stats_repository: R, inventory_repository: T) -> Self {
        Self {
            db,
            stats_repository,
            inventory_repository,
        }
    }
}

// Implement StatsService trait for ShopServiceImpl.
#[async_trait]
impl<R: StatsRepository + Clone + 'static, T: InventoryRepository + Clone + 'static> ShopService
    for ShopServiceImpl<R, T>
{
    async fn buy_item(
        &self,
        buyer_uuid: Uuid,
        item_def_id: i32,
        item_uuid: Uuid,
        item_state_blob: String,
        item_price: i32,
        bought_using: MoneyType,
    ) -> Result<(), DbErr> {
        let stats_repo = self.stats_repository.clone();
        let inv_repo = self.inventory_repository.clone();

        self.db
            .transaction::<_, (), DbErr>(move |tx| {
                Box::pin(async move {
                    inv_repo
                        .add_or_update_tx(tx, buyer_uuid, item_uuid, item_def_id, item_state_blob)
                        .await?;

                    match bought_using {
                        MoneyType::BUCKS => {
                            stats_repo
                                .change_bucks_tx(tx, buyer_uuid, item_price)
                                .await?;
                        }
                        MoneyType::COINS => {
                            stats_repo
                                .change_coins_tx(tx, buyer_uuid, item_price)
                                .await?;
                        }
                    }

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
