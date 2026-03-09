use chrono::Utc;
use rand::Rng;
use rand::seq::SliceRandom;
use rocket::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    domain::{Competition, LeaderboardResponse, SubmitScoreRequest},
    repository::competitions::{CompetitionsRepository, CompetitionsRepositoryImpl},
};

/// Competition type enum for type safety
#[derive(Debug, Clone, Copy)]
enum CompetitionType {
    MostFish = 1,
    LargestFish = 2,
    MostItems = 3,
}

impl CompetitionType {
    /// Convert enum to string for JSON serialization (Unity expects strings)
    fn to_string(&self) -> String {
        match self {
            CompetitionType::MostFish => "MostFish".to_string(),
            CompetitionType::LargestFish => "LargestFish".to_string(),
            CompetitionType::MostItems => "MostItems".to_string(),
        }
    }
    
    /// Convert enum to i32 for database storage
    fn to_i32(&self) -> i32 {
        *self as i32
    }
}

/// Currency enum for type safety
#[derive(Debug, Clone, Copy)]
enum Currency {
    Coins,
    Bucks,
}

impl Currency {
    /// Convert enum to string for database storage and JSON serialization
    fn to_string(&self) -> String {
        match self {
            Currency::Coins => "COINS".to_string(),
            Currency::Bucks => "BUCKS".to_string(),
        }
    }
}

/// Predefined competition template to ensure catchable and interesting competitions
#[derive(Clone)]
struct CompetitionTemplate {
    competition_type: CompetitionType,
    target_fish_id: i32,
    description: &'static str,
}

/// Get the predefined list of possible competitions
/// These are hand-picked to ensure fish are catchable and competitions are interesting
fn get_competition_templates() -> Vec<CompetitionTemplate> {
    vec![
        // MostFish competitions
        CompetitionTemplate { competition_type: CompetitionType::MostFish, target_fish_id: 1, description: "Catch the most Common Fish" },
        CompetitionTemplate { competition_type: CompetitionType::MostFish, target_fish_id: 5, description: "Catch the most Trout" },
        CompetitionTemplate { competition_type: CompetitionType::MostFish, target_fish_id: 10, description: "Catch the most Bass" },
        CompetitionTemplate { competition_type: CompetitionType::MostFish, target_fish_id: 15, description: "Catch the most Salmon" },
        CompetitionTemplate { competition_type: CompetitionType::MostFish, target_fish_id: 20, description: "Catch the most Catfish" },
        CompetitionTemplate { competition_type: CompetitionType::MostFish, target_fish_id: 25, description: "Catch the most Pike" },
        CompetitionTemplate { competition_type: CompetitionType::MostFish, target_fish_id: 0, description: "Catch the most fish (any type)" },
        
        // LargestFish competitions
        CompetitionTemplate { competition_type: CompetitionType::LargestFish, target_fish_id: 5, description: "Catch the largest Trout" },
        CompetitionTemplate { competition_type: CompetitionType::LargestFish, target_fish_id: 10, description: "Catch the largest Bass" },
        CompetitionTemplate { competition_type: CompetitionType::LargestFish, target_fish_id: 15, description: "Catch the largest Salmon" },
        CompetitionTemplate { competition_type: CompetitionType::LargestFish, target_fish_id: 30, description: "Catch the largest Tuna" },
        CompetitionTemplate { competition_type: CompetitionType::LargestFish, target_fish_id: 35, description: "Catch the largest Marlin" },
        CompetitionTemplate { competition_type: CompetitionType::LargestFish, target_fish_id: 0, description: "Catch the largest fish (any type)" },
        
        // MostItems competitions
        CompetitionTemplate { competition_type: CompetitionType::MostItems, target_fish_id: 50, description: "Collect the most special items" },
        CompetitionTemplate { competition_type: CompetitionType::MostItems, target_fish_id: 55, description: "Collect the most treasure chests" },
        CompetitionTemplate { competition_type: CompetitionType::MostItems, target_fish_id: 60, description: "Collect the most rare lures" },
    ]
}

