use chrono::{DateTime, Utc};
use rocket::async_trait;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, Condition, DatabaseTransaction, DbErr,
    EntityTrait, QueryFilter,
};
use uuid::Uuid;

use crate::entity::{friend_requests, friends};

#[async_trait]
pub trait FriendRepository: Send + Sync {
    async fn remove_friend(
        &self,
        tx: &DatabaseTransaction,
        user_one_id: Uuid,
        user_two_id: Uuid,
    ) -> Result<(), DbErr>;

    async fn add_friend(
        &self,
        tx: &DatabaseTransaction,
        user_one: Uuid,
        user_two: Uuid,
    ) -> Result<(), DbErr>;

    async fn remove_friend_request(
        &self,
        tx: &DatabaseTransaction,
        user_one_id: Uuid,
        user_two_id: Uuid,
    ) -> Result<(), DbErr>;

    async fn add_friend_request(
        &self,
        tx: &DatabaseTransaction,
        user_one_id: Uuid,
        user_two_id: Uuid,
        sender_id: Uuid,
        request_created_time: DateTime<Utc>,
    ) -> Result<(), DbErr>;
}

#[derive(Debug, Clone)]
pub struct FriendRepositoryImpl {}

impl FriendRepositoryImpl {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl FriendRepository for FriendRepositoryImpl {
    async fn remove_friend(
        &self,
        tx: &DatabaseTransaction,
        user_one_id: Uuid,
        user_two_id: Uuid,
    ) -> Result<(), DbErr> {
        let result = friends::Entity::delete_many()
            .filter(
                Condition::any()
                    .add(
                        Condition::all()
                            .add(friends::Column::UserOneId.eq(user_one_id))
                            .add(friends::Column::UserTwoId.eq(user_two_id)),
                    )
                    .add(
                        Condition::all()
                            .add(friends::Column::UserOneId.eq(user_two_id))
                            .add(friends::Column::UserTwoId.eq(user_one_id)),
                    )
                    .to_owned(),
            )
            .exec(tx)
            .await?;

        if result.rows_affected == 0 {
            return Err(DbErr::RecordNotUpdated);
        }

        Ok(())
    }

    async fn add_friend(
        &self,
        tx: &DatabaseTransaction,
        user_one_id: Uuid,
        user_two_id: Uuid,
    ) -> Result<(), DbErr> {
        friends::ActiveModel {
            user_one_id: Set(user_one_id),
            user_two_id: Set(user_two_id),
        }
        .insert(tx)
        .await?;

        Ok(())
    }

    async fn remove_friend_request(
        &self,
        tx: &DatabaseTransaction,
        user_one_id: Uuid,
        user_two_id: Uuid,
    ) -> Result<(), DbErr> {
        let result = friend_requests::Entity::delete_many()
            .filter(
                Condition::any()
                    .add(
                        Condition::all()
                            .add(friend_requests::Column::UserOneId.eq(user_one_id))
                            .add(friend_requests::Column::UserTwoId.eq(user_two_id)),
                    )
                    .add(
                        Condition::all()
                            .add(friend_requests::Column::UserOneId.eq(user_two_id))
                            .add(friend_requests::Column::UserTwoId.eq(user_one_id)),
                    )
                    .to_owned(),
            )
            .exec(tx)
            .await?;

        if result.rows_affected == 0 {
            return Err(DbErr::RecordNotInserted);
        }

        Ok(())
    }

    async fn add_friend_request(
        &self,
        tx: &DatabaseTransaction,
        user_one_id: Uuid,
        user_two_id: Uuid,
        sender_id: Uuid,
        request_created_time: DateTime<Utc>,
    ) -> Result<(), DbErr> {
        friend_requests::ActiveModel {
            user_one_id: Set(user_one_id),
            user_two_id: Set(user_two_id),
            request_sender_id: Set(sender_id),
            request_created_time: Set(request_created_time.fixed_offset()),
        }
        .insert(tx)
        .await?;

        Ok(())
    }
}
