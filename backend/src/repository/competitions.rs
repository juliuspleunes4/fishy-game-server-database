// TODO: These queries use sqlx::query! macros for compile-time SQL validation.
// If you see "set DATABASE_URL" errors and don't have database access, this is expected.
// Project owner should run `cargo sqlx prepare` to generate .sqlx/ cache and commit it,
// then all contributors can compile without DATABASE_URL while keeping type safety.

use crate::domain::{Competition, CompetitionResult};
use chrono::{DateTime, Utc};
use rocket::async_trait;
use sqlx::{Error, PgPool};
use uuid::Uuid;

#[async_trait]
pub trait CompetitionsRepository: Send + Sync {
    /// Get all active competitions (status = 'ACTIVE')
    async fn get_active_competitions(&self) -> Result<Vec<Competition>, sqlx::Error>;
    
    /// Get all upcoming competitions (status = 'SCHEDULED')
    async fn get_upcoming_competitions(&self) -> Result<Vec<Competition>, sqlx::Error>;
    
    /// Get a specific competition by ID
    async fn get_competition_by_id(&self, competition_id: Uuid) -> Result<Option<Competition>, sqlx::Error>;
    
    /// Get leaderboard for a specific competition
    async fn get_competition_results(&self, competition_id: Uuid) -> Result<Vec<CompetitionResult>, sqlx::Error>;
    
    /// Submit or update a player's score for a competition
    async fn submit_score(&self, competition_id: Uuid, player_id: Uuid, score: i32) -> Result<(), sqlx::Error>;
    
    /// Create a new competition
    async fn create_competition(
        &self,
        competition_id: Uuid,
        competition_type: i32,
        target_fish_id: i32,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        reward_currency: String,
        prize_pool: Vec<i32>,
    ) -> Result<(), sqlx::Error>;
    
    /// Update competition status
    async fn update_competition_status(&self, competition_id: Uuid, status: String) -> Result<(), sqlx::Error>;
    
    /// Count competitions by status
    async fn count_competitions_by_status(&self, status: String) -> Result<i64, sqlx::Error>;
}

#[derive(Debug, Clone)]
pub struct CompetitionsRepositoryImpl {
    pool: PgPool,
}

impl CompetitionsRepositoryImpl {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CompetitionsRepository for CompetitionsRepositoryImpl {
    async fn get_active_competitions(&self) -> Result<Vec<Competition>, sqlx::Error> {
        let competitions = sqlx::query_as!(
            Competition,
            r#"SELECT competition_id, competition_type, target_fish_id, 
                      start_time, end_time, reward_currency, prize_pool, 
                      created_at, status
               FROM competitions
               WHERE status = 'ACTIVE'
               ORDER BY start_time ASC"#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(competitions)
    }

    async fn get_upcoming_competitions(&self) -> Result<Vec<Competition>, sqlx::Error> {
        let competitions = sqlx::query_as!(
            Competition,
            r#"SELECT competition_id, competition_type, target_fish_id, 
                      start_time, end_time, reward_currency, prize_pool, 
                      created_at, status
               FROM competitions
               WHERE status = 'SCHEDULED'
               ORDER BY start_time ASC"#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(competitions)
    }

    async fn get_competition_by_id(&self, competition_id: Uuid) -> Result<Option<Competition>, sqlx::Error> {
        let competition = sqlx::query_as!(
            Competition,
            r#"SELECT competition_id, competition_type, target_fish_id, 
                      start_time, end_time, reward_currency, prize_pool, 
                      created_at, status
               FROM competitions
               WHERE competition_id = $1"#,
            competition_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(competition)
    }

    async fn get_competition_results(&self, competition_id: Uuid) -> Result<Vec<CompetitionResult>, sqlx::Error> {
        let results = sqlx::query_as!(
            CompetitionResult,
            r#"SELECT result_id, competition_id, player_id, score, last_updated
               FROM competition_results
               WHERE competition_id = $1
               ORDER BY score DESC, last_updated ASC
               LIMIT 100"#,
            competition_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }

    async fn submit_score(&self, competition_id: Uuid, player_id: Uuid, score: i32) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"INSERT INTO competition_results (competition_id, player_id, score, last_updated)
               VALUES ($1, $2, $3, NOW())
               ON CONFLICT (competition_id, player_id)
               DO UPDATE SET score = $3, last_updated = NOW()"#,
            competition_id,
            player_id,
            score
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn create_competition(
        &self,
        competition_id: Uuid,
        competition_type: i32,
        target_fish_id: i32,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        reward_currency: String,
        prize_pool: Vec<i32>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"INSERT INTO competitions 
               (competition_id, competition_type, target_fish_id, start_time, end_time, 
                reward_currency, prize_pool, status)
               VALUES ($1, $2, $3, $4, $5, $6, $7, 'SCHEDULED')"#,
            competition_id,
            competition_type,
            target_fish_id,
            start_time,
            end_time,
            reward_currency,
            &prize_pool
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn update_competition_status(&self, competition_id: Uuid, status: String) -> Result<(), sqlx::Error> {
        let result = sqlx::query!(
            r#"UPDATE competitions
               SET status = $2
               WHERE competition_id = $1"#,
            competition_id,
            status
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(Error::RowNotFound);
        }

        Ok(())
    }

    async fn count_competitions_by_status(&self, status: String) -> Result<i64, sqlx::Error> {
        let result = sqlx::query!(
            r#"SELECT COUNT(*) as count FROM competitions WHERE status = $1"#,
            status
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result.count.unwrap_or(0))
    }
}
