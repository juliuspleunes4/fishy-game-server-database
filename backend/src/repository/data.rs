use crate::domain::{
    ActiveEffect, FishData, Friend, FriendRequest, InventoryItem, MailEntry, UserData,
};
use crate::entity::{
    fish_caught, fish_caught_area, fish_caught_bait, friend_requests, friends, inventory_item,
    mail, mailbox, player_effects, stats, users,
};
use chrono::Utc;
use rocket::async_trait;
use sea_orm::prelude::Expr;
use sea_orm::sea_query::Alias;
use sea_orm::{
    ColumnTrait, Condition, DatabaseConnection, DatabaseTransaction, DbErr, EntityTrait, ExprTrait,
    JoinType, QueryFilter, QuerySelect, RelationTrait, TransactionError, TransactionTrait,
};
use std::collections::HashMap;
use uuid::Uuid;

#[async_trait]
pub trait DataRepository: Send + Sync {
    async fn retreive_all(&self, user_id: Uuid) -> Result<Option<UserData>, DbErr>;
}

#[derive(Debug, Clone)]
pub struct DataRepositoryImpl {
    db: DatabaseConnection,
}

impl DataRepositoryImpl {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    async fn fetch_user_name(
        tx: &DatabaseTransaction,
        user_id: Uuid,
    ) -> Result<Option<String>, DbErr> {
        let user = users::Entity::find_by_id(user_id).one(tx).await?;
        Ok(user.map(|u| u.name))
    }

    async fn fetch_stats(
        tx: &DatabaseTransaction,
        user_id: Uuid,
    ) -> Result<Option<stats::Model>, DbErr> {
        stats::Entity::find_by_id(user_id).one(tx).await
    }

    async fn fetch_fish_data(
        tx: &DatabaseTransaction,
        user_id: Uuid,
    ) -> Result<Vec<FishData>, DbErr> {
        let fish_list = fish_caught::Entity::find()
            .filter(fish_caught::Column::UserId.eq(user_id))
            .all(tx)
            .await?;

        let areas = fish_caught_area::Entity::find()
            .filter(fish_caught_area::Column::UserId.eq(user_id))
            .all(tx)
            .await?;

        let baits = fish_caught_bait::Entity::find()
            .filter(fish_caught_bait::Column::UserId.eq(user_id))
            .all(tx)
            .await?;

        let mut area_map: HashMap<i32, Vec<i32>> = HashMap::new();
        for a in areas {
            area_map.entry(a.fish_id).or_default().push(a.area_id);
        }

        let mut bait_map: HashMap<i32, Vec<i32>> = HashMap::new();
        for b in baits {
            bait_map.entry(b.fish_id).or_default().push(b.bait_id);
        }

        Ok(fish_list
            .into_iter()
            .map(|f| FishData {
                fish_id: f.fish_id,
                amount: f.amount,
                max_length: f.max_length,
                first_caught: f.first_caught,
                areas: area_map.remove(&f.fish_id).unwrap_or_default(),
                baits: bait_map.remove(&f.fish_id).unwrap_or_default(),
            })
            .collect())
    }

    async fn fetch_inventory(
        tx: &DatabaseTransaction,
        user_id: Uuid,
    ) -> Result<Vec<InventoryItem>, DbErr> {
        let items = inventory_item::Entity::find()
            .filter(inventory_item::Column::UserId.eq(user_id))
            .all(tx)
            .await?;

        Ok(items
            .into_iter()
            .map(|i| InventoryItem {
                item_uuid: i.item_uuid,
                definition_id: i.definition_id,
                state_blob: i.state_blob,
            })
            .collect())
    }

    async fn fetch_mailbox(
        tx: &DatabaseTransaction,
        user_id: Uuid,
    ) -> Result<Vec<MailEntry>, DbErr> {
        let sender = Alias::new("sender");

        let rel_mail = mailbox::Relation::Mail.def();
        let rel_sender = mail::Entity::belongs_to(users::Entity)
            .from(mail::Column::SenderId)
            .to(users::Column::UserId)
            .into();

        let rows = mailbox::Entity::find()
            .filter(mailbox::Column::UserId.eq(user_id))
            .join(JoinType::InnerJoin, rel_mail)
            .join_as(JoinType::InnerJoin, rel_sender, sender.clone())
            .select_only()
            .column_as(mail::Column::MailId, "mail_id")
            .column_as(mail::Column::Title, "title")
            .column_as(mail::Column::Message, "message")
            .column_as(mail::Column::SendTime, "send_time")
            .column_as(mailbox::Column::Read, "read")
            .column_as(mailbox::Column::Archived, "archived")
            .expr_as(Expr::col((sender, users::Column::Name)), "sender_name")
            .into_model::<MailEntry>()
            .all(tx)
            .await?;

        Ok(rows
            .into_iter()
            .map(|e| MailEntry {
                send_time: e.send_time.with_timezone(&Utc),
                ..e
            })
            .collect())
    }

