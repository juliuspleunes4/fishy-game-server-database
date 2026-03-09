// TODO: These queries use sqlx::query! macros for compile-time SQL validation.
// If you see "set DATABASE_URL" errors and don't have database access, this is expected.
// Project owner should run `cargo sqlx prepare` to generate .sqlx/ cache and commit it,
// then all contributors can compile without DATABASE_URL while keeping type safety.
use crate::{domain::{Competition, CompetitionResult}, entity::{competition_results, competitions}};
use chrono::{DateTime, Utc};
use rocket::async_trait;
use sea_orm::{
    ActiveValue::Set, ColumnTrait, DatabaseTransaction, DbErr, EntityTrait, Order, QueryFilter,
    QueryOrder, ActiveModelTrait, QuerySelect, PaginatorTrait,
};
use sea_orm::sea_query::OnConflict;
use uuid::Uuid;

#[async_trait]
pub trait CompetitionsRepository: Send + Sync {
    /// Get all active competitions (status = 'ACTIVE')
    async fn get_active_competitions(&self, tx: &DatabaseTransaction) -> Result<Vec<Competition>, DbErr>;
    
    /// Get all upcoming competitions (status = 'SCHEDULED')
    async fn get_upcoming_competitions(&self, tx: &DatabaseTransaction) -> Result<Vec<Competition>, DbErr>;
    
    /// Get a specific competition by ID
    async fn get_competition_by_id(&self, tx: &DatabaseTransaction, competition_id: Uuid) -> Result<Option<Competition>, DbErr>;
    
    /// Get leaderboard for a specific competition
    async fn get_competition_results(&self, tx: &DatabaseTransaction, competition_id: Uuid) -> Result<Vec<CompetitionResult>, DbErr>;
    
    /// Submit or update a player's score for a competition
    async fn submit_score(&self, tx: &DatabaseTransaction, competition_id: Uuid, player_id: Uuid, score: i32) -> Result<(), DbErr>;
    
    /// Create a new competition
    async fn create_competition(
        &self,
        tx: &DatabaseTransaction,
        competition_id: Uuid,
        competition_type: i32,
        target_fish_id: i32,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        reward_currency: String,
        prize_pool: Vec<i32>,
    ) -> Result<(), DbErr>;
    
    /// Update competition status
    async fn update_competition_status(&self, tx: &DatabaseTransaction, competition_id: Uuid, status: String) -> Result<(), DbErr>;
    
    /// Count competitions by status
    async fn count_competitions_by_status(&self, tx: &DatabaseTransaction, status: String) -> Result<i64, DbErr>;
}

#[derive(Debug, Clone)]
pub struct CompetitionsRepositoryImpl;

impl CompetitionsRepositoryImpl {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl CompetitionsRepository for CompetitionsRepositoryImpl {
    async fn get_active_competitions(&self, tx: &DatabaseTransaction) -> Result<Vec<Competition>, DbErr> {
        let models = competitions::Entity::find()
            .filter(competitions::Column::Status.eq("ACTIVE"))
            .order_by(competitions::Column::StartTime, Order::Asc)
            .all(tx)
            .await?;

        Ok(models
            .into_iter()
            .map(|m| Competition {
                competition_id: m.competition_id,
                competition_type: convert_type_to_string(m.competition_type),
                target_fish_id: m.target_fish_id,
                start_time: m.start_time.with_timezone(&Utc),
                end_time: m.end_time.with_timezone(&Utc),
                reward_currency: m.reward_currency,
                prize_pool: m.prize_pool,
                created_at: m.created_at.with_timezone(&Utc),
                status: m.status,
            })
            .collect())
    }

    async fn get_upcoming_competitions(&self, tx: &DatabaseTransaction) -> Result<Vec<Competition>, DbErr> {
        let models = competitions::Entity::find()
            .filter(competitions::Column::Status.eq("SCHEDULED"))
            .order_by(competitions::Column::StartTime, Order::Asc)
            .all(tx)
            .await?;

        Ok(models
            .into_iter()
            .map(|m| Competition {
                competition_id: m.competition_id,
                competition_type: convert_type_to_string(m.competition_type),
                target_fish_id: m.target_fish_id,
                start_time: m.start_time.with_timezone(&Utc),
                end_time: m.end_time.with_timezone(&Utc),
                reward_currency: m.reward_currency,
                prize_pool: m.prize_pool,
                created_at: m.created_at.with_timezone(&Utc),
                status: m.status,
            })
            .collect())
    }

