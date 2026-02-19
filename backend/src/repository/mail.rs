use chrono::{DateTime, Utc};
use rocket::async_trait;
use sqlx::{Error, PgPool};
use uuid::Uuid;

#[async_trait]
pub trait MailRepository: Send + Sync {
    async fn create(
        &self,
        mail_id: Uuid,
        sender_id: Uuid,
        receiver_ids: Vec<Uuid>,
        title: String,
        message: String,
        send_time: DateTime<Utc>,
    ) -> Result<(), sqlx::Error>;

    async fn delete(&self, user_id: Uuid, mail_id: Uuid) -> Result<(), sqlx::Error>;

    async fn read(&self, user_id: Uuid, mail_id: Uuid, read: bool) -> Result<(), sqlx::Error>;

    async fn archive(
        &self,
        user_id: Uuid,
        mail_id: Uuid,
        archived: bool,
    ) -> Result<(), sqlx::Error>;
}

#[derive(Debug, Clone)]
pub struct MailRepositoryImpl {
    pool: PgPool,
}

impl MailRepositoryImpl {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MailRepository for MailRepositoryImpl {
    async fn create(
        &self,
        mail_id: Uuid,
        sender_id: Uuid,
        receiver_ids: Vec<Uuid>,
        title: String,
        message: String,
        send_time: DateTime<Utc>,
    ) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        if let Err(e) = sqlx::query!(
            "INSERT INTO mail (mail_id, sender_id, title, message, send_time)
            VALUES ($1, $2, $3, $4, $5);",
            mail_id,
            sender_id,
            title,
            message,
            send_time,
        )
        .execute(&mut *tx)
        .await
        {
            dbg!(&e);
            return Err(e);
        }

        for receiver in receiver_ids {
            if let Err(e) = sqlx::query!(
                "INSERT INTO mailbox (user_id, mail_id, read, archived)
                VALUES ($1, $2, $3, $4);",
                receiver,
                mail_id,
                false,
                false,
            )
            .execute(&mut *tx)
            .await
            {
                dbg!(&e);
                return Err(e);
            }
        }

        match tx.commit().await {
            Ok(_) => Ok(()),
            Err(e) => {
                dbg!(&e);
                Err(e)
            }
        }
    }

    async fn delete(&self, user_id: Uuid, mail_id: Uuid) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        // Delete the mailbox entry first
        if let Err(e) = sqlx::query!(
            "DELETE FROM mailbox
            WHERE user_id = $1 AND mail_id = $2;",
            user_id,
            mail_id,
        )
        .execute(&mut *tx)
        .await
        {
            dbg!(&e);
            return Err(e);
        }

        // Remove the mail itself if nobody has a reference to it anymore
        if let Err(e) = sqlx::query!(
            "DELETE FROM mail
                WHERE mail_id = $1
                    AND NOT EXISTS (
                        SELECT 1 FROM mailbox WHERE mail_id = $1
                    );",
            mail_id,
        )
        .execute(&mut *tx)
        .await
        {
            dbg!(&e);
            return Err(e);
        }

        match tx.commit().await {
            Ok(_) => Ok(()),
            Err(e) => {
                dbg!(&e);
                Err(e)
            }
        }
    }

    async fn read(&self, user_id: Uuid, mail_id: Uuid, read: bool) -> Result<(), sqlx::Error> {
        let result = match sqlx::query!(
            "UPDATE mailbox SET read = $3 WHERE user_id = $1 AND mail_id = $2",
            user_id,
            mail_id,
            read,
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
            return Err(Error::RowNotFound);
        }

        Ok(())
    }

    async fn archive(
        &self,
        user_id: Uuid,
        mail_id: Uuid,
        archived: bool,
    ) -> Result<(), sqlx::Error> {
        let result = match sqlx::query!(
            "UPDATE mailbox SET archived = $3 WHERE user_id = $1 AND mail_id = $2",
            user_id,
            mail_id,
            archived,
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
            return Err(Error::RowNotFound);
        }

        Ok(())
    }
}
