use std::sync::Arc;

use rocket::{post, routes, serde::json::Json, State};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::service::friends::FriendService;

/// Request body for adding a friend.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct FriendRequests {
    pub user_one: Uuid,
    pub user_two: Uuid,
    pub sender_id: Uuid,
}

/// Request body for removing a friend.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct RemoveFriendRequests {
    pub user_one: Uuid,
    pub user_two: Uuid,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct HandleFriendRequest {
    pub user_one: Uuid,
    pub user_two: Uuid,
    pub request_accepted: bool,
}

#[utoipa::path(
    post,
    path = "/friend/remove_friend",
    request_body = RemoveFriendRequests,
    responses(
        (status = 201, description = "Successfully removed friend", body = bool),
        (status = 400, description = "Invalid input data"),
        (status = 500, description = "Internal server error")
    ),
    description = "Removes a friend from the database",
    operation_id = "removeFriend",
    tag = "Friends"
)]
#[post("/remove_friend", data = "<payload>")]
async fn remove_friend(
    payload: Json<RemoveFriendRequests>,
    friends_service: &State<Arc<dyn FriendService>>,
) -> Json<bool> {
    match friends_service
        .remove_friend(payload.user_one, payload.user_two)
        .await
    {
        Ok(()) => Json(true),
        Err(_) => Json(false),
    }
}

#[utoipa::path(
    post,
    path = "/friend/add_friend_request",
    request_body = FriendRequests,
    responses(
        (status = 201, description = "Successfully added a friend request", body = bool),
        (status = 400, description = "Invalid input data"),
        (status = 500, description = "Internal server error")
    ),
    description = "Adds a friend request to the database",
    operation_id = "addFriendRequest",
    tag = "Friends"
)]
#[post("/add_friend_request", data = "<payload>")]
async fn add_friend_request(
    payload: Json<FriendRequests>,
    friends_service: &State<Arc<dyn FriendService>>,
) -> Json<bool> {
    match friends_service
        .add_friend_request(payload.user_one, payload.user_two, payload.sender_id)
        .await
    {
        Ok(()) => Json(true),
        Err(_) => Json(false),
    }
}

#[utoipa::path(
    post,
    path = "/friend/handle_request",
    request_body = FriendRequests,
    responses(
        (status = 201, description = "Successfully handled a friend request", body = bool),
        (status = 400, description = "Invalid input data"),
        (status = 500, description = "Internal server error")
    ),
    description = "Handles a pending friend request",
    operation_id = "handleFriendRequest",
    tag = "Friends"
)]
#[post("/handle_request", data = "<payload>")]
async fn handle_friend_request(
    payload: Json<HandleFriendRequest>,
    friends_service: &State<Arc<dyn FriendService>>,
) -> Json<bool> {
    if payload.request_accepted == true {
        if friends_service
            .add_friend(payload.user_one, payload.user_two)
            .await
            .is_err()
        {
            return Json(false);
        }
    }
    match friends_service
        .remove_friend_request(payload.user_one, payload.user_two)
        .await
    {
        Ok(()) => Json(true),
        Err(_) => Json(false),
    }
}

// Combine all the friend routes.
pub fn friend_routes() -> Vec<rocket::Route> {
    routes![remove_friend, add_friend_request, handle_friend_request]
}
