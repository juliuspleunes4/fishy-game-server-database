# Backend Competition System Implementation Guide

This document contains everything needed to implement the competition/event system in the fishy-game-server-database backend. 

## Overview

We need to add a competition system where:
- Competitions are stored in the database (not in game server memory)
- Multiple game servers can share the same competitions
- Players can participate and their scores are tracked
- System auto-generates new competitions periodically
- Game servers poll the backend to get active/upcoming competitions

## 1. Database Schema (database-init.sql)

Add these tables to the end of database-init.sql:

```sql
-- Competition types: 1=MostFishCompetition, 2=MostItemsCompetition, 3=LargestFishCompetition
CREATE TABLE competitions (
    competition_id UUID PRIMARY KEY,
    competition_type INTEGER NOT NULL,
    target_fish_id INTEGER NOT NULL,
    start_time TIMESTAMPTZ NOT NULL,
    end_time TIMESTAMPTZ NOT NULL,
    reward_currency TEXT NOT NULL, -- 'coins' or 'bucks'
    prize_1st INTEGER NOT NULL,
    prize_2nd INTEGER NOT NULL,
    prize_3rd INTEGER NOT NULL,
    prize_4th INTEGER NOT NULL,
    prize_5th INTEGER NOT NULL,
    prize_6th INTEGER NOT NULL,
    prize_7th INTEGER NOT NULL,
    prize_8th INTEGER NOT NULL,
    prize_9th INTEGER NOT NULL,
    prize_10th INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT valid_competition_type CHECK (competition_type IN (1, 2, 3)),
    CONSTRAINT valid_currency CHECK (reward_currency IN ('coins', 'bucks')),
    CONSTRAINT valid_times CHECK (start_time < end_time)
);

CREATE TABLE competition_participants (
    competition_id UUID NOT NULL REFERENCES competitions(competition_id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(user_id),
    user_name TEXT NOT NULL,
    score INTEGER NOT NULL DEFAULT 0,
    last_updated TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (competition_id, user_id)
);

CREATE INDEX idx_competitions_time_range ON competitions(start_time, end_time);
CREATE INDEX idx_competitions_active ON competitions(start_time, end_time) WHERE end_time > NOW();
CREATE INDEX idx_competition_participants_score ON competition_participants(competition_id, score DESC);
```

## 2. Domain Models (src/domain.rs)

