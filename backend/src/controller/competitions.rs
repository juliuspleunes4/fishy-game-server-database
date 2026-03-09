use crate::{
    domain::{Competition, LeaderboardResponse, SubmitScoreRequest},
    service::competitions::CompetitionsService,
};
use rocket::{get, post, routes, serde::json::Json, State};
use std::sync::Arc;
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/competitions/active",
    responses(
        (status = 200, description = "List of active competitions", body = Vec<Competition>),
        (status = 500, description = "Internal server error")
    )
)]
#[get("/active")]
pub async fn get_active_competitions(
    competitions_service: &State<Arc<dyn CompetitionsService>>,
) -> Json<Vec<Competition>> {
    match competitions_service.get_active_competitions().await {
        Ok(competitions) => Json(competitions),
        Err(e) => {
            eprintln!("Error fetching active competitions: {:?}", e);
            Json(vec![])
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
    path = "/competitions/{competition_id}",
    params(
        ("competition_id" = String, Path, description = "Competition ID")
    ),
    responses(
        (status = 200, description = "Competition details", body = Competition),
        (status = 404, description = "Competition not found"),
        (status = 500, description = "Internal server error")
    )
)]
#[get("/<competition_id>")]
pub async fn get_competition(
    competition_id: String,
    competitions_service: &State<Arc<dyn CompetitionsService>>,
) -> Option<Json<Competition>> {
    let uuid = match Uuid::parse_str(&competition_id) {
        Ok(id) => id,
        Err(e) => {
            eprintln!("Invalid UUID: {:?}", e);
            return None;
        }
    };
    
    match competitions_service
        .get_competition_by_id(uuid)
        .await
    {
        Ok(Some(competition)) => Some(Json(competition)),
        Ok(None) => None,
        Err(e) => {
            eprintln!("Error fetching competition: {:?}", e);
            None
        }
    }
}

#[utoipa::path(
    get,
    path = "/competitions/{competition_id}/results",
    params(
        ("competition_id" = String, Path, description = "Competition ID")
    ),
    responses(
        (status = 200, description = "Competition leaderboard", body = LeaderboardResponse),
        (status = 500, description = "Internal server error")
    )
)]
#[get("/<competition_id>/results")]
pub async fn get_competition_results(
    competition_id: String,
    competitions_service: &State<Arc<dyn CompetitionsService>>,
) -> Json<LeaderboardResponse> {
    let uuid = match Uuid::parse_str(&competition_id) {
        Ok(id) => id,
        Err(e) => {
            eprintln!("Invalid UUID: {:?}", e);
            return Json(LeaderboardResponse {
                competition_id: Uuid::nil(),
                results: vec![],
            });
        }
    };
    
    match competitions_service.get_leaderboard(uuid).await {
        Ok(leaderboard) => Json(leaderboard),
        Err(e) => {
            eprintln!("Error fetching leaderboard: {:?}", e);
            Json(LeaderboardResponse {
                competition_id: uuid,
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

pub fn competition_routes() -> Vec<rocket::Route> {
    routes![
        get_active_competitions,
        get_upcoming_competitions,
        get_competition,
        get_competition_results,
        submit_score,
        generate_competitions
    ]
}
