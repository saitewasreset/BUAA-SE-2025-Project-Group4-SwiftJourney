use crate::application::ApplicationError;
use crate::application::commands::train_dish::OrderTrainDishCommand;
use crate::application::service::transaction::TransactionInfoDTO;
use crate::domain::model::dish::{DishId, DishTime};
use crate::domain::model::personal_info::PersonalInfoId;
use crate::domain::model::station::StationId;
use crate::domain::model::takeaway::TakeawayDishId;
use crate::domain::model::train::TrainId;
use async_trait::async_trait;
use rust_decimal::Decimal;
use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DishOrderRequestDTO {
    pub name: String,
    pub personal_id: Uuid,
    pub amount: u32,
    pub dish_time: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VerifiedDishOrderRequest {
    pub dish_id: DishId,
    pub train_id: TrainId,
    pub personal_id: PersonalInfoId,
    pub unit_price: Decimal,
    pub amount: Decimal,
    pub dish_time: DishTime,
    pub active_time: DateTimeWithTimeZone,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TakeawayOrderRequestDTO {
    pub station: String,
    pub shop_name: String,
    pub name: String,
    pub personal_id: Uuid,
    pub amount: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VerifiedTakeawayOrderRequest {
    pub takeaway_dish_id: TakeawayDishId,
    pub train_id: TrainId,
    pub station_id: StationId,
    pub personal_id: PersonalInfoId,
    pub unit_price: Decimal,
    pub amount: Decimal,
    pub active_time: DateTimeWithTimeZone,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TrainDishOrderRequestDTO {
    pub train_number: String,
    pub origin_departure_time: String,

    pub dishes: Vec<DishOrderRequestDTO>,
    pub takeaway: Vec<TakeawayOrderRequestDTO>,
}

#[async_trait]
pub trait TrainDishApplicationService: 'static + Send + Sync {
    async fn order_dish(
        &self,
        command: OrderTrainDishCommand,
    ) -> Result<TransactionInfoDTO, Box<dyn ApplicationError>>;
}


#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use uuid::Uuid;

    #[test]
    fn test_error_code() {
        let e1 = TrainDishApplicationServiceError::InvalidDishName("Sushi".into());
        assert_eq!(e1.error_code(), 22001);

        let e2 = TrainDishApplicationServiceError::InvalidAmount;
        assert_eq!(e2.error_code(), 22002);

        let e3 = TrainDishApplicationServiceError::InvalidTakeawayStation("StationX".into());
        assert_eq!(e3.error_code(), 22003);

        let e4 = TrainDishApplicationServiceError::InvalidTakeawayShopName("ShopY".into());
        assert_eq!(e4.error_code(), 22004);

        let e5 = TrainDishApplicationServiceError::InvalidTakeawayName("Burger".into());
        assert_eq!(e5.error_code(), 22005);

        let e6 = TrainDishApplicationServiceError::NoRelatedTrainOrder;
        assert_eq!(e6.error_code(), 22006);
    }

    #[test]
    fn test_error_message() {
        let e1 = TrainDishApplicationServiceError::InvalidDishName("Sushi".into());
        assert!(e1.error_message().contains("Sushi"));

        let e2 = TrainDishApplicationServiceError::InvalidAmount;
        assert!(e2.error_message().contains("Invalid dish name"));

        let e3 = TrainDishApplicationServiceError::InvalidTakeawayStation("StationX".into());
        assert!(e3.error_message().contains("StationX"));

        let e4 = TrainDishApplicationServiceError::InvalidTakeawayShopName("ShopY".into());
        assert!(e4.error_message().contains("ShopY"));

        let e5 = TrainDishApplicationServiceError::InvalidTakeawayName("Burger".into());
        assert!(e5.error_message().contains("Burger"));

        let e6 = TrainDishApplicationServiceError::NoRelatedTrainOrder;
        assert_eq!(e6.error_message(), "No related train order found");
    }

    #[derive(Debug)]
    struct MockError;

    impl std::fmt::Display for MockError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Mock error")
        }
    }
    impl std::error::Error for MockError {}
    impl ApplicationError for MockError {
        fn error_code(&self) -> u32 { 9999 }
        fn error_message(&self) -> String { "Mock error".to_string() }
    }

    struct TestTrainDishService {
        fail: bool,
    }

    #[async_trait]
    impl TrainDishApplicationService for TestTrainDishService {
        async fn order_dish(
            &self,
            _command: OrderTrainDishCommand,
        ) -> Result<TransactionInfoDTO, Box<dyn ApplicationError>> {
            if self.fail {
                Err(Box::new(MockError))
            } else {
                Ok(TransactionInfoDTO {
                    transaction_id: Uuid::new_v4(),
                    amount: 100.0f64,
                    status: "success".into(),
                })
            }
        }
    }

    #[tokio::test]
    async fn test_order_dish_success() {
        let service = TestTrainDishService { fail: false };

        let command = OrderTrainDishCommand {
            session_id: "session1".into(),
            info: TrainDishOrderRequestDTO {
                train_number: "G123".into(),
                origin_departure_time: "2025-08-26T08:00:00Z".into(),
                dishes: vec![
                    DishOrderRequestDTO {
                        name: "Beef Noodles".into(),
                        personal_id: Uuid::new_v4(),
                        amount: 2,
                        dish_time: "2025-08-26T12:00:00Z".into(),
                    }
                ],
                takeaway: vec![
                    TakeawayOrderRequestDTO {
                        station: "StationA".into(),
                        shop_name: "ShopX".into(),
                        name: "Burger".into(),
                        personal_id: Uuid::new_v4(),
                        amount: 1,
                    }
                ],
            }
        };
        let result = service.order_dish(command).await;
        assert!(result.is_ok());
        let tx = result.unwrap();
        assert!(tx.amount > 0.0f64);
    }

    #[tokio::test]
    async fn test_order_dish_failure() {
        let service = TestTrainDishService { fail: true };

        let command = OrderTrainDishCommand {
            session_id: "session1".into(),
            info: TrainDishOrderRequestDTO {
                train_number: "G123".into(),
                origin_departure_time: "2025-08-26T08:00:00Z".into(),
                dishes: vec![],
                takeaway: vec![],
            }
        };

        let result = service.order_dish(command).await;
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert_eq!(err.error_code(), 9999);
        assert_eq!(err.error_message(), "Mock error");
    }
}