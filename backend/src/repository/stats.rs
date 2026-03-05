use crate::{
    domain::StatFish,
    entity::{fish_caught, fish_caught_area, fish_caught_bait, stats},
};
use rocket::async_trait;
use sea_orm::{
    prelude::Expr,
    sea_query::{Alias, Func, OnConflict},
    ActiveValue::{NotSet, Set},
    ColumnTrait, DatabaseTransaction, DbErr, EntityTrait, ExprTrait, QueryFilter,
};
use uuid::Uuid;

#[async_trait]
pub trait StatsRepository: Send + Sync {
    async fn add_xp(
        &self,
        tx: &DatabaseTransaction,
        user_id: Uuid,
        amount: i32,
    ) -> Result<(), DbErr>;

    async fn change_bucks(
        &self,
        tx: &DatabaseTransaction,
        user_id: Uuid,
        amount: i32,
    ) -> Result<(), DbErr>;

    async fn change_coins(
        &self,
        tx: &DatabaseTransaction,
        user_id: Uuid,
        amount: i32,
    ) -> Result<(), DbErr>;

    /// Transactional variants for use inside an existing `DatabaseTransaction`.
    async fn change_bucks_tx(
        &self,
        tx: &DatabaseTransaction,
        user_id: Uuid,
        amount: i32,
    ) -> Result<(), DbErr>;

    async fn change_coins_tx(
        &self,
        tx: &DatabaseTransaction,
        user_id: Uuid,
        amount: i32,
    ) -> Result<(), DbErr>;

    async fn add_playtime(
        &self,
        tx: &DatabaseTransaction,
        user_id: Uuid,
        amount: i32,
    ) -> Result<(), DbErr>;

    async fn add_fish_tx(
        &self,
        tx: &DatabaseTransaction,
        user_caught: Uuid,
        fish: StatFish,
    ) -> Result<(), DbErr>;

    async fn select_rod(
        &self,
        tx: &DatabaseTransaction,
        user_id: Uuid,
        rod_uid: Uuid,
    ) -> Result<(), DbErr>;

    async fn select_bait(
        &self,
        tx: &DatabaseTransaction,
        user_id: Uuid,
        bait_uid: Uuid,
    ) -> Result<(), DbErr>;

    async fn insert_new_stats(
        &self,
        tx: &DatabaseTransaction,
        user_id: Uuid,
        coins: i32,
        bucks: i32,
    ) -> Result<(), DbErr>;
}

#[derive(Debug, Clone)]
pub struct StatsRepositoryImpl;

impl StatsRepositoryImpl {
    pub fn new() -> Self {
        Self
    }

    async fn add_fish_caught(tx: &DatabaseTransaction, user_caught: Uuid, fish: &StatFish) -> Result<(), DbErr> {
        fish_caught::Entity::insert(fish_caught::ActiveModel {
            user_id: Set(user_caught),
            fish_id: Set(fish.fish_id),
            amount: Set(1),
            max_length: Set(fish.length),
            first_caught: NotSet,
        })
        .on_conflict(
            OnConflict::columns([fish_caught::Column::UserId, fish_caught::Column::FishId])
                .update_columns([fish_caught::Column::Amount, fish_caught::Column::MaxLength])
                .value(
                    fish_caught::Column::Amount,
                    Expr::col(fish_caught::Column::Amount).add(1),
                )
                .value(
                    fish_caught::Column::MaxLength,
                    Func::greatest([
                        Expr::col(fish_caught::Column::MaxLength),
                        Expr::col((Alias::new("excluded"), fish_caught::Column::MaxLength)),
                    ]),
                )
                .to_owned(),
        )
        .exec(tx)
        .await?;
        Ok(())
    }

    async fn add_fish_bait_caught(
        tx: &DatabaseTransaction,
        user_caught: Uuid,
        fish: &StatFish,
    ) -> Result<(), DbErr> {
        fish_caught_bait::Entity::insert(fish_caught_bait::ActiveModel {
            user_id: Set(user_caught),
            fish_id: Set(fish.fish_id),
            bait_id: Set(fish.bait_id),
        })
        .on_conflict(OnConflict::new().do_nothing().to_owned())
        .exec(tx)
        .await?;

        Ok(())
    }

    async fn add_fish_area_caught(
        tx: &DatabaseTransaction,
        user_caught: Uuid,
        fish: &StatFish,
    ) -> Result<(), DbErr> {
        fish_caught_area::Entity::insert(fish_caught_area::ActiveModel {
            user_id: Set(user_caught),
            fish_id: Set(fish.fish_id),
            area_id: Set(fish.area_id),
        })
        .on_conflict(OnConflict::new().do_nothing().to_owned())
        .exec(tx)
        .await?;
        Ok(())
    }
}

