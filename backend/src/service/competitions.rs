use crate::domain::{Competition, CompetitionWithLeaderboard, CreateCompetitionRequest, GenerateCompetitionsRequest, UpdateScoreRequest};
use crate::repository::competitions::CompetitionsRepository;
use chrono::{DateTime, Duration, Utc};
use rocket::async_trait;
use sqlx::Error;
use uuid::Uuid;
use rand::Rng;

#[async_trait]
pub trait CompetitionsService: Send + Sync {
    async fn create_competition(&self, request: CreateCompetitionRequest) -> Result<Competition, Error>;
    async fn get_active_competitions(&self) -> Result<Vec<CompetitionWithLeaderboard>, Error>;
    async fn get_upcoming_competitions(&self) -> Result<Vec<Competition>, Error>;
    async fn get_competition_with_leaderboard(&self, competition_id: Uuid) -> Result<Option<CompetitionWithLeaderboard>, Error>;
    async fn update_score(&self, request: UpdateScoreRequest) -> Result<(), Error>;
    async fn generate_competitions(&self, request: GenerateCompetitionsRequest) -> Result<Vec<Competition>, Error>;
    async fn cleanup_old_competitions(&self, days_old: i64) -> Result<u64, Error>;
}

pub struct CompetitionsServiceImpl<T: CompetitionsRepository> {
    competitions_repository: T,
}

impl<R: CompetitionsRepository> CompetitionsServiceImpl<R> {
    pub fn new(competitions_repository: R) -> Self {
        Self { competitions_repository }
    }
}

#[async_trait]
impl<R: CompetitionsRepository> CompetitionsService for CompetitionsServiceImpl<R> {
    async fn create_competition(&self, request: CreateCompetitionRequest) -> Result<Competition, Error> {
        self.competitions_repository.create_competition(request).await
    }

    async fn get_active_competitions(&self) -> Result<Vec<CompetitionWithLeaderboard>, Error> {
        let competitions = self.competitions_repository.get_active_competitions().await?;
        let mut result = Vec::new();

        for competition in competitions {
            let leaderboard = self.competitions_repository
                .get_leaderboard(competition.competition_id, 100)
                .await?;
            
            result.push(CompetitionWithLeaderboard {
                competition,
                leaderboard,
            });
        }

        Ok(result)
    }

    async fn get_upcoming_competitions(&self) -> Result<Vec<Competition>, Error> {
        self.competitions_repository.get_upcoming_competitions().await
    }

    async fn get_competition_with_leaderboard(&self, competition_id: Uuid) -> Result<Option<CompetitionWithLeaderboard>, Error> {
        if let Some(competition) = self.competitions_repository.get_competition_by_id(competition_id).await? {
            let leaderboard = self.competitions_repository
                .get_leaderboard(competition_id, 100)
                .await?;
            
            Ok(Some(CompetitionWithLeaderboard {
                competition,
                leaderboard,
            }))
        } else {
            Ok(None)
        }
    }

    async fn update_score(&self, request: UpdateScoreRequest) -> Result<(), Error> {
        // Verify competition exists and is active
        if let Some(competition) = self.competitions_repository.get_competition_by_id(request.competition_id).await? {
            let now = Utc::now();
            if competition.start_time <= now && competition.end_time > now {
                self.competitions_repository
                    .update_participant_score(
                        request.competition_id,
                        request.user_id,
                        request.user_name,
                        request.score,
                    )
                    .await?;
                Ok(())
            } else {
                Err(Error::Protocol("Competition is not currently active".into()))
            }
        } else {
            Err(Error::RowNotFound)
        }
    }

    async fn generate_competitions(&self, request: GenerateCompetitionsRequest) -> Result<Vec<Competition>, Error> {
        let mut rng = rand::thread_rng();
        let mut generated = Vec::new();

        // Get the latest upcoming competition to schedule after it
        let existing_competitions = self.competitions_repository.get_upcoming_competitions().await?;
        let mut next_start_time = if let Some(last) = existing_competitions.last() {
            last.end_time + Duration::hours(rng.gen_range(6..13))
        } else {
            Utc::now() + Duration::hours(rng.gen_range(2..6))
        };

        for _ in 0..request.count {
            // Random competition type (1, 2, or 3)
            let competition_type = rng.gen_range(1..=3);
            
            // Random fish ID (1-100, adjust based on your game's fish count)
            let target_fish_id = rng.gen_range(1..=100);
            
            // Event duration: 12-48 hours
            let duration_hours = rng.gen_range(12..49);
            let end_time = next_start_time + Duration::hours(duration_hours);
            
            // 70% coins, 30% bucks
            let reward_currency = if rng.gen_bool(0.7) {
                "coins".to_string()
            } else {
                "bucks".to_string()
            };
            
            // Generate prize distribution
            let prizes = if reward_currency == "coins" {
                vec![
                    rng.gen_range(800..1201),   // 1st
                    rng.gen_range(500..801),    // 2nd
                    rng.gen_range(300..501),    // 3rd
                    rng.gen_range(200..301),    // 4th
                    rng.gen_range(150..201),    // 5th
                    rng.gen_range(100..151),    // 6th
                    rng.gen_range(75..101),     // 7th
                    rng.gen_range(50..76),      // 8th
                    rng.gen_range(30..51),      // 9th
                    rng.gen_range(20..31),      // 10th
                ]
            } else {
                vec![
                    rng.gen_range(80..121),     // 1st
                    rng.gen_range(50..81),      // 2nd
                    rng.gen_range(30..51),      // 3rd
                    rng.gen_range(20..31),      // 4th
                    rng.gen_range(15..21),      // 5th
                    rng.gen_range(10..16),      // 6th
                    rng.gen_range(8..11),       // 7th
                    rng.gen_range(6..9),        // 8th
                    rng.gen_range(4..7),        // 9th
                    rng.gen_range(2..5),        // 10th
                ]
            };

            let competition = self.create_competition(CreateCompetitionRequest {
                competition_type,
                target_fish_id,
                start_time: next_start_time,
                end_time,
                reward_currency,
                prizes,
            }).await?;

            generated.push(competition);

            // Schedule next competition 6-12 hours after this one ends
            next_start_time = end_time + Duration::hours(rng.gen_range(6..13));
        }

        Ok(generated)
    }

    async fn cleanup_old_competitions(&self, days_old: i64) -> Result<u64, Error> {
        let cutoff = Utc::now() - Duration::days(days_old);
        self.competitions_repository.delete_old_competitions(cutoff).await
    }
}
