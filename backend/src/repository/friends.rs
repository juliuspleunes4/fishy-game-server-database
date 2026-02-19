use chrono::{DateTime, Utc};
use rocket::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

#[async_trait]
pub trait FriendRepository: Send + Sync {
    async fn remove_friend(&self, user_one_id: Uuid, user_two_id: Uuid) -> Result<(), sqlx::Error>;

    async fn add_friend(&self, user_one: Uuid, user_two: Uuid) -> Result<(), sqlx::Error>;

    async fn remove_friend_request(
        &self,
        original_sender: Uuid,
        original_receiver: Uuid,
    ) -> Result<(), sqlx::Error>;

    async fn add_friend_request(
        &self,
        sender: Uuid,
        receiver: Uuid,
        sender_id: Uuid,
        request_created_time: DateTime<Utc>,
    ) -> Result<(), sqlx::Error>;
}

#[derive(Debug, Clone)]
pub struct FriendRepositoryImpl {
    pool: PgPool,
}

impl FriendRepositoryImpl {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl FriendRepository for FriendRepositoryImpl {
    async fn remove_friend(&self, user_one_id: Uuid, user_two_id: Uuid) -> Result<(), sqlx::Error> {
        let result = match sqlx::query!(
            "DELETE FROM friends 
                WHERE (user_one_id = $1 AND user_two_id = $2)
                OR (user_one_id = $2 AND user_two_id = $1)",
            user_one_id,
            user_two_id,
        )
        .execute(&self.pool)
        .await
        {
            Ok(o) => o,
            Err(e) => {
                dbg!(&e);
                return Err(e);
            }
        };

        if result.rows_affected() == 0 {
            return Err(sqlx::Error::RowNotFound);
        }

        Ok(())
    }

    async fn add_friend(&self, user_one_id: Uuid, user_two_id: Uuid) -> Result<(), sqlx::Error> {
        let result = match sqlx::query!(
            "INSERT INTO friends (user_one_id, user_two_id) VALUES ($1, $2)",
            user_one_id,
            user_two_id,
        )
        .execute(&self.pool)
        .await
        {
            Ok(o) => o,
            Err(e) => {
                dbg!(&e);
                return Err(e);
            }
        };

        if result.rows_affected() == 0 {
            return Err(sqlx::Error::RowNotFound);
        }

        Ok(())
    }

    async fn remove_friend_request(
        &self,
        user_one_id: Uuid,
        user_two_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        let result = match sqlx::query!(
            "DELETE FROM friend_requests 
                WHERE (user_one_id = $1 AND user_two_id = $2)
                OR (user_one_id = $2 AND user_two_id = $1)",
            user_one_id,
            user_two_id,
        )
        .execute(&self.pool)
        .await
        {
            Ok(o) => o,
            Err(e) => {
                dbg!(&e);
                return Err(e);
            }
        };

        if result.rows_affected() == 0 {
            return Err(sqlx::Error::RowNotFound);
        }

        Ok(())
    }

    async fn add_friend_request(
        &self,
        sender: Uuid,
        receiver: Uuid,
        sender_id: Uuid,
        request_created_time: DateTime<Utc>,
    ) -> Result<(), sqlx::Error> {
        let result = match sqlx::query!(
            "INSERT INTO friend_requests (user_one_id, user_two_id, request_sender_id, request_created_time) VALUES ($1, $2, $3, $4)",
            sender,
            receiver,
            sender_id,
            request_created_time,
        )
        .execute(&self.pool)
        .await {
            Ok(o) => o,
            Err(e) => {
                dbg!(&e);
                return Err(e);
            }
        };

        if result.rows_affected() == 0 {
            return Err(sqlx::Error::RowNotFound);
        }

        Ok(())
    }
}
