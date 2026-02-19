use rocket::async_trait;
use uuid::Uuid;

use crate::{
    domain::{ItemType, SelectItemRequest, StatFish},
    repository::stats::StatsRepository,
};

// Here you add your business logic here.
#[async_trait]
pub trait StatsService: Send + Sync {
    async fn add_xp(&self, user_id: Uuid, amount: i32) -> Result<(), sqlx::Error>;

    async fn change_bucks(&self, user_id: Uuid, amount: i32) -> Result<(), sqlx::Error>;

    async fn change_coins(&self, user_id: Uuid, amount: i32) -> Result<(), sqlx::Error>;

    async fn add_playtime(&self, user_id: Uuid, amount: i32) -> Result<(), sqlx::Error>;

    async fn add_fish(&self, fish: StatFish) -> Result<(), sqlx::Error>;

    async fn select_item(&self, select_item: SelectItemRequest) -> Result<(), sqlx::Error>;
}

pub struct StatsServiceImpl<T: StatsRepository> {
    stats_repository: T,
}

impl<R: StatsRepository> StatsServiceImpl<R> {
    // create a new function for StatsServiceImpl.
    pub fn new(stats_repository: R) -> Self {
        Self { stats_repository }
    }
}

// Implement StatsService trait for StatsServiceImpl.
#[async_trait]
impl<R: StatsRepository> StatsService for StatsServiceImpl<R> {
    async fn add_xp(&self, user_id: Uuid, amount: i32) -> Result<(), sqlx::Error> {
        self.stats_repository.add_xp(user_id, amount).await
    }

    async fn change_bucks(&self, user_id: Uuid, amount: i32) -> Result<(), sqlx::Error> {
        self.stats_repository.change_bucks(user_id, amount).await
    }

    async fn change_coins(&self, user_id: Uuid, amount: i32) -> Result<(), sqlx::Error> {
        self.stats_repository.change_coins(user_id, amount).await
    }

    async fn add_playtime(&self, user_id: Uuid, amount: i32) -> Result<(), sqlx::Error> {
        self.stats_repository.add_playtime(user_id, amount).await
    }

    async fn add_fish(&self, fish: StatFish) -> Result<(), sqlx::Error> {
        self.stats_repository.add_fish(fish).await
    }

    async fn select_item(&self, item_request: SelectItemRequest) -> Result<(), sqlx::Error> {
        match item_request.item_type {
            ItemType::Rod => {
                self.stats_repository
                    .select_rod(item_request.user_id, item_request.item_uid)
                    .await
            }
            ItemType::Bait => {
                self.stats_repository
                    .select_bait(item_request.user_id, item_request.item_uid)
                    .await
            }
            ItemType::Extra => unimplemented!(),
        }
    }
}