/// Calculate prize pool based on competition duration and winner count
/// Longer competitions get bigger total prize pools
fn calculate_prize_pool(duration_hours: i64, winner_count: usize, currency: Currency) -> Vec<i32> {
    // Base prize pool multiplier based on duration
    let duration_multiplier = if duration_hours <= 12 {
        1.0
    } else if duration_hours <= 24 {
        1.5
    } else if duration_hours <= 36 {
        2.0
    } else {
        2.5  // 36-48 hours
    };
    
    // Calculate total prize pool
    let total_prize = match currency {
        Currency::Coins => (200.0 * duration_multiplier) as i32,  // 200-500 COINS total
        Currency::Bucks => (4000.0 * duration_multiplier) as i32, // 4k-10k BUCKS total
    };
    
    // Distribute prizes logarithmically (first place gets most, decreases logarithmically)
    let mut prizes = Vec::with_capacity(winner_count);
    let mut remaining_pool = total_prize as f64;
    
    for rank in 1..=winner_count {
        // Logarithmic decay: each rank gets less than the previous
        // Formula: prize = remaining * (1 / (1 + log2(rank)))
        let rank_weight = 1.0 / (1.0 + (rank as f64).log2());
        let prize = (remaining_pool * rank_weight) as i32;
        
        // Ensure minimum prize of 1
        let prize = prize.max(1);
        prizes.push(prize);
        
        // Reduce remaining pool for next rank
        remaining_pool -= prize as f64;
        
        // Stop if we've exhausted the pool
        if remaining_pool <= 0.0 {
            break;
        }
    }
    
    prizes
}

/// Calculate number of winners based on competition duration
/// Longer competitions have more winners (10-50 range, weighted by duration)
fn calculate_winner_count(duration_hours: i64) -> usize {
    let mut rng = rand::thread_rng();
    
    if duration_hours <= 12 {
        // Short competitions: 10-20 winners
        rng.gen_range(10..=20)
    } else if duration_hours <= 24 {
        // Medium competitions: 20-35 winners
        rng.gen_range(20..=35)
    } else if duration_hours <= 36 {
        // Long competitions: 30-45 winners
        rng.gen_range(30..=45)
    } else {
        // Very long competitions: 40-50 winners
        rng.gen_range(40..=50)
    }
}

/// Service layer for business logic related to competitions
#[async_trait]
pub trait CompetitionsService: Send + Sync {
    async fn get_active_competition(&self) -> Result<Option<Competition>, sqlx::Error>;
    
    async fn get_upcoming_competitions(&self) -> Result<Vec<Competition>, sqlx::Error>;
    
    async fn get_competition_by_id(&self, competition_id: Uuid) -> Result<Option<Competition>, sqlx::Error>;
    
    async fn get_leaderboard(&self, competition_id: Uuid) -> Result<LeaderboardResponse, sqlx::Error>;
    
    async fn submit_score(&self, request: SubmitScoreRequest) -> Result<(), sqlx::Error>;
    
    async fn generate_competitions_if_needed(&self) -> Result<Vec<Competition>, sqlx::Error>;
}

pub struct CompetitionsServiceImpl<T: CompetitionsRepository> {
    pool: PgPool,
    competitions_repository: T,
}

impl<R: CompetitionsRepository> CompetitionsServiceImpl<R> {
    pub fn new(pool: PgPool, competitions_repository: R) -> Self {
        Self {
            pool,
            competitions_repository,
        }
    }
}

#[async_trait]
impl<R: CompetitionsRepository> CompetitionsService for CompetitionsServiceImpl<R> {
    async fn get_active_competition(&self) -> Result<Option<Competition>, sqlx::Error> {
        // Get all active competitions and return the first one (should only be one)
        let competitions = self.competitions_repository.get_active_competitions().await?;
        Ok(competitions.into_iter().next())
    }

    async fn get_upcoming_competitions(&self) -> Result<Vec<Competition>, sqlx::Error> {
        self.competitions_repository
            .get_upcoming_competitions()
            .await
    }

    async fn get_competition_by_id(&self, competition_id: Uuid) -> Result<Option<Competition>, sqlx::Error> {
        self.competitions_repository
            .get_competition_by_id(competition_id)
            .await
    }

    async fn get_leaderboard(&self, competition_id: Uuid) -> Result<LeaderboardResponse, sqlx::Error> {
        let results = self
            .competitions_repository
            .get_competition_results(competition_id)
            .await?;

        Ok(LeaderboardResponse {
            competition_id,
            results,
        })
    }

