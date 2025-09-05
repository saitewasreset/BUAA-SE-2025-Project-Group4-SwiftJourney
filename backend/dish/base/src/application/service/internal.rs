use async_trait::async_trait;
use shared::application_error::ApplicationError;
use shared::domain::RepositoryError;
use shared::internal::dish::command::{SaveRawDishCommand, SaveRawTakeawayCommand};
use shared::internal::dish::dto::{DbDishDTO, DbTakeawayDishDTO, DbTakeawayShopDTO};
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum DishInternalServiceError {
    #[error("dish not found")]
    NotFound(Uuid, String),
    #[error("related service error")]
    RelatedServiceError(#[from] anyhow::Error),
}

impl ApplicationError for DishInternalServiceError {
    fn error_code(&self) -> u32 {
        match self {
            DishInternalServiceError::NotFound(_, _) => 94005,
            DishInternalServiceError::RelatedServiceError(_) => 94001,
        }
    }

    fn error_message(&self) -> String {
        self.to_string()
    }
}

#[async_trait]
pub trait DishInternalService: 'static + Send + Sync {
    async fn save_raw_dish(&self, command: SaveRawDishCommand) -> Result<(), RepositoryError>;
    async fn save_raw_takeaway(
        &self,
        command: SaveRawTakeawayCommand,
    ) -> Result<(), RepositoryError>;

    async fn db_get_dishes(&self) -> Result<Vec<DbDishDTO>, RepositoryError>;
    async fn db_get_takeaway_dishes(&self) -> Result<Vec<DbTakeawayDishDTO>, RepositoryError>;

    async fn db_get_takeaway_shops(&self) -> Result<Vec<DbTakeawayShopDTO>, RepositoryError>;
}
