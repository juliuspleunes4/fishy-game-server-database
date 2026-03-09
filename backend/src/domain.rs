use chrono::{DateTime, Utc};
use sea_orm::FromQueryResult;
use serde::Deserialize;
use serde::Serialize;
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct LoginResponse {
    pub code: i16,
    pub jwt: String,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub user_id: Uuid,
    pub name: String,
    pub email: String,
    pub password: String,
    pub salt: String,
    pub created: DateTime<Utc>,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, FromRow)]
pub struct StatFish {
    pub fish_id: i32,
    pub length: i32,
    pub bait_id: i32,
    pub area_id: i32,
}

/// Request body for adding playtime of a player
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SelectItemRequest {
    pub user_id: Uuid,
    pub item_uid: Uuid,
    pub item_type: ItemType,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone, Copy)]
pub enum ItemType {
    Rod,
    Bait,
    Extra,
}

// Struct to retreive user data
#[derive(Serialize, Debug, Deserialize)]
pub struct UserData {
    pub name: String,
    pub xp: i32,
    pub coins: i32,
    pub bucks: i32,
    pub total_playtime: i32,
    pub selected_rod: Option<Uuid>,
    pub selected_bait: Option<Uuid>,
    pub fish_data: Vec<FishData>,
    pub inventory_items: Vec<InventoryItem>,
    pub mailbox: Vec<MailEntry>,
    pub friends: Vec<Friend>,
    pub friend_requests: Vec<FriendRequest>,
    pub active_effects: Vec<ActiveEffect>,
}

#[derive(Serialize, Debug, Deserialize)]
pub struct FishData {
    pub fish_id: i32,
    pub amount: i32,
    pub max_length: i32,
    pub first_caught: chrono::NaiveDate,
    pub areas: Vec<i32>,
    pub baits: Vec<i32>,
}

#[derive(Serialize, Debug, Deserialize)]
pub struct InventoryItem {
    pub item_uuid: Uuid,
    pub definition_id: i32,
    pub state_blob: String,
}

#[derive(Serialize, Debug, Deserialize, sea_orm::FromQueryResult)]
pub struct MailEntry {
    pub mail_id: Uuid,
    pub title: String,
    pub message: String,
    pub send_time: chrono::DateTime<Utc>,
    pub read: bool,
    pub archived: bool,
    pub sender_name: String,
}

#[derive(Serialize, Debug, Deserialize, sea_orm::FromQueryResult)]
pub struct Friend {
    pub friend_id: Uuid,
    pub friend_name: String,
}

#[derive(Serialize, Debug, Deserialize, sea_orm::FromQueryResult)]
pub struct FriendRequest {
    pub other_id: Uuid,
    pub other_name: String,
    pub request_sender_id: Uuid,
}

#[derive(Serialize, Debug, Deserialize, FromRow, FromQueryResult)]
pub struct ActiveEffect {
    pub item_id: i32,
    pub expiry_time: DateTime<Utc>,
}

/// Request body for adding an active effect
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AddActiveEffectRequest {
    pub user_id: Uuid,
    pub item_id: i32,
    #[schema(value_type = String, format = DateTime)]
    pub expiry_time: DateTime<Utc>,
}

/// Request body for removing an active effect
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RemoveActiveEffectRequest {
    pub user_id: Uuid,
    pub item_id: i32,
}

// ============================================================================
// Competition System Domain Types
// ============================================================================

/// Competition record from database
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Competition {
    pub competition_id: Uuid,
    pub competition_type: i32,
    pub target_fish_id: i32,
    #[schema(value_type = String, format = DateTime)]
    pub start_time: DateTime<Utc>,
    #[schema(value_type = String, format = DateTime)]
    pub end_time: DateTime<Utc>,
    pub reward_currency: String,
    pub prize_pool: Vec<i32>,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    pub status: String,
}

/// Competition result/leaderboard entry
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct CompetitionResult {
    pub result_id: Uuid,
    pub competition_id: Uuid,
    pub player_id: Uuid,
    pub score: i32,
    #[schema(value_type = String, format = DateTime)]
    pub last_updated: DateTime<Utc>,
}

/// Request to submit a score to a competition
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SubmitScoreRequest {
    pub competition_id: Uuid,
    pub player_id: Uuid,
    pub score: i32,
}

/// Response containing leaderboard data
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LeaderboardResponse {
    pub competition_id: Uuid,
    pub results: Vec<CompetitionResult>,
}