Add these structs to domain.rs:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Competition {
    pub competition_id: Uuid,
    pub competition_type: i32,
    pub target_fish_id: i32,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub reward_currency: String,
    pub prize_1st: i32,
    pub prize_2nd: i32,
    pub prize_3rd: i32,
    pub prize_4th: i32,
    pub prize_5th: i32,
    pub prize_6th: i32,
    pub prize_7th: i32,
    pub prize_8th: i32,
    pub prize_9th: i32,
    pub prize_10th: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct CompetitionParticipant {
    pub competition_id: Uuid,
    pub user_id: Uuid,
    pub user_name: String,
    pub score: i32,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CompetitionWithLeaderboard {
    #[serde(flatten)]
    pub competition: Competition,
    pub leaderboard: Vec<CompetitionParticipant>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateCompetitionRequest {
    pub competition_type: i32,
    pub target_fish_id: i32,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub reward_currency: String,
    pub prizes: Vec<i32>, // Must contain exactly 10 values for places 1-10
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateScoreRequest {
    pub competition_id: Uuid,
    pub user_id: Uuid,
    pub user_name: String,
    pub score: i32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GenerateCompetitionsRequest {
    pub count: i32, // Number of competitions to generate
}
```

## 3. Repository Layer (src/repository/competitions.rs)

Create new file src/repository/competitions.rs:

```rust
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
```

Add to src/repository/mod.rs:

```rust
pub mod competitions;
```

## 4. Service Layer (src/service/competitions.rs)

Create new file src/service/competitions.rs:

```rust
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
```

Add to src/service/mod.rs:

```rust
pub mod competitions;
```

## 5. Controller Layer (src/controller/competitions.rs)

Create new file src/controller/competitions.rs:

```rust
use crate::domain::{Competition, CompetitionWithLeaderboard, CreateCompetitionRequest, GenerateCompetitionsRequest, UpdateScoreRequest};
use crate::service::competitions::CompetitionsService;
use rocket::{get, post, routes, serde::json::Json, State, Route};
use std::sync::Arc;
use uuid::Uuid;

#[utoipa::path(
    post,
    path = "/competitions/create",
    request_body = CreateCompetitionRequest,
    responses(
        (status = 201, description = "Competition created successfully", body = Competition),
        (status = 400, description = "Invalid input data"),
        (status = 500, description = "Internal server error")
    ),
    description = "Creates a new competition",
    operation_id = "createCompetition",
    tag = "Competitions"
)]
#[post("/create", data = "<payload>")]
async fn create_competition(
    payload: Json<CreateCompetitionRequest>,
    competitions_service: &State<Arc<dyn CompetitionsService>>,
) -> Result<Json<Competition>, Json<String>> {
    match competitions_service.create_competition(payload.into_inner()).await {
        Ok(competition) => Ok(Json(competition)),
        Err(e) => Err(Json(format!("Failed to create competition: {}", e))),
    }
}

#[utoipa::path(
    get,
    path = "/competitions/active",
    responses(
        (status = 200, description = "Active competitions with leaderboards", body = Vec<CompetitionWithLeaderboard>),
        (status = 500, description = "Internal server error")
    ),
    description = "Gets all currently active competitions with their leaderboards",
    operation_id = "getActiveCompetitions",
    tag = "Competitions"
)]
#[get("/active")]
async fn get_active_competitions(
    competitions_service: &State<Arc<dyn CompetitionsService>>,
) -> Result<Json<Vec<CompetitionWithLeaderboard>>, Json<String>> {
    match competitions_service.get_active_competitions().await {
        Ok(competitions) => Ok(Json(competitions)),
        Err(e) => Err(Json(format!("Failed to get active competitions: {}", e))),
    }
}

#[utoipa::path(
    get,
    path = "/competitions/upcoming",
    responses(
        (status = 200, description = "Upcoming competitions", body = Vec<Competition>),
        (status = 500, description = "Internal server error")
    ),
    description = "Gets upcoming competitions that haven't started yet",
    operation_id = "getUpcomingCompetitions",
    tag = "Competitions"
)]
#[get("/upcoming")]
async fn get_upcoming_competitions(
    competitions_service: &State<Arc<dyn CompetitionsService>>,
) -> Result<Json<Vec<Competition>>, Json<String>> {
    match competitions_service.get_upcoming_competitions().await {
        Ok(competitions) => Ok(Json(competitions)),
        Err(e) => Err(Json(format!("Failed to get upcoming competitions: {}", e))),
    }
}

#[utoipa::path(
    get,
    path = "/competitions/{competition_id}",
    responses(
        (status = 200, description = "Competition with leaderboard", body = CompetitionWithLeaderboard),
        (status = 404, description = "Competition not found"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("competition_id" = Uuid, Path, description = "Competition UUID")
    ),
    description = "Gets a specific competition with its leaderboard",
    operation_id = "getCompetition",
    tag = "Competitions"
)]
#[get("/<competition_id>")]
async fn get_competition(
    competition_id: String,
    competitions_service: &State<Arc<dyn CompetitionsService>>,
) -> Result<Json<CompetitionWithLeaderboard>, Json<String>> {
    let uuid = Uuid::parse_str(&competition_id)
        .map_err(|_| Json("Invalid UUID format".to_string()))?;
    
    match competitions_service.get_competition_with_leaderboard(uuid).await {
        Ok(Some(competition)) => Ok(Json(competition)),
        Ok(None) => Err(Json("Competition not found".to_string())),
        Err(e) => Err(Json(format!("Failed to get competition: {}", e))),
    }
}

#[utoipa::path(
    post,
    path = "/competitions/update_score",
    request_body = UpdateScoreRequest,
    responses(
        (status = 200, description = "Score updated successfully", body = bool),
        (status = 400, description = "Invalid input or competition not active"),
        (status = 500, description = "Internal server error")
    ),
    description = "Updates a player's score in a competition",
    operation_id = "updateScore",
    tag = "Competitions"
)]
#[post("/update_score", data = "<payload>")]
async fn update_score(
    payload: Json<UpdateScoreRequest>,
    competitions_service: &State<Arc<dyn CompetitionsService>>,
) -> Json<bool> {
    match competitions_service.update_score(payload.into_inner()).await {
        Ok(()) => Json(true),
        Err(_) => Json(false),
    }
}

