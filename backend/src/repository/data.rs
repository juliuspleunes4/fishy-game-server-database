use rocket::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{
    ActiveEffect, FishData, Friend, FriendRequest, InventoryItem, MailEntry, UserData,
};

#[async_trait]
pub trait DataRepository: Send + Sync {
    async fn retreive_all(&self, user_id: Uuid) -> Result<Option<UserData>, sqlx::Error>;
}

#[derive(Debug, Clone)]
pub struct DataRepositoryImpl {
    pool: PgPool,
}

impl DataRepositoryImpl {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DataRepository for DataRepositoryImpl {
    async fn retreive_all(&self, user_id: Uuid) -> Result<Option<UserData>, sqlx::Error> {
        let rows = match sqlx::query!(
            "SELECT 
            u.name,
            s.xp,
            s.coins,
            s.bucks,
            s.total_playtime,
            s.selected_rod,
            s.selected_bait,
            COALESCE(
                json_agg(
                    json_build_object(
                        'fish_id', fc.fish_id,
                        'amount', fc.amount,
                        'max_length', fc.max_length,
                        'first_caught', fc.first_caught,
                        'areas', fca.areas,
                        'baits', fcb.baits
                    )
                ) FILTER (WHERE fc.fish_id IS NOT NULL), '[]'
            ) AS fish_data,
            COALESCE(
                json_agg(
                    DISTINCT jsonb_build_object(
                        'definition_id', i.definition_id,
                        'item_uuid', i.item_uuid,
                        'state_blob', i.state_blob
                    )
                ) FILTER (WHERE i.definition_id IS NOT NULL), '[]'
            ) AS inventory_item,
            COALESCE(
                json_agg(
                    DISTINCT jsonb_build_object(
                        'mail_id', m.mail_id,
                        'title', m.title,
                        'message', m.message,
                        'send_time', m.send_time,
                        'read', mb.read,
                        'archived', mb.archived
                    )
                ) FILTER (WHERE m.mail_id IS NOT NULL), '[]'
            ) AS mailbox,
            COALESCE(
                (
                    SELECT json_agg(json_build_array(f.user_one_id, f.user_two_id))
                    FROM friends f
                    WHERE f.user_one_id = $1 OR f.user_two_id = $1
                ), '[]'
            ) AS friends,
            COALESCE(
                (
                    SELECT json_agg(json_build_array(fr.user_one_id, fr.user_two_id, fr.request_sender_id))
                    FROM friend_requests fr
                    WHERE fr.user_one_id = $1 OR fr.user_two_id = $1
                ), '[]'
            ) AS friend_requests,
            COALESCE(
                (
                    SELECT json_agg(json_build_object(
                        'item_id', ae.item_id,
                        'expiry_time', ae.expiry_time
                    ))
                    FROM player_effects ae
                    WHERE ae.user_id = $1 AND ae.expiry_time > NOW()
                ), '[]'
            ) AS player_effects
            FROM users u
            LEFT JOIN stats s ON u.user_id = s.user_id
            LEFT JOIN fish_caught fc ON u.user_id = fc.user_id
            LEFT JOIN (
                SELECT user_id, fish_id, json_agg(area_id) AS areas
                FROM fish_caught_area
                GROUP BY user_id, fish_id
            ) fca ON fc.user_id = fca.user_id AND fc.fish_id = fca.fish_id
            LEFT JOIN (
                SELECT user_id, fish_id, json_agg(bait_id) AS baits
                FROM fish_caught_bait
                GROUP BY user_id, fish_id
            ) fcb ON fc.user_id = fcb.user_id AND fc.fish_id = fcb.fish_id
            LEFT JOIN inventory_item i ON u.user_id = i.user_id
            LEFT JOIN mailbox mb ON u.user_id = mb.user_id
            LEFT JOIN mail m ON mb.mail_id = m.mail_id
            WHERE u.user_id = $1
            GROUP BY u.user_id, u.name, u.email, u.created, s.xp, s.coins, s.bucks, s.total_playtime, s.selected_rod, s.selected_bait;
            ",
            user_id
        )
        .fetch_optional(&self.pool)
        .await {
            Ok(o) => o,
            Err(e) => {
                dbg!(&e);
                return Err(e);
            }
        };

        if let Some(data) = rows {
            let fish_data: Vec<FishData> =
                match serde_json::from_value(data.fish_data.unwrap_or_default()) {
                    Ok(o) => o,
                    Err(e) => {
                        dbg!(&e);
                        return Err(sqlx::Error::WorkerCrashed);
                    }
                };

            let inventory_items: Vec<InventoryItem> =
                match serde_json::from_value(data.inventory_item.unwrap_or_default()) {
                    Ok(o) => o,
                    Err(e) => {
                        dbg!(&e);
                        return Err(sqlx::Error::WorkerCrashed);
                    }
                };

            let mailbox: Vec<MailEntry> =
                match serde_json::from_value(data.mailbox.unwrap_or_default()) {
                    Ok(o) => o,
                    Err(e) => {
                        dbg!(&e);
                        return Err(sqlx::Error::WorkerCrashed);
                    }
                };

            let friends: Vec<Friend> =
                match serde_json::from_value(data.friends.unwrap_or_default()) {
                    Ok(o) => o,
                    Err(e) => {
                        dbg!(&e);
                        return Err(sqlx::Error::WorkerCrashed);
                    }
                };

            let friend_requests: Vec<FriendRequest> =
                match serde_json::from_value(data.friend_requests.unwrap_or_default()) {
                    Ok(o) => o,
                    Err(e) => {
                        dbg!(&e);
                        return Err(sqlx::Error::WorkerCrashed);
                    }
                };

            let active_effects: Vec<ActiveEffect> =
                match serde_json::from_value(data.player_effects.unwrap_or_default()) {
                    Ok(o) => o,
                    Err(e) => {
                        dbg!(&e);
                        return Err(sqlx::Error::WorkerCrashed);
                    }
                };

            let user_data = UserData {
                name: data.name,
                xp: data.xp,
                coins: data.coins,
                bucks: data.bucks,
                selected_rod: data.selected_rod,
                selected_bait: data.selected_bait,
                total_playtime: data.total_playtime,
                fish_data,
                inventory_items,
                mailbox,
                friends,
                friend_requests,
                active_effects,
            };

            return Ok(Some(user_data));
        }
        Err(sqlx::Error::WorkerCrashed)
    }
}
