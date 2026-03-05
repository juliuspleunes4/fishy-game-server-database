use rocket::async_trait;
use sea_orm::{DatabaseConnection, DbErr, TransactionError, TransactionTrait};
use uuid::Uuid;

use crate::{domain::UserData, repository::data::DataRepository};

#[async_trait]
pub trait DataService: Send + Sync {
    async fn retreive_all(&self, user_id: Uuid) -> Result<UserData, DbErr>;
}

pub struct DataServiceImpl<U: DataRepository + Clone> {
    db: DatabaseConnection,
    data_repository: U,
}

impl<U: DataRepository + Clone> DataServiceImpl<U> {
    pub fn new(db: DatabaseConnection, data_repository: U) -> Self {
        Self { db, data_repository }
    }
}

#[async_trait]
impl<U: DataRepository + Clone + 'static> DataService for DataServiceImpl<U> {
    async fn retreive_all(&self, user_id: Uuid) -> Result<UserData, DbErr> {
        let data_repo = self.data_repository.clone();

        let result = self
            .db
            .transaction::<_, Option<UserData>, DbErr>(move |tx| {
                Box::pin(async move { data_repo.retreive_all(tx, user_id).await })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Connection(e) => e,
                TransactionError::Transaction(e) => e,
            })?;

        match result {
            Some(data) => Ok(data),
            None => Err(DbErr::RecordNotFound("user data not found".into())),
        }
    }
}
