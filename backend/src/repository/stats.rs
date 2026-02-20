use crate::{domain::StatFish, entity::{fish_caught, fish_caught_area, fish_caught_bait, stats}};
use rocket::async_trait;
use sea_orm::{
    ActiveValue::{NotSet, Set}, ColumnTrait, DatabaseConnection, DatabaseTransaction, DbErr,
    EntityTrait, ExprTrait, QueryFilter, TransactionError, TransactionTrait,
    prelude::Expr, sea_query::{Alias, Func, OnConflict},
};
use sqlx::PgPool;
use uuid::Uuid;

#[async_trait]
pub trait StatsRepository: Send + Sync {
    async fn add_xp(&self, user_id: Uuid, amount: i32) -> Result<(), DbErr>;

    async fn change_bucks(&self, user_id: Uuid, amount: i32) -> Result<(), DbErr>;

    async fn change_coins(&self, user_id: Uuid, amount: i32) -> Result<(), DbErr>;

    async fn add_playtime(&self, user_id: Uuid, amount: i32) -> Result<(), DbErr>;

    async fn add_fish_caught(tx: &DatabaseTransaction, fish: &StatFish) -> Result<(), DbErr>;

    async fn add_fish_bait_caught(tx: &DatabaseTransaction, fish: &StatFish) -> Result<(), DbErr>;

    async fn add_fish_area_caught(tx: &DatabaseTransaction, fish: &StatFish) -> Result<(), DbErr>;

    async fn add_fish(&self, fish: StatFish) -> Result<(), DbErr>;

    async fn select_rod(&self, user_id: Uuid, rod_uid: Uuid) -> Result<(), DbErr>;

    async fn select_bait(&self, user_id: Uuid, bait_uid: Uuid) -> Result<(), DbErr>;
}

#[derive(Debug, Clone)]
pub struct StatsRepositoryImpl {
    db: DatabaseConnection,
}

impl StatsRepositoryImpl {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl StatsRepository for StatsRepositoryImpl {
    async fn add_xp(&self, user_id: Uuid, amount: i32) -> Result<(), DbErr> {
        let result = stats::Entity::update_many()
            .col_expr(stats::Column::Xp, Expr::col(stats::Column::Xp).add(amount))
            .filter(stats::Column::UserId.eq(user_id))
            .exec(&self.db)
            .await?;

        if result.rows_affected == 0 {
            return Err(DbErr::RecordNotFound("stats->xp".into()));
        }

        Ok(())
    }

    async fn change_bucks(&self, user_id: Uuid, amount: i32) -> Result<(), DbErr> {
        let result = stats::Entity::update_many()
            .col_expr(
                stats::Column::Bucks,
                Expr::col(stats::Column::Bucks).add(amount),
            )
            .filter(stats::Column::UserId.eq(user_id))
            .exec(&self.db)
            .await?;

        if result.rows_affected == 0 {
            return Err(DbErr::RecordNotFound("stats->bucks".into()));
        }

        Ok(())
    }

    async fn change_coins(&self, user_id: Uuid, amount: i32) -> Result<(), DbErr> {
        let result = stats::Entity::update_many()
            .col_expr(
                stats::Column::Coins,
                Expr::col(stats::Column::Coins).add(amount),
            )
            .filter(stats::Column::UserId.eq(user_id))
            .exec(&self.db)
            .await?;

        if result.rows_affected == 0 {
            return Err(DbErr::RecordNotFound("stats->bucks".into()));
        }

        Ok(())
    }

    async fn add_playtime(&self, user_id: Uuid, amount: i32) -> Result<(), DbErr> {
        let result = stats::Entity::update_many().col_expr(stats::Column::TotalPlaytime, Expr::col(stats::Column::TotalPlaytime).add(amount))
        .filter(stats::Column::Coins.eq(user_id)).exec(&self.db).await?;
        
        if result.rows_affected == 0 {
            return Err(DbErr::RecordNotFound("stats->bucks".into()));
        }

        Ok(())
    }

    async fn add_fish_caught(tx: &DatabaseTransaction, fish: &StatFish) -> Result<(), DbErr> {
        fish_caught::Entity::insert(
            fish_caught::ActiveModel {
                user_id: Set(fish.user_id),
                fish_id: Set(fish.fish_id),
                amount: Set(1),
                max_length: Set(fish.length),
                first_caught: NotSet,
        }).on_conflict(
        OnConflict::columns([
            fish_caught::Column::UserId,
            fish_caught::Column::FishId,
        ])
        .update_columns([
            fish_caught::Column::Amount,
            fish_caught::Column::MaxLength,
        ])
        .value(
            fish_caught::Column::Amount,
            Expr::col(fish_caught::Column::Amount).add(1),
        )
        .value(
        fish_caught::Column::MaxLength,
        Func::greatest([
            Expr::col(fish_caught::Column::MaxLength),
            Expr::col((Alias::new("excluded"), fish_caught::Column::MaxLength))
        ]),
        ).to_owned()).exec(tx).await?;
        Ok(())
    }

    async fn add_fish_bait_caught(tx: &DatabaseTransaction, fish: &StatFish) -> Result<(), DbErr> {
        fish_caught_bait::Entity::insert(
            fish_caught_bait::ActiveModel {
                user_id: Set(fish.user_id),
                fish_id: Set(fish.fish_id),
                bait_id: Set(fish.bait_id),
            }
        ).on_conflict(OnConflict::new().do_nothing().to_owned())
        .exec(tx).await?;

        Ok(())
    }

    async fn add_fish_area_caught(tx: &DatabaseTransaction, fish: &StatFish) -> Result<(), DbErr> {
        fish_caught_area::Entity::insert(
            fish_caught_area::ActiveModel {
                user_id: Set(fish.user_id),
                fish_id: Set(fish.fish_id),
                area_id: Set(fish.area_id),
            }
        ).on_conflict(OnConflict::new().do_nothing().to_owned()).exec(tx).await?;
        Ok(())
    }

    async fn add_fish(&self, fish: StatFish) -> Result<(), DbErr> {
        self.db
            .transaction::<_, (), DbErr>(|tx| {
                Box::pin(async move {
                    Self::add_fish_caught(tx, &fish).await?;
                    Self::add_fish_bait_caught(tx, &fish).await?;
                    Self::add_fish_area_caught(tx, &fish).await?;
                    Ok(())
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Connection(e) => e,
                TransactionError::Transaction(e) => e,
            })
    }

    async fn select_rod(&self, user_id: Uuid, rod_uid: Uuid) -> Result<(), DbErr> {
        stats::Entity::update_many()
            .col_expr(stats::Column::SelectedRod, Expr::value(Some(rod_uid)))
            .filter(stats::Column::UserId.eq(user_id))
            .exec(&self.db).await?;

        Ok(())
    }

    async fn select_bait(&self, user_id: Uuid, bait_uid: Uuid) -> Result<(), DbErr> {
        stats::Entity::update_many()
            .col_expr(stats::Column::SelectedBait, Expr::value(Some(bait_uid)))
            .filter(stats::Column::UserId.eq(user_id))
            .exec(&self.db).await?;

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