    async fn get_competition_by_id(&self, tx: &DatabaseTransaction, competition_id: Uuid) -> Result<Option<Competition>, DbErr> {
        let model = competitions::Entity::find_by_id(competition_id).one(tx).await?;

        Ok(model.map(|m| Competition {
            competition_id: m.competition_id,
            competition_type: convert_type_to_string(m.competition_type),
            target_fish_id: m.target_fish_id,
            start_time: m.start_time.with_timezone(&Utc),
            end_time: m.end_time.with_timezone(&Utc),
            reward_currency: m.reward_currency,
            prize_pool: m.prize_pool,
            created_at: m.created_at.with_timezone(&Utc),
            status: m.status,
        }))
    }

    async fn get_competition_results(&self, tx: &DatabaseTransaction, competition_id: Uuid) -> Result<Vec<CompetitionResult>, DbErr> {
        let models = competition_results::Entity::find()
            .filter(competition_results::Column::CompetitionId.eq(competition_id))
            .order_by(competition_results::Column::Score, Order::Desc)
            .order_by(competition_results::Column::LastUpdated, Order::Asc)
            .limit(100)
            .all(tx)
            .await?;

        Ok(models
            .into_iter()
            .map(|m| CompetitionResult {
                result_id: m.result_id,
                competition_id: m.competition_id,
                player_id: m.player_id,
                score: m.score,
                last_updated: m.last_updated.with_timezone(&Utc),
            })
            .collect())
    }

    async fn submit_score(&self, tx: &DatabaseTransaction, competition_id: Uuid, player_id: Uuid, score: i32) -> Result<(), DbErr> {
        let now = chrono::Utc::now();
        
        competition_results::Entity::insert(competition_results::ActiveModel {
            result_id: Set(Uuid::new_v4()),
            competition_id: Set(competition_id),
            player_id: Set(player_id),
            score: Set(score),
            last_updated: Set(now.fixed_offset()),
        })
        .on_conflict(
            OnConflict::columns([
                competition_results::Column::CompetitionId,
                competition_results::Column::PlayerId,
            ])
            .update_columns([
                competition_results::Column::Score,
                competition_results::Column::LastUpdated,
            ])
            .to_owned()
        )
        .exec(tx)
        .await?;

        Ok(())
    }

    async fn create_competition(
        &self,
        tx: &DatabaseTransaction,
        competition_id: Uuid,
        competition_type: i32,
        target_fish_id: i32,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        reward_currency: String,
        prize_pool: Vec<i32>,
    ) -> Result<(), DbErr> {
        let now = chrono::Utc::now();

        competitions::ActiveModel {
            competition_id: Set(competition_id),
            competition_type: Set(competition_type),
            target_fish_id: Set(target_fish_id),
            start_time: Set(start_time.fixed_offset()),
            end_time: Set(end_time.fixed_offset()),
            reward_currency: Set(reward_currency),
            prize_pool: Set(prize_pool),
            created_at: Set(now.fixed_offset()),
            status: Set("SCHEDULED".to_string()),
        }
        .insert(tx)
        .await?;

        Ok(())
    }

    async fn update_competition_status(&self, tx: &DatabaseTransaction, competition_id: Uuid, status: String) -> Result<(), DbErr> {
        let competition = competitions::Entity::find_by_id(competition_id)
            .one(tx)
            .await?
            .ok_or(DbErr::RecordNotFound("Competition not found".to_string()))?;

        let mut active: competitions::ActiveModel = competition.into();
        active.status = Set(status);
        active.update(tx).await?;

        Ok(())
    }

    async fn count_competitions_by_status(&self, tx: &DatabaseTransaction, status: String) -> Result<i64, DbErr> {
        let count = competitions::Entity::find()
            .filter(competitions::Column::Status.eq(status))
            .count(tx)
            .await?;

        Ok(count as i64)
    }
}

// Helper function to convert competition type integer to string
fn convert_type_to_string(competition_type: i32) -> String {
    match competition_type {
        1 => "MostFish".to_string(),
        2 => "LargestFish".to_string(),
        3 => "MostItems".to_string(),
        _ => format!("Unknown({})", competition_type),
    }
}
