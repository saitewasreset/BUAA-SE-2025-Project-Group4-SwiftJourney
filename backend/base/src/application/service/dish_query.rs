use std::collections::HashMap;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::application::{ApplicationError, commands::dish_query::DishQueryDTO};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DishInfoDTO {
    pub available_time: Vec<String>,
    pub name: String,
    #[serde(rename = "type")]
    pub dish_type: String,
    pub picture: String,
    pub price: f64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TakeawayDishInfoDTO {
    pub name: String,
    pub picture: String,
    pub price: f64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TakeawayDTO {
    pub shop_name: String,
    pub dishes: Vec<TakeawayDishInfoDTO>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TrainDishInfoDTO {
    pub train_number: String,
    pub origin_departure_time: String,
    pub terminal_arrival_time: String,

    pub dishes: Vec<DishInfoDTO>,

    pub takeaway: HashMap<String, Vec<TakeawayDTO>>,

    pub can_booking: bool,
    pub reason: Option<String>,
}

#[async_trait]
pub trait DishQueryService: 'static + Send + Sync {
    async fn query_dish(
        &self,
        query: DishQueryDTO,
        session_id: String,
    ) -> Result<TrainDishInfoDTO, Box<dyn ApplicationError>>;
}


#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use crate::application::commands::dish_query::DishQueryDTO;

    // ------------------------------
    // 模拟 ApplicationError
    // ------------------------------
    #[derive(Debug)]
    struct MockError {
        code: u32,
        msg: String,
    }

    impl std::fmt::Display for MockError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.msg)
        }
    }

    impl std::error::Error for MockError {}

    impl ApplicationError for MockError {
        fn error_code(&self) -> u32 {
            self.code
        }

        fn error_message(&self) -> String {
            self.msg.clone()
        }
    }

    #[test]
    fn test_application_error_positive() {
        let err = MockError { code: 404, msg: "Not Found".to_string() };
        assert_eq!(err.error_code(), 404);
        assert_eq!(err.error_message(), "Not Found");
        assert_eq!(err.to_string(), "Not Found");
    }

    #[test]
    fn test_application_error_negative_empty_message() {
        let err = MockError { code: 500, msg: "".to_string() };
        assert_eq!(err.error_code(), 500);
        assert!(err.error_message().is_empty());
    }

    // ------------------------------
    // 模拟 DishQueryService
    // ------------------------------
    struct MockDishQueryService;

    #[async_trait]
    impl DishQueryService for MockDishQueryService {
        async fn query_dish(
            &self,
            query: DishQueryDTO,
            session_id: String,
        ) -> Result<TrainDishInfoDTO, Box<dyn ApplicationError>> {
            if session_id.is_empty() {
                return Err(Box::new(MockError { code: 401, msg: "Session missing".to_string() }));
            }
            if query.train_number == "INVALID" {
                return Err(Box::new(MockError { code: 404, msg: "Train not found".to_string() }));
            }

            let dish = DishInfoDTO {
                available_time: vec!["breakfast".to_string()],
                name: "牛肉饭".to_string(),
                dish_type: "main".to_string(),
                picture: "beef.png".to_string(),
                price: 25.5,
            };

            let takeaway_dish = TakeawayDishInfoDTO {
                name: "炸鸡".to_string(),
                picture: "chicken.png".to_string(),
                price: 18.0,
            };

            let takeaway = TakeawayDTO {
                shop_name: "肯德基".to_string(),
                dishes: vec![takeaway_dish],
            };

            let mut takeaway_map = HashMap::new();
            takeaway_map.insert("station1".to_string(), vec![takeaway]);

            Ok(TrainDishInfoDTO {
                train_number: query.train_number,
                origin_departure_time: "08:00".to_string(),
                terminal_arrival_time: "12:00".to_string(),
                dishes: vec![dish],
                takeaway: takeaway_map,
                can_booking: true,
                reason: None,
            })
        }
    }

    // ------------------------------
    // query_dish 测试
    // ------------------------------

    #[tokio::test]
    async fn test_query_dish_positive() {
        let service = MockDishQueryService;
        let query = DishQueryDTO {
            train_number: "G123".to_string(),
            origin_departure_time: "2025-08-26T08:00:00".to_string(),
        };

        let result = service.query_dish(query, "session123".to_string()).await;
        assert!(result.is_ok());

        let dto = result.unwrap();
        assert_eq!(dto.train_number, "G123");
        assert!(dto.can_booking);
        assert!(dto.reason.is_none());
        assert!(!dto.dishes.is_empty());
        assert!(!dto.takeaway.is_empty());
    }

    #[tokio::test]
    async fn test_query_dish_negative_empty_session() {
        let service = MockDishQueryService;
        let query = DishQueryDTO {
            train_number: "G123".to_string(),
            origin_departure_time: "2025-08-26T08:00:00".to_string(),
        };

        let result = service.query_dish(query, "".to_string()).await;
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.error_code(), 401);
        assert_eq!(err.error_message(), "Session missing");
    }

    #[tokio::test]
    async fn test_query_dish_negative_invalid_train() {
        let service = MockDishQueryService;
        let query = DishQueryDTO {
            train_number: "INVALID".to_string(),
            origin_departure_time: "2025-08-26T08:00:00".to_string(),
        };

        let result = service.query_dish(query, "session123".to_string()).await;
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.error_code(), 404);
        assert_eq!(err.error_message(), "Train not found");
    }

    // ------------------------------
    // DTO 序列化/反序列化测试
    // ------------------------------

    #[test]
    fn test_dish_info_dto_serialization() {
        let dish = DishInfoDTO {
            available_time: vec!["lunch".to_string()],
            name: "鱼香肉丝".to_string(),
            dish_type: "main".to_string(),
            picture: "fish.png".to_string(),
            price: 20.0,
        };

        let json = serde_json::to_string(&dish).unwrap();
        assert!(json.contains("鱼香肉丝"));

        let deser: DishInfoDTO = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.name, "鱼香肉丝");
        assert_eq!(deser.price, 20.0);
    }
}

