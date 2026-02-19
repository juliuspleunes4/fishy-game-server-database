use std::sync::Arc;

use rocket::{post, routes, serde::json::Json, State};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{domain::UserData, service::data::DataService};

/// Request body for adding an item.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct RetreiveDataRequest {
    pub user_id: Uuid,
}

// Utoipa is the crate that generates swagger documentation for your endpoints.
// The documentation for each endpoint is combined in docs.rs
// Make sure to add your endpoint in docs.rs when you write new endpoints.
#[utoipa::path(
    post,
    path = "/data/retreive_all_playerdata",
    request_body = RetreiveDataRequest,
    responses(
        (status = 201, description = "Retreived successfully", body = bool),
        (status = 400, description = "Invalid input data"),
        (status = 500, description = "Internal server error")
    ),
    description = "Retreives user data from the database",
    operation_id = "retreiveItem",
    tag = "userData"
)]
#[post("/retreive_all_playerdata", data = "<payload>")]
async fn retreive_player_data(
    payload: Json<RetreiveDataRequest>,
    inventory_service: &State<Arc<dyn DataService>>,
) -> Json<Option<UserData>> {
    match inventory_service.retreive_all(payload.user_id).await {
        Ok(o) => Json(Some(o)),
        Err(_) => Json(None),
    }
}

// Combine all the data routes.
pub fn data_routes() -> Vec<rocket::Route> {
    routes![retreive_player_data]
}
