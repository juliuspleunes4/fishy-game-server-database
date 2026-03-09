use chrono::Utc;
use rand::Rng;
use rocket::async_trait;
use uuid::Uuid;

use crate::{
    domain::{Competition, LeaderboardResponse, SubmitScoreRequest},
    repository::competitions::CompetitionsRepository,
};

/// Service layer for business logic related to competitions
#[async_trait]
pub trait CompetitionsService: Send + Sync {
    async fn get_active_competitions(&self) -> Result<Vec<Competition>, sqlx::Error>;
    
    async fn get_upcoming_competitions(&self) -> Result<Vec<Competition>, sqlx::Error>;
    
    async fn get_competition_by_id(&self, competition_id: Uuid) -> Result<Option<Competition>, sqlx::Error>;
    
    async fn get_leaderboard(&self, competition_id: Uuid) -> Result<LeaderboardResponse, sqlx::Error>;
    
    async fn submit_score(&self, request: SubmitScoreRequest) -> Result<(), sqlx::Error>;
    
    async fn generate_competitions_if_needed(&self) -> Result<Vec<Competition>, sqlx::Error>;
}

pub struct CompetitionsServiceImpl<T: CompetitionsRepository> {
    competitions_repository: T,
}

impl<R: CompetitionsRepository> CompetitionsServiceImpl<R> {
    pub fn new(competitions_repository: R) -> Self {
        Self {
            competitions_repository,
        }
    }
}

#[async_trait]
impl<R: CompetitionsRepository> CompetitionsService for CompetitionsServiceImpl<R> {
    async fn get_active_competitions(&self) -> Result<Vec<Competition>, sqlx::Error> {
        self.competitions_repository.get_active_competitions().await
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
        // Count scheduled and active competitions
        let scheduled_count = self
            .competitions_repository
            .count_competitions_by_status("SCHEDULED".to_string())
            .await?;
        let active_count = self
            .competitions_repository
            .count_competitions_by_status("ACTIVE".to_string())
            .await?;

        let total_count = scheduled_count + active_count;

        // If we have 3 or more competitions, no need to generate new ones
        if total_count >= 3 {
            return Ok(vec![]);
        }

        let needed = 3 - total_count;

        // Get the latest competition's end time to schedule after it
        let upcoming = self
            .competitions_repository
            .get_upcoming_competitions()
            .await?;
        let active = self
            .competitions_repository
            .get_active_competitions()
            .await?;

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
        let random_params: Vec<(i64, i64, i32, i32, String, Vec<i32>)> = {
            let mut rng = rand::thread_rng();
            (0..needed)
                .map(|_| {
                    let gap_hours = rng.gen_range(6..=12);
                    let duration_hours = rng.gen_range(12..=48);
                    let competition_type = rng.gen_range(1..=3);
                    let target_fish_id = rng.gen_range(1..=100);
                    let reward_currency = if rng.gen_bool(0.7) {
                        "COINS".to_string()
                    } else {
                        "BUCKS".to_string()
                    };
                    
                    let prize_pool = if reward_currency == "COINS" {
                        vec![
                            rng.gen_range(800..=1200),   // 1st
                            rng.gen_range(600..=900),    // 2nd
                            rng.gen_range(400..=600),    // 3rd
                            rng.gen_range(300..=450),    // 4th
                            rng.gen_range(200..=300),    // 5th
                            rng.gen_range(150..=225),    // 6th
                            rng.gen_range(100..=150),    // 7th
                            rng.gen_range(75..=112),     // 8th
                            rng.gen_range(50..=75),      // 9th
                            rng.gen_range(20..=30),      // 10th
                        ]
                    } else {
                        vec![
                            rng.gen_range(80..=120),     // 1st
                            rng.gen_range(60..=90),      // 2nd
                            rng.gen_range(40..=60),      // 3rd
                            rng.gen_range(30..=45),      // 4th
                            rng.gen_range(20..=30),      // 5th
                            rng.gen_range(15..=22),      // 6th
                            rng.gen_range(10..=15),      // 7th
                            rng.gen_range(7..=11),       // 8th
                            rng.gen_range(5..=7),        // 9th
                            rng.gen_range(2..=4),        // 10th
                        ]
                    };

                    (gap_hours, duration_hours, competition_type, target_fish_id, reward_currency, prize_pool)
                })
                .collect()
        };

        let mut new_competitions = vec![];

        for (gap_hours, duration_hours, competition_type, target_fish_id, reward_currency, prize_pool) in random_params {
            next_start_time = next_start_time + chrono::Duration::hours(gap_hours);
            let end_time = next_start_time + chrono::Duration::hours(duration_hours);

            let competition_id = Uuid::new_v4();

            self.competitions_repository
                .create_competition(
                    competition_id,
                    competition_type,
                    target_fish_id,
                    next_start_time,
                    end_time,
                    reward_currency.clone(),
                    prize_pool.clone(),
                )
                .await?;

            new_competitions.push(Competition {
                competition_id,
                competition_type,
                target_fish_id,
                start_time: next_start_time,
                end_time,
                reward_currency,
                prize_pool,
                created_at: Utc::now(),
                status: "SCHEDULED".to_string(),
            });

            // Next competition starts after this one ends
            next_start_time = end_time;
        }

        Ok(new_competitions)
    }
}
