use crate::{
    domain::{Competition, LeaderboardResponse, SubmitScoreRequest},
    service::competitions::CompetitionsService,
};
use rocket::{get, post, routes, serde::json::Json, State};
use std::sync::Arc;
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/competition/active",
    responses(
        (status = 200, description = "Active competition", body = Competition),
        (status = 404, description = "No active competition"),
        (status = 500, description = "Internal server error")
    )
)]
#[get("/active")]
pub async fn get_active_competition(
    competitions_service: &State<Arc<dyn CompetitionsService>>,
) -> Option<Json<Competition>> {
    match competitions_service.get_active_competition().await {
        Ok(Some(competition)) => Some(Json(competition)),
        Ok(None) => None,
        Err(e) => {
            eprintln!("Error fetching active competition: {:?}", e);
            None
        }
    }
}

#[utoipa::path(
    get,
    path = "/competitions/upcoming",
    responses(
        (status = 200, description = "List of upcoming competitions", body = Vec<Competition>),
        (status = 500, description = "Internal server error")
    )
)]
#[get("/upcoming")]
pub async fn get_upcoming_competitions(
    competitions_service: &State<Arc<dyn CompetitionsService>>,
) -> Json<Vec<Competition>> {
    match competitions_service.get_upcoming_competitions().await {
        Ok(competitions) => Json(competitions),
        Err(e) => {
            eprintln!("Error fetching upcoming competitions: {:?}", e);
            Json(vec![])
        }
    }
}

#[utoipa::path(
    get,
    path = "/competitions/results",
    responses(
        (status = 200, description = "Active competition leaderboard", body = LeaderboardResponse),
        (status = 404, description = "No active competition"),
        (status = 500, description = "Internal server error")
    )
)]
#[get("/results")]
pub async fn get_competition_results(
    competitions_service: &State<Arc<dyn CompetitionsService>>,
) -> Json<LeaderboardResponse> {
    // Get the active competition first
    match competitions_service.get_active_competition().await {
        Ok(Some(competition)) => {
            // Get leaderboard for the active competition
            match competitions_service.get_leaderboard(competition.competition_id).await {
                Ok(leaderboard) => Json(leaderboard),
                Err(e) => {
                    eprintln!("Error fetching leaderboard: {:?}", e);
                    Json(LeaderboardResponse {
                        competition_id: competition.competition_id,
                        results: vec![],
                    })
                }
            }
        }
        Ok(None) => {
            // No active competition
            Json(LeaderboardResponse {
                competition_id: Uuid::nil(),
                results: vec![],
            })
        }
        Err(e) => {
            eprintln!("Error fetching active competition: {:?}", e);
            Json(LeaderboardResponse {
                competition_id: Uuid::nil(),
                results: vec![],
            })
        }
    }
}

#[utoipa::path(
    post,
    path = "/competitions/submit_score",
    request_body = SubmitScoreRequest,
    responses(
        (status = 200, description = "Score submitted successfully", body = bool),
        (status = 400, description = "Invalid request or competition not active"),
        (status = 500, description = "Internal server error")
    )
)]
#[post("/submit_score", data = "<request>")]
pub async fn submit_score(
    request: Json<SubmitScoreRequest>,
    competitions_service: &State<Arc<dyn CompetitionsService>>,
) -> Json<bool> {
    match competitions_service.submit_score(request.into_inner()).await {
        Ok(_) => Json(true),
        Err(e) => {
            eprintln!("Error submitting score: {:?}", e);
            Json(false)
        }
    }
}

#[utoipa::path(
    post,
    path = "/competitions/generate",
    responses(
        (status = 200, description = "Generated competitions", body = Vec<Competition>),
        (status = 500, description = "Internal server error")
    )
)]
#[post("/generate")]
pub async fn generate_competitions(
    competitions_service: &State<Arc<dyn CompetitionsService>>,
) -> Json<Vec<Competition>> {
    match competitions_service
        .generate_competitions_if_needed()
        .await
    {
        Ok(competitions) => Json(competitions),
        Err(e) => {
            eprintln!("Error generating competitions: {:?}", e);
            Json(vec![])
        }
    }
}

pub fn active_competition_routes() -> Vec<rocket::Route> {
    routes![get_active_competition]
}

pub fn competition_routes() -> Vec<rocket::Route> {
    routes![
        get_upcoming_competitions,
        get_competition_results,
        submit_score,
        generate_competitions
    ]
}
