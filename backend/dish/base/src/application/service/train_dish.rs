use async_trait::async_trait;
use shared::api::ApplicationError;
use shared::internal::dish::command::OrderTrainDishCommand;
use shared::internal::order::dto::TransactionInfoDTO;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TrainDishApplicationServiceError {
    #[error("Invalid dish name: {0}")]
    InvalidDishName(String),
    #[error("Invalid dish name")]
    InvalidAmount,
    #[error("Invalid takeaway station: {0}")]
    InvalidTakeawayStation(String),
    #[error("Invalid takeaway shop name: {0}")]
    InvalidTakeawayShopName(String),
    #[error("Invalid takeaway name: {0}")]
    InvalidTakeawayName(String),
    #[error("No related train order found")]
    NoRelatedTrainOrder,
}

impl ApplicationError for TrainDishApplicationServiceError {
    fn error_code(&self) -> u32 {
        match self {
            TrainDishApplicationServiceError::InvalidDishName(_) => 22001,
            TrainDishApplicationServiceError::InvalidAmount => 22002,
            TrainDishApplicationServiceError::InvalidTakeawayStation(_) => 22003,
            TrainDishApplicationServiceError::InvalidTakeawayShopName(_) => 22004,
            TrainDishApplicationServiceError::InvalidTakeawayName(_) => 22005,
            TrainDishApplicationServiceError::NoRelatedTrainOrder => 22006,
        }
    }

    fn error_message(&self) -> String {
        self.to_string()
    }
}

#[async_trait]
pub trait TrainDishApplicationService: 'static + Send + Sync {
    async fn order_dish(
        &self,
        command: OrderTrainDishCommand,
    ) -> Result<TransactionInfoDTO, Box<dyn ApplicationError>>;
}
