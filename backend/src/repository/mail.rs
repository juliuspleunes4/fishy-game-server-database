use crate::entity::{mail, mailbox};
use chrono::{DateTime, FixedOffset, Utc};
use rocket::async_trait;
use sea_orm::{
    prelude::Expr, sea_query::Query, ActiveModelTrait, ActiveValue::Set, ColumnTrait, Condition,
    DatabaseTransaction, DbErr, EntityTrait, QueryFilter,
};
use uuid::Uuid;

#[async_trait]
pub trait MailRepository: Send + Sync {
    async fn create_tx(
        &self,
        tx: &DatabaseTransaction,
        mail_id: Uuid,
        sender_id: Uuid,
        receiver_ids: Vec<Uuid>,
        title: String,
        message: String,
        send_time: DateTime<Utc>,
    ) -> Result<(), DbErr>;

    async fn delete_tx(
        &self,
        tx: &DatabaseTransaction,
        user_id: Uuid,
        mail_id: Uuid,
    ) -> Result<(), DbErr>;

    async fn read(
        &self,
        tx: &DatabaseTransaction,
        user_id: Uuid,
        mail_id: Uuid,
        read: bool,
    ) -> Result<(), DbErr>;

    async fn archive(
        &self,
        tx: &DatabaseTransaction,
        user_id: Uuid,
        mail_id: Uuid,
        archived: bool,
    ) -> Result<(), DbErr>;
}

#[derive(Debug, Clone)]
pub struct MailRepositoryImpl;

impl MailRepositoryImpl {
    pub fn new() -> Self {
        Self
    }

    async fn insert_mail(
        tx: &DatabaseTransaction,
        mail_id: Uuid,
        sender_id: Uuid,
        tilte: String,
        message: String,
        send_time: DateTime<Utc>,
    ) -> Result<(), DbErr> {
        let utc_offset =
            FixedOffset::east_opt(0).ok_or_else(|| DbErr::Custom("Invalid UTC offset".into()))?;

        mail::ActiveModel {
            mail_id: Set(mail_id),
            sender_id: Set(sender_id),
            title: Set(tilte),
            message: Set(message),
            send_time: Set(send_time.with_timezone(&utc_offset)),
        }
        .insert(tx)
        .await?;

        Ok(())
    }

    async fn insert_into_mailbox(
        tx: &DatabaseTransaction,
        user_id: Uuid,
        mail_id: Uuid,
    ) -> Result<(), DbErr> {
        mailbox::ActiveModel {
            user_id: Set(user_id),
            mail_id: Set(mail_id),
            read: Set(false),
            archived: Set(false),
        }
        .insert(tx)
        .await?;
        Ok(())
    }

    async fn delete_mail_mailbox(
        tx: &DatabaseTransaction,
        user_id: Uuid,
        mail_id: Uuid,
    ) -> Result<(), DbErr> {
        mailbox::Entity::delete_many()
            .filter(
                Condition::all()
                    .add(mailbox::Column::UserId.eq(user_id))
                    .add(mailbox::Column::MailId.eq(mail_id)),
            )
            .exec(tx)
            .await?;
        Ok(())
    }

    async fn delete_mail(tx: &DatabaseTransaction, mail_id: Uuid) -> Result<(), DbErr> {
        mail::Entity::delete_many()
            .filter(
                Condition::all()
                    .add(mail::Column::MailId.eq(mail_id))
                    .add(Expr::not_exists(
                        Query::select()
                            .column(mailbox::Column::MailId)
                            .from(mailbox::Entity)
                            .and_where(mailbox::Column::MailId.eq(mail_id))
                            .to_owned(),
                    )),
            )
            .exec(tx)
            .await?;

        Ok(())
    }
}

#[async_trait]
impl MailRepository for MailRepositoryImpl {
    async fn create_tx(
        &self,
        tx: &DatabaseTransaction,
        mail_id: Uuid,
        sender_id: Uuid,
        receiver_ids: Vec<Uuid>,
        title: String,
        message: String,
        send_time: DateTime<Utc>,
    ) -> Result<(), DbErr> {
        Self::insert_mail(tx, mail_id, sender_id, title, message, send_time).await?;
        for receiver in receiver_ids {
            Self::insert_into_mailbox(tx, receiver, mail_id).await?;
        }
        Ok(())
    }

    async fn delete_tx(
        &self,
        tx: &DatabaseTransaction,
        user_id: Uuid,
        mail_id: Uuid,
    ) -> Result<(), DbErr> {
        Self::delete_mail_mailbox(tx, user_id, mail_id).await?;
        Self::delete_mail(tx, mail_id).await?;
        Ok(())
    }

    async fn read(
        &self,
        tx: &DatabaseTransaction,
        user_id: Uuid,
        mail_id: Uuid,
        read: bool,
    ) -> Result<(), DbErr> {
        let result = mailbox::Entity::update_many()
            .col_expr(mailbox::Column::Read, Expr::value(read))
            .filter(
                Condition::all()
                    .add(mailbox::Column::UserId.eq(user_id))
                    .add(mailbox::Column::MailId.eq(mail_id)),
            )
            .exec(tx)
            .await?;

        if result.rows_affected == 0 {
            return Err(DbErr::RecordNotUpdated);
        }

        Ok(())
    }

    async fn archive(
        &self,
        tx: &DatabaseTransaction,
        user_id: Uuid,
        mail_id: Uuid,
        archived: bool,
    ) -> Result<(), DbErr> {
        let result = mailbox::Entity::update_many()
            .col_expr(mailbox::Column::Read, Expr::value(archived))
            .filter(
                Condition::all()
                    .add(mailbox::Column::UserId.eq(user_id))
                    .add(mailbox::Column::MailId.eq(mail_id)),
            )
            .exec(tx)
            .await?;

        if result.rows_affected == 0 {
            return Err(DbErr::RecordNotUpdated);
        }

        Ok(())
    }
}
