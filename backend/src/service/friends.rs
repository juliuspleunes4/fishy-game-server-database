use chrono::Utc;
use rocket::async_trait;
use sea_orm::{DatabaseConnection, DbErr, TransactionError, TransactionTrait};
use uuid::Uuid;

use crate::repository::friends::FriendRepository;

/// business logic for authorisation.
#[async_trait]
pub trait FriendService: Send + Sync {
    async fn remove_friend(&self, user_one_id: Uuid, user_two_id: Uuid) -> Result<(), DbErr>;

    async fn add_friend_request(
        &self,
        user_one_id: Uuid,
        user_two_id: Uuid,
        sender: Uuid,
    ) -> Result<(), DbErr>;

    async fn handle_friend_request(
        &self,
        user_one_id: Uuid,
        user_two_id: Uuid,
        accepted: bool,
    ) -> Result<(), DbErr>;
}

pub struct FriendServiceImpl<U: FriendRepository + Clone> {
    db: DatabaseConnection,
    friend_repository: U,
}

impl<U: FriendRepository + Clone> FriendServiceImpl<U> {
    pub fn new(db: DatabaseConnection, friend_repository: U) -> Self {
        Self {
            db,
            friend_repository,
        }
    }
}

// Implement the friend service trait for FriendServiceImpl.
#[async_trait]
impl<U: FriendRepository + Clone + 'static> FriendService for FriendServiceImpl<U> {
    async fn remove_friend(&self, user_one_id: Uuid, user_two_id: Uuid) -> Result<(), DbErr> {
        let friend_repo = self.friend_repository.clone();
        self.db
            .transaction::<_, (), DbErr>(move |tx| {
                Box::pin(async move {
                    friend_repo
                        .remove_friend(tx, user_one_id, user_two_id)
                        .await?;

                    Ok(())
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Connection(e) => e,
                TransactionError::Transaction(e) => e,
            })
    }

    async fn add_friend_request(
        &self,
        user_one_id: Uuid,
        user_two_id: Uuid,
        sender: Uuid,
    ) -> Result<(), DbErr> {
        let friend_repo = self.friend_repository.clone();
        self.db
            .transaction::<_, (), DbErr>(move |tx| {
                Box::pin(async move {
                    friend_repo
                        .add_friend_request(tx, user_one_id, user_two_id, sender, Utc::now())
                        .await?;

                    Ok(())
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Connection(e) => e,
                TransactionError::Transaction(e) => e,
            })
    }

    async fn handle_friend_request(
        &self,
        user_one_id: Uuid,
        user_two_id: Uuid,
        accepted: bool,
    ) -> Result<(), DbErr> {
        let friend_repo = self.friend_repository.clone();
        self.db
            .transaction::<_, (), DbErr>(move |tx| {
                Box::pin(async move {
                    friend_repo
                        .remove_friend_request(tx, user_one_id, user_two_id)
                        .await?;
                    if accepted {
                        friend_repo.add_friend(tx, user_one_id, user_two_id).await?;
                    }

                    Ok(())
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Connection(e) => e,
                TransactionError::Transaction(e) => e,
            })
    }
}
