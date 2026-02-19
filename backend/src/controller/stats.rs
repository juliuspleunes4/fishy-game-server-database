use crate::{
    domain::{SelectItemRequest, StatFish},
    service::stats::StatsService,
};
use rocket::{post, routes, serde::json::Json, State};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;

/// Request body for adding xp to an account.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct AddXPRequest {
    pub user_id: Uuid,
    pub amount: i32,
}

/// Request body for changing the coins amount of a player
#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct ChangeBucksRequest {
    pub user_id: Uuid,
    pub amount: i32,
}

/// Request body for changing the bucks amount of a player
#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct ChangeCoinsRequest {
    pub user_id: Uuid,
    pub amount: i32,
}

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
}

#[utoipa::path(
    post,
    path = "/stats/add_xp",
    request_body = AddXPRequest,
    responses(
        (status = 201, description = "xp added successfully", body = bool),
        (status = 400, description = "Invalid input data"),
        (status = 500, description = "Internal server error")
    ),
    description = "Adds xp to a given user account",
    operation_id = "addXP",
    tag = "Stats"
)]
#[post("/add_xp", data = "<payload>")]
async fn add_xp(
    payload: Json<AddXPRequest>,
    stats_service: &State<Arc<dyn StatsService>>,
) -> Json<bool> {
    if payload.amount < 0 {
        return Json(false);
    }
    match stats_service.add_xp(payload.user_id, payload.amount).await {
        Ok(()) => Json(true),
        Err(_) => Json(false),
    }
}

#[utoipa::path(
    post,
    path = "/stats/change_bucks",
    request_body = ChangeBucksRequest,
    responses(
        (status = 201, description = "bucks changed successfully", body = bool),
        (status = 400, description = "Invalid input data"),
        (status = 500, description = "Internal server error")
    ),
    description = "Changes the amount of bucks of a given user account",
    operation_id = "changeBucks",
    tag = "Stats"
)]
#[post("/change_bucks", data = "<payload>")]
async fn change_bucks(
    payload: Json<ChangeBucksRequest>,
    stats_service: &State<Arc<dyn StatsService>>,
) -> Json<bool> {
    match stats_service
        .change_bucks(payload.user_id, payload.amount)
        .await
    {
        Ok(()) => Json(true),
        Err(_) => Json(false),
    }
}

#[utoipa::path(
    post,
    path = "/stats/change_coins",
    request_body = ChangeCoinsRequest,
    responses(
        (status = 201, description = "coins changed successfully", body = bool),
        (status = 400, description = "Invalid input data"),
        (status = 500, description = "Internal server error")
    ),
    description = "Changes the amount of coins of a given user account",
    operation_id = "changeCoins",
    tag = "Stats"
)]
#[post("/change_coins", data = "<payload>")]
async fn change_coins(
    payload: Json<ChangeCoinsRequest>,
    stats_service: &State<Arc<dyn StatsService>>,
) -> Json<bool> {
    match stats_service
        .change_coins(payload.user_id, payload.amount)
        .await
    {
        Ok(()) => Json(true),
        Err(_) => Json(false),
    }
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
        .add_fish(StatFish {
            user_id: payload.user_id,
            fish_id: payload.fish_id,
            length: payload.length,
            bait_id: payload.bait_id,
            area_id: payload.area_id,
        })
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
        add_xp,
        change_bucks,
        change_coins,
        add_playtime,
        add_fish,
        select_item
    ]
}
