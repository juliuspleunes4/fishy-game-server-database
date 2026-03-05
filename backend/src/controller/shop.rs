use rocket::{State, post, routes, serde::json::Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::service::shop::ShopService;

/// Request body for buying an item.
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone, Copy)]
pub enum MoneyType {
    COINS,
    BUCKS,
}

/// Request body for buying an item.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct BuyItemRequest {
    pub player_id: Uuid,
    pub item_def_id: i32,
    pub item_uuid: Uuid,
    pub item_state_blob: String,
    pub item_price: i32,
    pub bought_using: MoneyType,
}

#[utoipa::path(
    post,
    path = "/shop/buy_item",
    request_body = BuyItemRequest,
    responses(
        (status = 201, description = "Item bough successfully", body = bool),
        (status = 400, description = "Invalid input data"),
        (status = 500, description = "Internal server error")
    ),
    description = "Buys an item",
    operation_id = "buyItem",
    tag = "shop"
)]
#[post("/buy_item", data = "<payload>")]
async fn buy_item(
    payload: Json<BuyItemRequest>,
    shop_service: &State<Arc<dyn ShopService>>,
) -> Json<bool> {
    match shop_service
        .buy_item(
            payload.player_id,
            payload.item_def_id,
            payload.item_uuid,
            payload.item_state_blob.clone(),
            payload.item_price,
            payload.bought_using,
        )
        .await
    {
        Ok(()) => Json(true),
        Err(_) => Json(false),
    }
}

pub fn shop_routes() -> Vec<rocket::Route> {
    routes![
        buy_item,
    ]
}