    async fn submit_score(&self, request: SubmitScoreRequest) -> Result<(), sqlx::Error> {
        // Validate that the competition exists and is active
        let competition = self
            .competitions_repository
            .get_competition_by_id(request.competition_id)
            .await?;

        if competition.is_none() {
            return Err(sqlx::Error::RowNotFound);
        }

        let competition = competition.unwrap();
        if competition.status != "ACTIVE" {
            return Err(sqlx::Error::Protocol(
                "Competition is not active".into(),
            ));
        }

        // Submit the score
        self.competitions_repository
            .submit_score(request.competition_id, request.player_id, request.score)
            .await
    }

    async fn generate_competitions_if_needed(&self) -> Result<Vec<Competition>, sqlx::Error> {
        // Start a transaction for atomic competition generation
        let mut tx = self.pool.begin().await?;
        
        // Count scheduled and active competitions within the transaction
        let scheduled_count = CompetitionsRepositoryImpl::count_competitions_by_status_tx(
            &mut tx,
            "SCHEDULED".to_string()
        ).await?;
        
        let active_count = CompetitionsRepositoryImpl::count_competitions_by_status_tx(
            &mut tx,
            "ACTIVE".to_string()
        ).await?;

        let total_count = scheduled_count + active_count;

        // If we have 10 or more competitions, no need to generate new ones
        if total_count >= 10 {
            // Rollback the transaction (no changes made)
            tx.rollback().await?;
            return Ok(vec![]);
        }

        let needed = 10 - total_count;

        // Get the latest competition's end time to schedule after it
        let upcoming = CompetitionsRepositoryImpl::get_upcoming_competitions_tx(&mut tx).await?;
        let active = CompetitionsRepositoryImpl::get_active_competitions_tx(&mut tx).await?;

        let last_end_time = upcoming
            .iter()
            .chain(active.iter())
            .map(|c| c.end_time)
            .max()
            .unwrap_or_else(Utc::now);

        let mut next_start_time = if last_end_time > Utc::now() {
            last_end_time
        } else {
            Utc::now()
        };

        // Generate all random parameters in a separate scope before any await calls
        let random_params: Vec<(i64, i64, CompetitionType, i32, Currency, Vec<i32>)> = {
            let mut rng = rand::thread_rng();
            let templates = get_competition_templates();
            
            (0..needed)
                .map(|_| {
                    let gap_hours = rng.gen_range(6..=12);
                    let duration_hours = rng.gen_range(12..=48);
                    
                    // Pick a random template from the predefined list
                    let template = templates.choose(&mut rng)
                        .expect("Competition templates should not be empty");
                    
                    let competition_type = template.competition_type;
                    let target_fish_id = template.target_fish_id;
                    
                    // 70% coins, 30% bucks
                    let reward_currency = if rng.gen_bool(0.7) {
                        Currency::Coins
                    } else {
                        Currency::Bucks
                    };
                    
                    // Calculate winner count based on duration
                    let winner_count = calculate_winner_count(duration_hours);
                    
                    // Calculate prize pool based on duration and winner count
                    let prize_pool = calculate_prize_pool(duration_hours, winner_count, reward_currency);

                    (gap_hours, duration_hours, competition_type, target_fish_id, reward_currency, prize_pool)
                })
                .collect()
        };

        let mut new_competitions = vec![];

        // Create all competitions within the transaction
        for (gap_hours, duration_hours, competition_type, target_fish_id, reward_currency, prize_pool) in random_params {
            next_start_time = next_start_time + chrono::Duration::hours(gap_hours);
            let end_time = next_start_time + chrono::Duration::hours(duration_hours);

            let competition_id = Uuid::new_v4();

            // Use transaction method instead of repository method
            CompetitionsRepositoryImpl::create_competition_tx(
                &mut tx,
                competition_id,
                competition_type.to_i32(),
                target_fish_id,
                next_start_time,
                end_time,
                reward_currency.to_string(),
                prize_pool.clone(),
            )
            .await?;

            new_competitions.push(Competition {
                competition_id,
                competition_type: competition_type.to_string(),
                target_fish_id,
                start_time: next_start_time,
                end_time,
                reward_currency: reward_currency.to_string(),
                prize_pool,
                created_at: Utc::now(),
                status: "SCHEDULED".to_string(),
            });

            // Next competition starts after this one ends
            next_start_time = end_time;
        }

        // Commit the transaction - all competitions created atomically
        tx.commit().await?;

        Ok(new_competitions)
    }
}
