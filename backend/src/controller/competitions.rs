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
