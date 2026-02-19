use crate::domain::StatFish;
use rocket::async_trait;
use sqlx::{Error, PgPool};
use uuid::Uuid;

#[async_trait]
pub trait StatsRepository: Send + Sync {
    async fn add_xp(&self, user_id: Uuid, amount: i32) -> Result<(), sqlx::Error>;

    async fn change_bucks(&self, user_id: Uuid, amount: i32) -> Result<(), sqlx::Error>;

    async fn change_coins(&self, user_id: Uuid, amount: i32) -> Result<(), sqlx::Error>;

    async fn add_playtime(&self, user_id: Uuid, amount: i32) -> Result<(), sqlx::Error>;

    async fn add_fish(&self, fish: StatFish) -> Result<(), sqlx::Error>;

    async fn select_rod(&self, user_id: Uuid, rod_uid: Uuid) -> Result<(), sqlx::Error>;

    async fn select_bait(&self, user_id: Uuid, bait_uid: Uuid) -> Result<(), sqlx::Error>;
}

#[derive(Debug, Clone)]
pub struct StatsRepositoryImpl {
    pool: PgPool,
}

impl StatsRepositoryImpl {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl StatsRepository for StatsRepositoryImpl {
    async fn add_xp(&self, user_id: Uuid, amount: i32) -> Result<(), sqlx::Error> {
        let result = sqlx::query!(
            "UPDATE stats
            SET xp = xp + $1
            WHERE user_id = $2",
            amount,
            user_id,
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(Error::RowNotFound);
        }

        Ok(())
    }

    async fn change_bucks(&self, user_id: Uuid, amount: i32) -> Result<(), sqlx::Error> {
        let result = sqlx::query!(
            "UPDATE stats
            SET bucks = bucks + $1
            WHERE user_id = $2",
            amount,
            user_id,
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(Error::RowNotFound);
        }

        Ok(())
    }

    async fn change_coins(&self, user_id: Uuid, amount: i32) -> Result<(), sqlx::Error> {
        let result = sqlx::query!(
            "UPDATE stats
            SET coins = coins + $1
            WHERE user_id = $2",
            amount,
            user_id,
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(Error::RowNotFound);
        }

        Ok(())
    }

    async fn add_playtime(&self, user_id: Uuid, amount: i32) -> Result<(), sqlx::Error> {
        let result = sqlx::query!(
            "UPDATE stats
            SET total_playtime = total_playtime + $1
            WHERE user_id = $2",
            amount,
            user_id,
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(Error::RowNotFound);
        }

        Ok(())
    }

    async fn add_fish(&self, fish: StatFish) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        // Insert or update fish
        sqlx::query!(
            "
            INSERT INTO fish_caught (user_id, fish_id, amount, max_length, first_caught)
            VALUES ($1, $2, 1, $3, CURRENT_DATE)
            ON CONFLICT (user_id, fish_id)
            DO UPDATE SET
                amount = fish_caught.amount + 1,
                max_length = GREATEST(fish_caught.max_length, EXCLUDED.max_length);
            ",
            fish.user_id,
            fish.fish_id,
            fish.length,
        )
        .execute(&mut *tx)
        .await?;

        // Insert user-fish-bait combination
        sqlx::query!(
            "INSERT INTO fish_caught_bait (user_id, fish_id, bait_id)
            VALUES ($1, $2, $3)
            ON CONFLICT DO NOTHING;
            ",
            fish.user_id,
            fish.fish_id,
            fish.bait_id,
        )
        .execute(&mut *tx)
        .await?;

        // Insert user-fish-area combination
        sqlx::query!(
            "INSERT INTO fish_caught_area (user_id, fish_id, area_id)
            VALUES ($1, $2, $3)
            ON CONFLICT DO NOTHING;
            ",
            fish.user_id,
            fish.fish_id,
            fish.area_id,
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    async fn select_rod(&self, user_id: Uuid, rod_uid: Uuid) -> Result<(), sqlx::Error> {
        let result = match sqlx::query!(
            "UPDATE stats
            SET selected_rod = $2
            WHERE user_id = $1",
            user_id,
            rod_uid,
        )
        .execute(&self.pool)
        .await
        {
            Ok(o) => o,
            Err(e) => {
                dbg!(&e);
                return Err(e);
            }
        };

        if result.rows_affected() == 0 {
            return Err(Error::RowNotFound);
        }

        Ok(())
    }

    async fn select_bait(&self, user_id: Uuid, bait_uid: Uuid) -> Result<(), sqlx::Error> {
        let result = match sqlx::query!(
            "UPDATE stats
            SET selected_bait = $2
            WHERE user_id = $1",
            user_id,
            bait_uid,
        )
        .execute(&self.pool)
        .await
        {
            Ok(o) => o,
            Err(e) => {
                dbg!(&e);
                return Err(e);
            }
        };

        if result.rows_affected() == 0 {
            return Err(Error::RowNotFound);
        }

        Ok(())
    }
}
