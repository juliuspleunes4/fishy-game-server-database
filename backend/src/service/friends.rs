use chrono::Utc;
use rocket::async_trait;
use sea_orm::DbErr;
use uuid::Uuid;

use crate::repository::friends::FriendRepository;

/// business logic for authorisation.
#[async_trait]
pub trait FriendService: Send + Sync {
    async fn remove_friend(&self, user_one_id: Uuid, user_two_id: Uuid) -> Result<(), DbErr>;

    async fn add_friend(&self, user_one_id: Uuid, user_two_id: Uuid) -> Result<(), DbErr>;

    async fn remove_friend_request(
        &self,
        user_one_id: Uuid,
        user_two_id: Uuid,
    ) -> Result<(), DbErr>;

    async fn add_friend_request(
        &self,
        user_one_id: Uuid,
        user_two_id: Uuid,
        sender: Uuid,
    ) -> Result<(), DbErr>;
}

pub struct FriendServiceImpl<U: FriendRepository> {
    friend_repository: U,
}

impl<U: FriendRepository> FriendServiceImpl<U> {
    pub fn new(friend_repository: U) -> Self {
        Self { friend_repository }
    }
}

// Implement the friend service trait for FriendServiceImpl.
#[async_trait]
impl<U: FriendRepository> FriendService for FriendServiceImpl<U> {
    async fn remove_friend(&self, user_one_id: Uuid, user_two_id: Uuid) -> Result<(), DbErr> {
        self.friend_repository
            .remove_friend(user_one_id, user_two_id)
            .await
    }

    async fn add_friend(&self, user_one_id: Uuid, user_two_id: Uuid) -> Result<(), DbErr> {
        let (user_one, user_two) = if user_one_id < user_two_id {
            (user_one_id, user_two_id)
        } else {
            (user_two_id, user_one_id)
        };
        self.friend_repository.add_friend(user_one, user_two).await
    }

    async fn remove_friend_request(
        &self,
        user_one_id: Uuid,
        user_two_id: Uuid,
    ) -> Result<(), DbErr> {
        self.friend_repository
            .remove_friend_request(user_one_id, user_two_id)
            .await
    }

    async fn add_friend_request(
        &self,
        user_one_id: Uuid,
        user_two_id: Uuid,
        sender: Uuid,
    ) -> Result<(), DbErr> {
        let (user_one, user_two) = if user_one_id < user_two_id {
            (user_one_id, user_two_id)
        } else {
            (user_two_id, user_one_id)
        };
        self.friend_repository
            .add_friend_request(user_one, user_two, sender, Utc::now())
            .await
    }
}