    async fn fetch_friends(tx: &DatabaseTransaction, user_id: Uuid) -> Result<Vec<Friend>, DbErr> {
        let u1 = Alias::new("u1");
        let u2 = Alias::new("u2");

        let rel_user_one = friends::Entity::belongs_to(users::Entity)
            .from(friends::Column::UserOneId)
            .to(users::Column::UserId)
            .into();

        let rel_user_two = friends::Entity::belongs_to(users::Entity)
            .from(friends::Column::UserTwoId)
            .to(users::Column::UserId)
            .into();

        friends::Entity::find()
            .filter(
                Condition::any()
                    .add(friends::Column::UserOneId.eq(user_id))
                    .add(friends::Column::UserTwoId.eq(user_id)),
            )
            .join_as(JoinType::InnerJoin, rel_user_one, u1.clone())
            .join_as(JoinType::InnerJoin, rel_user_two, u2.clone())
            .select_only()
            .expr_as(
                Expr::case(
                    Expr::col(friends::Column::UserOneId).eq(user_id),
                    Expr::col(friends::Column::UserTwoId),
                )
                .finally(Expr::col(friends::Column::UserOneId)),
                "friend_id",
            )
            .expr_as(
                Expr::case(
                    Expr::col(friends::Column::UserOneId).eq(user_id),
                    Expr::col((u2, users::Column::Name)),
                )
                .finally(Expr::col((u1, users::Column::Name))),
                "friend_name",
            )
            .into_model::<Friend>()
            .all(tx)
            .await
    }

    async fn fetch_friend_requests(
        tx: &DatabaseTransaction,
        user_id: Uuid,
    ) -> Result<Vec<FriendRequest>, DbErr> {
        let u1 = Alias::new("u1");
        let u2 = Alias::new("u2");

        let rel_user_one = friend_requests::Entity::belongs_to(users::Entity)
            .from(friend_requests::Column::UserOneId)
            .to(users::Column::UserId)
            .into();

        let rel_user_two = friend_requests::Entity::belongs_to(users::Entity)
            .from(friend_requests::Column::UserTwoId)
            .to(users::Column::UserId)
            .into();

        friend_requests::Entity::find()
            .filter(
                Condition::any()
                    .add(friend_requests::Column::UserOneId.eq(user_id))
                    .add(friend_requests::Column::UserTwoId.eq(user_id)),
            )
            .join_as(JoinType::InnerJoin, rel_user_one, u1.clone())
            .join_as(JoinType::InnerJoin, rel_user_two, u2.clone())
            .select_only()
            .expr_as(
                Expr::case(
                    Expr::col(friend_requests::Column::UserOneId).eq(user_id),
                    Expr::col(friend_requests::Column::UserTwoId),
                )
                .finally(Expr::col(friend_requests::Column::UserOneId)),
                "other_id",
            )
            .expr_as(
                Expr::case(
                    Expr::col(friend_requests::Column::UserOneId).eq(user_id),
                    Expr::col((u2, users::Column::Name)),
                )
                .finally(Expr::col((u1, users::Column::Name))),
                "other_name",
            )
            .expr_as(
                Expr::col(friend_requests::Column::RequestSenderId),
                "request_sender_id",
            )
            .into_model::<FriendRequest>()
            .all(tx)
            .await
    }

    async fn fetch_active_effects(
        tx: &DatabaseTransaction,
        user_id: Uuid,
    ) -> Result<Vec<ActiveEffect>, DbErr> {
        let now = Utc::now().fixed_offset();

        let rows = player_effects::Entity::find()
            .filter(
                player_effects::Column::UserId
                    .eq(user_id)
                    .and(player_effects::Column::ExpiryTime.gt(now)),
            )
            .all(tx)
            .await?;

        Ok(rows
            .into_iter()
            .map(|e| ActiveEffect {
                item_id: e.item_id,
                expiry_time: e.expiry_time.with_timezone(&Utc),
            })
            .collect())
    }
}

#[async_trait]
impl DataRepository for DataRepositoryImpl {
    async fn retreive_all(&self, user_id: Uuid) -> Result<Option<UserData>, DbErr> {
        self.db
            .transaction::<_, Option<UserData>, DbErr>(|tx| {
                Box::pin(async move {
                    let name = match Self::fetch_user_name(tx, user_id).await? {
                        Some(name) => name,
                        None => return Ok(None),
                    };

                    let stats = match Self::fetch_stats(tx, user_id).await? {
                        Some(s) => s,
                        None => return Ok(None),
                    };

                    let fish_data = Self::fetch_fish_data(tx, user_id).await?;
                    let inventory_items = Self::fetch_inventory(tx, user_id).await?;
                    let mailbox = Self::fetch_mailbox(tx, user_id).await?;
                    let friends = Self::fetch_friends(tx, user_id).await?;
                    let friend_requests = Self::fetch_friend_requests(tx, user_id).await?;
                    let active_effects = Self::fetch_active_effects(tx, user_id).await?;

                    Ok(Some(UserData {
                        name,
                        xp: stats.xp,
                        coins: stats.coins,
                        bucks: stats.bucks,
                        total_playtime: stats.total_playtime,
                        selected_rod: stats.selected_rod,
                        selected_bait: stats.selected_bait,
                        fish_data,
                        inventory_items,
                        mailbox,
                        friends,
                        friend_requests,
                        active_effects,
                    }))
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Connection(e) => e,
                TransactionError::Transaction(e) => e,
            })
    }
}
