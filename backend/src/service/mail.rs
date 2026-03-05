use chrono::Utc;
use rocket::async_trait;
use sea_orm::{DatabaseConnection, DbErr, TransactionError, TransactionTrait};
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

pub struct MailServiceImpl<T: MailRepository + Clone> {
    db: DatabaseConnection,
    mail_repository: T,
}

impl<R: MailRepository + Clone> MailServiceImpl<R> {
    // create a new function for MailServiceImpl.
    pub fn new(db: DatabaseConnection, mail_repository: R) -> Self {
        Self { db, mail_repository }
    }
}

// Implement MailService trait for MailServiceImpl.
#[async_trait]
impl<R: MailRepository + Clone + 'static> MailService for MailServiceImpl<R> {
    async fn create(
        &self,
        mail_id: Uuid,
        sender_id: Uuid,
        receiver_ids: Vec<Uuid>,
        title: String,
        message: String,
    ) -> Result<(), DbErr> {
        let mail_repo = self.mail_repository.clone();
        let send_time = Utc::now();

        self.db
            .transaction::<_, (), DbErr>(move |tx| {
                Box::pin(async move {
                    mail_repo
                        .create_tx(tx, mail_id, sender_id, receiver_ids, title, message, send_time)
                        .await
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Connection(e) => e,
                TransactionError::Transaction(e) => e,
            })
    }

    async fn delete(&self, user_id: Uuid, mail_id: Uuid) -> Result<(), DbErr> {
        let mail_repo = self.mail_repository.clone();

        self.db
            .transaction::<_, (), DbErr>(move |tx| {
                Box::pin(async move { mail_repo.delete_tx(tx, user_id, mail_id).await })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Connection(e) => e,
                TransactionError::Transaction(e) => e,
            })
    }

    async fn change_read_state(
        &self,
        user_id: Uuid,
        mail_id: Uuid,
        read: bool,
    ) -> Result<(), DbErr> {
        let mail_repo = self.mail_repository.clone();

        self.db
            .transaction::<_, (), DbErr>(move |tx| {
                Box::pin(async move { mail_repo.read(tx, user_id, mail_id, read).await })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Connection(e) => e,
                TransactionError::Transaction(e) => e,
            })
    }

    async fn change_archive_state(
        &self,
        user_id: Uuid,
        mail_id: Uuid,
        archive: bool,
    ) -> Result<(), DbErr> {
        let mail_repo = self.mail_repository.clone();

        self.db
            .transaction::<_, (), DbErr>(move |tx| {
                Box::pin(async move { mail_repo.archive(tx, user_id, mail_id, archive).await })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Connection(e) => e,
                TransactionError::Transaction(e) => e,
            })
    }
}
