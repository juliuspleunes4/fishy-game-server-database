use std::sync::Arc;

use rocket::{State, post, routes, serde::json::Json};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::service::fishmarket::FishmarketService;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct FishToSell {
    pub fish_uid: Uuid,
    pub fish_id: i32,
    pub new_state_blob: Option<String>,
}

/// Request body for selling a fish
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SellFishesRequest {
    pub seller_id: Uuid,
    pub fishes: Vec<FishToSell>,
    pub price: i32,
}

#[utoipa::path(
    post,
    path = "/fish_market/sell_fishes",
    request_body = SellFishesRequest,
    responses(
        (status = 200, description = "Fishes sold successfully", body = bool),
        (status = 400, description = "Invalid request data"),
        (status = 500, description = "Internal server error")
    )
)]
#[post("/sell_fishes", data = "<payload>")]
pub async fn sell_fishes(
    payload: Json<SellFishesRequest>,
    fishmarket_service: &State<Arc<dyn FishmarketService>>,
) -> Json<bool> {
    let inner = payload.into_inner();
    match fishmarket_service.sell_fishes(inner.seller_id, inner.fishes, inner.price).await {
        Ok(_) => Json(true),
        Err(e) => {
            eprintln!("Error selling fishes: {:?}", e);
            Json(false)
        }
    }
}

pub fn routes() -> Vec<rocket::Route> {
    routes![
        sell_fishes,
    ]
}