#[utoipa::path(
    post,
    path = "/competitions/generate",
    request_body = GenerateCompetitionsRequest,
    responses(
        (status = 201, description = "Competitions generated successfully", body = Vec<Competition>),
        (status = 400, description = "Invalid input data"),
        (status = 500, description = "Internal server error")
    ),
    description = "Generates random competitions automatically",
    operation_id = "generateCompetitions",
    tag = "Competitions"
)]
#[post("/generate", data = "<payload>")]
async fn generate_competitions(
    payload: Json<GenerateCompetitionsRequest>,
    competitions_service: &State<Arc<dyn CompetitionsService>>,
) -> Result<Json<Vec<Competition>>, Json<String>> {
    if payload.count < 1 || payload.count > 10 {
        return Err(Json("Count must be between 1 and 10".to_string()));
    }
    
    match competitions_service.generate_competitions(payload.into_inner()).await {
        Ok(competitions) => Ok(Json(competitions)),
        Err(e) => Err(Json(format!("Failed to generate competitions: {}", e))),
    }
}

#[utoipa::path(
    post,
    path = "/competitions/cleanup/{days_old}",
    responses(
        (status = 200, description = "Old competitions cleaned up", body = u64),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("days_old" = i64, Path, description = "Delete competitions older than this many days")
    ),
    description = "Deletes old completed competitions",
    operation_id = "cleanupCompetitions",
    tag = "Competitions"
)]
#[post("/cleanup/<days_old>")]
async fn cleanup_competitions(
    days_old: i64,
    competitions_service: &State<Arc<dyn CompetitionsService>>,
) -> Result<Json<u64>, Json<String>> {
    match competitions_service.cleanup_old_competitions(days_old).await {
        Ok(deleted) => Ok(Json(deleted)),
        Err(e) => Err(Json(format!("Failed to cleanup: {}", e))),
    }
}

pub fn competitions_routes() -> Vec<Route> {
    routes![
        create_competition,
        get_active_competitions,
        get_upcoming_competitions,
        get_competition,
        update_score,
        generate_competitions,
        cleanup_competitions
    ]
}
```

Add to src/controller/mod.rs:

```rust
pub mod competitions;
```

## 6. Update main.rs

Add these imports to the top of main.rs:

```rust
use crate::controller::competitions::competitions_routes;
use crate::repository::competitions::CompetitionsRepositoryImpl;
use crate::service::competitions::CompetitionsService;
use crate::service::competitions::CompetitionsServiceImpl;
```

Add to the repository initialization section (around line 135):

```rust
let competitions_repository = CompetitionsRepositoryImpl::new(pool.clone());
```

Add to the service initialization section (around line 165):

```rust
let competitions_service: Arc<dyn CompetitionsService> = Arc::new(
    CompetitionsServiceImpl::new(competitions_repository.clone())
);
```

Add to the .manage() chain (around line 185):

```rust
.manage(competitions_service)
```

Add to the .mount() chain (around line 200):

```rust
.mount("/competitions", competitions_routes())
```

## 7. Update docs.rs

Add to the imports:

```rust
use crate::controller::competitions::*;
```

Add to the openapi paths list:

```rust
create_competition,
get_active_competitions,
get_upcoming_competitions,
get_competition,
update_score,
generate_competitions,
cleanup_competitions,
```

## 8. Add rand dependency to Cargo.toml

Add this line to the [dependencies] section:

```toml
rand = "0.8"
```

## 9. Testing the API

After implementation, you can test with these curl commands:

Generate 3 competitions:
```bash
curl -X POST http://localhost:8000/competitions/generate \
  -H "Content-Type: application/json" \
  -d '{"count": 3}'
```

Get active competitions:
```bash
curl http://localhost:8000/competitions/active
```

Get upcoming competitions:
```bash
curl http://localhost:8000/competitions/upcoming
```

Update a player score:
```bash
curl -X POST http://localhost:8000/competitions/update_score \
  -H "Content-Type: application/json" \
  -d '{
    "competition_id": "UUID_HERE",
    "user_id": "USER_UUID",
    "user_name": "PlayerName",
    "score": 42
  }'
```

## 10. Unity Integration Notes

After backend is ready, the Unity game server needs to:

1. Poll `/competitions/active` and `/competitions/upcoming` every 60 seconds
2. Send score updates to `/competitions/update_score` when players progress
3. Remove the local event generation from CompetitionManager
4. Fetch leaderboards from the API instead of tracking locally

Competition types mapping:
- Type 1 = MostFishCompetition (catch most of specific fish)
- Type 2 = MostItemsCompetition (collect most of specific item)
- Type 3 = LargestFishCompetition (catch biggest fish by length)

The game server should store the backend URL in DatabaseEndpoints and add:
```csharp
public static string getActiveCompetitionsEndpoint = serverAddress + "competitions/active";
public static string getUpcomingCompetitionsEndpoint = serverAddress + "competitions/upcoming";
public static string updateCompetitionScoreEndpoint = serverAddress + "competitions/update_score";
```
