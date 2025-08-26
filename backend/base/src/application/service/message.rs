use crate::application::ApplicationError;
use crate::application::commands::message::HistoryMessageQuery;
use crate::domain::service::ServiceError;
use crate::domain::service::order::order_dto::OrderInfoDto;
use async_trait::async_trait;
use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Serialize)]
pub struct Message<T: Serialize> {
    #[serde(rename = "type")]
    pub type_name: String,
    pub data: T,
}

#[derive(Serialize, Clone)]
pub enum NotifyDTO {
    Order(OrderNotifyDTO),
    Trip(TripNotifyDTO),
}

#[derive(Serialize, Clone)]
pub struct OrderNotifyDTO {
    pub title: String,
    pub message_time: DateTimeWithTimeZone,
    pub order: Box<OrderInfoDto>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TripNotifyDTO {
    pub title: String,
    pub message_time: DateTimeWithTimeZone,
    pub train_number: String,
    pub departure_time: DateTimeWithTimeZone,
    pub departure_station: String,
    pub arrival_station: String,
}

#[derive(Debug, Error)]
pub enum MessageApplicationServiceError {
    #[error("an infrastructure error occurred")]
    InfrastructureError(ServiceError),
}

impl From<NotifyDTO> for Message<NotifyDTO> {
    fn from(notify: NotifyDTO) -> Self {
        Message {
            type_name: match notify {
                NotifyDTO::Order(_) => "order".to_string(),
                NotifyDTO::Trip(_) => "trip".to_string(),
            },
            data: notify,
        }
    }
}

impl ApplicationError for MessageApplicationServiceError {
    fn error_code(&self) -> u32 {
        match self {
            MessageApplicationServiceError::InfrastructureError(_) => 500, // Internal Server Error
        }
    }

    fn error_message(&self) -> String {
        self.to_string()
    }
}

#[async_trait]
pub trait MessageApplicationService: 'static + Send + Sync {
    async fn get_history(
        &self,
        query: HistoryMessageQuery,
    ) -> Result<Vec<NotifyDTO>, Box<dyn ApplicationError>>;
}



#[cfg(test)]
mod tests {
    use chrono::Utc;
    use super::*;
    use crate::domain::service::order::order_dto::HotelOrderDto;
    use crate::domain::service::order::order_dto::BaseOrderDto;

    // ---- 测试专用实现 ----
    struct TestMessageApplicationService {
        fail_history: bool,
    }

    #[async_trait]
    impl MessageApplicationService for TestMessageApplicationService {
        async fn get_history(
            &self,
            _query: HistoryMessageQuery,
        ) -> Result<Vec<NotifyDTO>, Box<dyn ApplicationError>> {
            if self.fail_history {
                Err(Box::new(TestError { code: 500, msg: "Failed to get history".into() }))
            } else {
                let now = Utc::now().into();
                let hotel_order = HotelOrderDto {
                    base: BaseOrderDto {
                        order_id: "order123".into(),
                        status: "completed".into(),
                        unit_price: 100.0,
                        amount: 1,
                        can_cancel: true,
                        reason: None,
                        order_type: "hotel".into(),
                    },
                    hotel_name: "Test Hotel".into(),
                    hotel_id: "hotel123".into(),
                    name: "Test Room".into(),
                    room_type: "Test Type".into(),
                    begin_date: "2022-01-01".into(),
                    end_date: "2022-01-02".into(),
                };

                let order_notify = OrderNotifyDTO {
                    title: "Order Completed".into(),
                    message_time: now,
                    order: Box::new(OrderInfoDto::Hotel(hotel_order)),
                };

                let trip_notify = TripNotifyDTO {
                    title: "Train Alert".into(),
                    message_time: now,
                    train_number: "G123".into(),
                    departure_time: now,
                    departure_station: "Station A".into(),
                    arrival_station: "Station B".into(),
                };

                Ok(vec![
                    NotifyDTO::Order(order_notify),
                    NotifyDTO::Trip(trip_notify),
                ])
            }
        }
    }

    // ---- 测试用 ApplicationError ----
    #[derive(Debug)]
    struct TestError {
        code: u32,
        msg: String,
    }

    impl std::fmt::Display for TestError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.msg)
        }
    }

    impl std::error::Error for TestError {}

    impl ApplicationError for TestError {
        fn error_code(&self) -> u32 { self.code }
        fn error_message(&self) -> String { self.msg.clone() }
    }

    // ---- 测试用例 ----

    #[tokio::test]
    async fn test_get_history_success() {
        let service = TestMessageApplicationService { fail_history: false };
        let query = HistoryMessageQuery { session_id: "s1".into(), limit: Some(10) };
        let res = service.get_history(query).await;
        assert!(res.is_ok());

        let history = res.unwrap();
        assert_eq!(history.len(), 2);

        match &history[0] {
            NotifyDTO::Order(order) => assert_eq!(order.title, "Order Completed"),
            _ => panic!("Expected OrderNotifyDTO"),
        }

        match &history[1] {
            NotifyDTO::Trip(trip) => assert_eq!(trip.title, "Train Alert"),
            _ => panic!("Expected TripNotifyDTO"),
        }
    }

    #[tokio::test]
    async fn test_get_history_failure() {
        let service = TestMessageApplicationService { fail_history: true };
        let query = HistoryMessageQuery { session_id: "s2".into(), limit: Some(10) };
        let res = service.get_history(query).await;
        assert!(res.is_err());

        let err = res.err().unwrap();
        assert_eq!(err.error_message(), "Failed to get history");
        assert_eq!(err.error_code(), 500);
    }
}