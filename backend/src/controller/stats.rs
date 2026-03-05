use crate::{
    domain::{SelectItemRequest, StatFish},
    service::stats::StatsService,
};
use rocket::{post, routes, serde::json::Json, State};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;

/// Request body for adding playtime of a player
#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct AddPlayTimeRequest {
    pub user_id: Uuid,
    pub amount: i32,
}

/// Request body for adding playtime of a player
#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct AddFishRequest {
    pub user_id: Uuid,
    pub length: i32,
    pub fish_id: i32,
    pub bait_id: i32,
    pub area_id: i32,
    pub xp_earned: i32,
}

#[utoipa::path(
    post,
    path = "/stats/add_playtime",
    request_body = AddPlayTimeRequest,
    responses(
        (status = 201, description = "playtime changed successfully", body = bool),
        (status = 400, description = "Invalid input data"),
        (status = 500, description = "Internal server error")
    ),
    description = "Adds more playtime to a given user account",
    operation_id = "changePlayetime",
    tag = "Stats"
)]
#[post("/add_playtime", data = "<payload>")]
async fn add_playtime(
    payload: Json<AddPlayTimeRequest>,
    stats_service: &State<Arc<dyn StatsService>>,
) -> Json<bool> {
    if payload.amount < 0 {
        return Json(false);
    }
    match stats_service
        .add_playtime(payload.user_id, payload.amount)
        .await
    {
        Ok(()) => Json(true),
        Err(_) => Json(false),
    }
}

#[utoipa::path(
    post,
    path = "/stats/add_fish",
    request_body = AddFishRequest,
    responses(
        (status = 201, description = "stat fish added successfully", body = bool),
        (status = 400, description = "Invalid input data"),
        (status = 500, description = "Internal server error")
    ),
    description = "Adds a stat fish to a given user account",
    operation_id = "changePlayetime",
    tag = "Stats"
)]
#[post("/add_fish", data = "<payload>")]
async fn add_fish(
    payload: Json<AddFishRequest>,
    stats_service: &State<Arc<dyn StatsService>>,
) -> Json<bool> {
    match stats_service
        .add_fish(payload.user_id, StatFish {
            fish_id: payload.fish_id,
            length: payload.length,
            bait_id: payload.bait_id,
            area_id: payload.area_id,
        }, payload.xp_earned)
        .await
    {
        Ok(()) => Json(true),
        Err(_) => Json(false),
    }
}

#[utoipa::path(
    post,
    path = "/stats/select_item",
    request_body = SelectItemRequest,
    responses(
        (status = 201, description = "Successfully selected an item", body = bool),
        (status = 400, description = "Invalid input data"),
        (status = 500, description = "Internal server error")
    ),
    description = "Select an item",
    operation_id = "selectItem",
    tag = "Stats"
)]
#[post("/select_item", data = "<payload>")]
async fn select_item(
    payload: Json<SelectItemRequest>,
    stats_service: &State<Arc<dyn StatsService>>,
) -> Json<bool> {
    match stats_service
        .select_item(SelectItemRequest {
            user_id: payload.user_id,
            item_uid: payload.item_uid,
            item_type: payload.item_type,
        })
        .await
    {
        Ok(()) => Json(true),
        Err(_) => Json(false),
    }
}

// Combine all the user routes.
pub fn stats_routes() -> Vec<rocket::Route> {
    routes![
        add_playtime,
        add_fish,
        select_item
    ]
}
