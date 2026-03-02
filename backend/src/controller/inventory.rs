use std::sync::Arc;

use rocket::{post, routes, serde::json::Json, State};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::service::inventory::InventoryService;

/// Request body for adding an item.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct UseItemRequest {
    pub user_id: Uuid,
    pub item_uuid: Uuid,
    pub definition_id: i32,
    pub state_blob: String,
}

/// Request body for adding an item.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct DegradeItemRequest {
    pub user_id: Uuid,
    pub item_uid: Uuid,
    pub amount: i32,
}

/// Request body for adding an item.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct IncreaseItemRequest {
    pub user_id: Uuid,
    pub item_uid: Uuid,
    pub amount: i32,
}

/// Request body for adding an item.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct DestroyItemRequest {
    pub user_id: Uuid,
    pub item_uid: Uuid,
}

// Utoipa is the crate that generates swagger documentation for your endpoints.
// The documentation for each endpoint is combined in docs.rs
// Make sure to add your endpoint in docs.rs when you write new endpoints.
#[utoipa::path(
    post,
    path = "/inventory/useItem",
    request_body = UseItemRequest,
    responses(
        (status = 201, description = "Item used successfully", body = bool),
        (status = 400, description = "Invalid input data"),
        (status = 500, description = "Internal server error")
    ),
    description = "Use an item in the inventory",
    operation_id = "useItem",
    tag = "Inventory"
)]
#[post("/use_item", data = "<payload>")]
async fn add_or_update_item(
    payload: Json<UseItemRequest>,
    inventory_service: &State<Arc<dyn InventoryService>>,
) -> Json<bool> {
    match inventory_service
        .use_item(
            payload.user_id,
            payload.item_uuid,
            payload.definition_id,
            payload.state_blob.clone(),
        )
        .await
    {
        Ok(()) => Json(true),
        Err(_) => Json(false),
    }
}

// Utoipa is the crate that generates swagger documentation for your endpoints.
// The documentation for each endpoint is combined in docs.rs
// Make sure to add your endpoint in docs.rs when you write new endpoints.
#[utoipa::path(
    post,
    path = "/inventory/destroy",
    request_body = DestroyItemRequest,
    responses(
        (status = 201, description = "Item removed successfully", body = bool),
        (status = 400, description = "Invalid input data"),
        (status = 500, description = "Internal server error")
    ),
    description = "Removes an item from the database",
    operation_id = "destroyItem",
    tag = "Inventory"
)]
#[post("/destroy", data = "<payload>")]
async fn destroy_item(
    payload: Json<DestroyItemRequest>,
    inventory_service: &State<Arc<dyn InventoryService>>,
) -> Json<bool> {
    match inventory_service
        .destroy(payload.user_id, payload.item_uid)
        .await
    {
        Ok(()) => Json(true),
        Err(_) => Json(false),
    }
}

// Combine all the inventory routes.
pub fn inventory_routes() -> Vec<rocket::Route> {
    routes![add_or_update_item, destroy_item]
}
