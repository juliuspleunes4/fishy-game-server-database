use crate::domain::{Competition, CompetitionParticipant, CreateCompetitionRequest};
use chrono::{DateTime, Utc};
use rocket::async_trait;
use sqlx::{PgPool, Error};
use uuid::Uuid;

#[async_trait]
pub trait CompetitionsRepository: Send + Sync {
    async fn create_competition(&self, request: CreateCompetitionRequest) -> Result<Competition, Error>;
    async fn get_active_competitions(&self) -> Result<Vec<Competition>, Error>;
    async fn get_upcoming_competitions(&self) -> Result<Vec<Competition>, Error>;
    async fn get_competition_by_id(&self, competition_id: Uuid) -> Result<Option<Competition>, Error>;
    async fn get_leaderboard(&self, competition_id: Uuid, limit: i32) -> Result<Vec<CompetitionParticipant>, Error>;
    async fn update_participant_score(&self, competition_id: Uuid, user_id: Uuid, user_name: String, score: i32) -> Result<(), Error>;
    async fn delete_old_competitions(&self, before: DateTime<Utc>) -> Result<u64, Error>;
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
    async fn create_competition(&self, request: CreateCompetitionRequest) -> Result<Competition, Error> {
        if request.prizes.len() != 10 {
            return Err(Error::Protocol("prizes must contain exactly 10 values".into()));
        }

        let competition_id = Uuid::new_v4();
        
        let competition = sqlx::query_as!(
            Competition,
            "INSERT INTO competitions (
                competition_id, competition_type, target_fish_id, 
                start_time, end_time, reward_currency,
                prize_1st, prize_2nd, prize_3rd, prize_4th, prize_5th,
                prize_6th, prize_7th, prize_8th, prize_9th, prize_10th
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            RETURNING *",
            competition_id,
            request.competition_type,
            request.target_fish_id,
            request.start_time,
            request.end_time,
            request.reward_currency,
            request.prizes[0],
            request.prizes[1],
            request.prizes[2],
            request.prizes[3],
            request.prizes[4],
            request.prizes[5],
            request.prizes[6],
            request.prizes[7],
            request.prizes[8],
            request.prizes[9],
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(competition)
    }

    async fn get_active_competitions(&self) -> Result<Vec<Competition>, Error> {
        let competitions = sqlx::query_as!(
            Competition,
            "SELECT * FROM competitions 
             WHERE start_time <= NOW() AND end_time > NOW()
             ORDER BY end_time ASC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(competitions)
    }

    async fn get_upcoming_competitions(&self) -> Result<Vec<Competition>, Error> {
        let competitions = sqlx::query_as!(
            Competition,
            "SELECT * FROM competitions 
             WHERE start_time > NOW()
             ORDER BY start_time ASC
             LIMIT 10"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(competitions)
    }

    async fn get_competition_by_id(&self, competition_id: Uuid) -> Result<Option<Competition>, Error> {
        let competition = sqlx::query_as!(
            Competition,
            "SELECT * FROM competitions WHERE competition_id = $1",
            competition_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(competition)
    }

    async fn get_leaderboard(&self, competition_id: Uuid, limit: i32) -> Result<Vec<CompetitionParticipant>, Error> {
        let participants = sqlx::query_as!(
            CompetitionParticipant,
            "SELECT * FROM competition_participants 
             WHERE competition_id = $1
             ORDER BY score DESC
             LIMIT $2",
            competition_id,
            limit as i64
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(participants)
    }

    async fn update_participant_score(
        &self,
        competition_id: Uuid,
        user_id: Uuid,
        user_name: String,
        score: i32,
    ) -> Result<(), Error> {
        sqlx::query!(
            "INSERT INTO competition_participants (competition_id, user_id, user_name, score, last_updated)
             VALUES ($1, $2, $3, $4, NOW())
             ON CONFLICT (competition_id, user_id) 
             DO UPDATE SET score = $4, user_name = $3, last_updated = NOW()",
            competition_id,
            user_id,
            user_name,
            score
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete_old_competitions(&self, before: DateTime<Utc>) -> Result<u64, Error> {
        let result = sqlx::query!(
            "DELETE FROM competitions WHERE end_time < $1",
            before
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }
}