#[async_trait]
impl StatsRepository for StatsRepositoryImpl {
    async fn add_xp(
        &self,
        tx: &DatabaseTransaction,
        user_id: Uuid,
        amount: i32,
    ) -> Result<(), DbErr> {
        let result = stats::Entity::update_many()
            .col_expr(stats::Column::Xp, Expr::col(stats::Column::Xp).add(amount))
            .filter(stats::Column::UserId.eq(user_id))
            .exec(tx)
            .await?;

        if result.rows_affected == 0 {
            return Err(DbErr::RecordNotFound("stats->xp".into()));
        }

        Ok(())
    }

    async fn change_bucks(
        &self,
        tx: &DatabaseTransaction,
        user_id: Uuid,
        amount: i32,
    ) -> Result<(), DbErr> {
        let result = stats::Entity::update_many()
            .col_expr(
                stats::Column::Bucks,
                Expr::col(stats::Column::Bucks).add(amount),
            )
            .filter(stats::Column::UserId.eq(user_id))
            .exec(tx)
            .await?;

        if result.rows_affected == 0 {
            return Err(DbErr::RecordNotFound("stats->bucks".into()));
        }

        Ok(())
    }

    async fn change_coins(
        &self,
        tx: &DatabaseTransaction,
        user_id: Uuid,
        amount: i32,
    ) -> Result<(), DbErr> {
        let result = stats::Entity::update_many()
            .col_expr(
                stats::Column::Coins,
                Expr::col(stats::Column::Coins).add(amount),
            )
            .filter(stats::Column::UserId.eq(user_id))
            .exec(tx)
            .await?;

        if result.rows_affected == 0 {
            return Err(DbErr::RecordNotFound("stats->bucks".into()));
        }

        Ok(())
    }

    async fn change_bucks_tx(
        &self,
        tx: &DatabaseTransaction,
        user_id: Uuid,
        amount: i32,
    ) -> Result<(), DbErr> {
        self.change_bucks(tx, user_id, amount).await
    }

    async fn change_coins_tx(
        &self,
        tx: &DatabaseTransaction,
        user_id: Uuid,
        amount: i32,
    ) -> Result<(), DbErr> {
        self.change_coins(tx, user_id, amount).await
    }

    async fn add_playtime(
        &self,
        tx: &DatabaseTransaction,
        user_id: Uuid,
        amount: i32,
    ) -> Result<(), DbErr> {
        let result = stats::Entity::update_many()
            .col_expr(
                stats::Column::TotalPlaytime,
                Expr::col(stats::Column::TotalPlaytime).add(amount),
            )
            .filter(stats::Column::Coins.eq(user_id))
            .exec(tx)
            .await?;

        if result.rows_affected == 0 {
            return Err(DbErr::RecordNotFound("stats->bucks".into()));
        }

        Ok(())
    }

    async fn add_fish_tx(
        &self,
        tx: &DatabaseTransaction,
        user_caught: Uuid,
        fish: StatFish,
    ) -> Result<(), DbErr> {
        Self::add_fish_caught(tx, user_caught, &fish).await?;
        Self::add_fish_bait_caught(tx, user_caught, &fish).await?;
        Self::add_fish_area_caught(tx, user_caught, &fish).await?;
        Ok(())
    }

    async fn select_rod(
        &self,
        tx: &DatabaseTransaction,
        user_id: Uuid,
        rod_uid: Uuid,
    ) -> Result<(), DbErr> {
        stats::Entity::update_many()
            .col_expr(stats::Column::SelectedRod, Expr::value(Some(rod_uid)))
            .filter(stats::Column::UserId.eq(user_id))
            .exec(tx)
            .await?;

        Ok(())
    }

    async fn select_bait(
        &self,
        tx: &DatabaseTransaction,
        user_id: Uuid,
        bait_uid: Uuid,
    ) -> Result<(), DbErr> {
        stats::Entity::update_many()
            .col_expr(stats::Column::SelectedBait, Expr::value(Some(bait_uid)))
            .filter(stats::Column::UserId.eq(user_id))
            .exec(tx)
            .await?;

        Ok(())
    }

    async fn insert_new_stats(
        &self,
        tx: &DatabaseTransaction,
        user_id: Uuid,
        coins: i32,
        bucks: i32,
    ) -> Result<(), DbErr> {
        stats::Entity::insert(stats::ActiveModel {
            user_id: Set(user_id),
            coins: Set(coins),
            bucks: Set(bucks),
            ..Default::default()
        })
        .exec(tx)
        .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{DatabaseBackend, QueryTrait};

    #[test]
    fn print_select_rod_sql() {
        let user_id = Uuid::new_v4();
        let rod_uid = Uuid::new_v4();

        let stmt = stats::Entity::update_many()
            .col_expr(stats::Column::SelectedRod, Expr::value(Some(rod_uid)))
            .filter(stats::Column::UserId.eq(user_id))
            .build(DatabaseBackend::Postgres);
        println!("{}", stmt);
    }
}

