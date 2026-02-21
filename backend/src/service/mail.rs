use chrono::Utc;
use rocket::async_trait;
use sea_orm::DbErr;
use uuid::Uuid;

use crate::repository::mail::MailRepository;

#[async_trait]
pub trait MailService: Send + Sync {
    async fn create(
        &self,
        mail_id: Uuid,
        sender_id: Uuid,
        receiver_ids: Vec<Uuid>,
        title: String,
        message: String,
    ) -> Result<(), DbErr>;

    async fn delete(&self, user_id: Uuid, mail_id: Uuid) -> Result<(), DbErr>;

    async fn change_read_state(
        &self,
        user_id: Uuid,
        mail_id: Uuid,
        read: bool,
    ) -> Result<(), DbErr>;

    async fn change_archive_state(
        &self,
        user_id: Uuid,
        mail_id: Uuid,
        archived: bool,
    ) -> Result<(), DbErr>;
}

pub struct MailServiceImpl<T: MailRepository> {
    mail_repository: T,
}

impl<R: MailRepository> MailServiceImpl<R> {
    // create a new function for MailServiceImpl.
    pub fn new(mail_repository: R) -> Self {
        Self { mail_repository }
    }
}

// Implement MailService trait for MailServiceImpl.
#[async_trait]
impl<R: MailRepository> MailService for MailServiceImpl<R> {
    async fn create(
        &self,
        mail_id: Uuid,
        sender_id: Uuid,
        receiver_ids: Vec<Uuid>,
        title: String,
        message: String,
    ) -> Result<(), DbErr> {
        self.mail_repository
            .create(mail_id, sender_id, receiver_ids, title, message, Utc::now())
            .await
    }

    async fn delete(&self, user_id: Uuid, mail_id: Uuid) -> Result<(), DbErr> {
        self.mail_repository.delete(user_id, mail_id).await
    }

    async fn change_read_state(
        &self,
        user_id: Uuid,
        mail_id: Uuid,
        read: bool,
    ) -> Result<(), DbErr> {
        self.mail_repository.read(user_id, mail_id, read).await
    }

    async fn change_archive_state(
        &self,
        user_id: Uuid,
        mail_id: Uuid,
        archive: bool,
    ) -> Result<(), DbErr> {
        self.mail_repository
            .archive(user_id, mail_id, archive)
            .await
    }
}
