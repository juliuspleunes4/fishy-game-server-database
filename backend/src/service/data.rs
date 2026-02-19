use rocket::async_trait;
use uuid::Uuid;

use crate::{domain::UserData, repository::data::DataRepository};

/// business logic for authorisation.
#[async_trait]
pub trait DataService: Send + Sync {
    async fn retreive_all(&self, user_id: Uuid) -> Result<UserData, sqlx::Error>;
}

pub struct DataServiceImpl<U: DataRepository> {
    data_repository: U,
}

impl<U: DataRepository> DataServiceImpl<U> {
    pub fn new(data_repository: U) -> Self {
        Self { data_repository }
    }
}

// Implement the data service trait for DataServiceImpl.
#[async_trait]
impl<U: DataRepository> DataService for DataServiceImpl<U> {
    async fn retreive_all(&self, user_id: Uuid) -> Result<UserData, sqlx::Error> {
        let data = match self.data_repository.retreive_all(user_id).await? {
            Some(user) => user,
            None => {
                return Err(sqlx::Error::WorkerCrashed);
            }
        };
        Ok(data)
    }
}
