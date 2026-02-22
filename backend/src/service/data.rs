use rocket::async_trait;
use sea_orm::DbErr;
use uuid::Uuid;

use crate::{domain::UserData, repository::data::DataRepository};

#[async_trait]
pub trait DataService: Send + Sync {
    async fn retreive_all(&self, user_id: Uuid) -> Result<UserData, DbErr>;
}

pub struct DataServiceImpl<U: DataRepository> {
    data_repository: U,
}

impl<U: DataRepository> DataServiceImpl<U> {
    pub fn new(data_repository: U) -> Self {
        Self { data_repository }
    }
}

#[async_trait]
impl<U: DataRepository> DataService for DataServiceImpl<U> {
    async fn retreive_all(&self, user_id: Uuid) -> Result<UserData, DbErr> {
        match self.data_repository.retreive_all(user_id).await? {
            Some(data) => Ok(data),
            None => Err(DbErr::RecordNotFound("user data not found".into())),
        }
    }
}
