use std::collections::HashMap;

use crate::application::ApplicationError;
use crate::application::commands::hotel::{
    HotelInfoQuery, HotelOrderInfoQuery, HotelQuery, NewCommentCommand, QuotaQuery,
};
use async_trait::async_trait;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum HotelServiceError {
    #[error("invalid begin/end date: {0} - {1}")]
    InvalidDateRange(NaiveDate, NaiveDate),
    // 范围可能无效，所以或许使用字符串传递一个参数更好？
    #[error("invalid date range: {0}")]
    InvalidDateRangeMessage(String),
    #[error("invalid rating: {0}")]
    InvalidRating(f64),
    #[error("comment length exceed: {actual} < {limit}")]
    CommentLengthExceed { limit: usize, actual: usize },
    #[error("comment count exceed")]
    CommentCountExceed,
    #[error("target not found: {0}")]
    TargetNotFound(String),
}

impl ApplicationError for HotelServiceError {
    fn error_code(&self) -> u32 {
        match self {
            HotelServiceError::InvalidDateRange(_, _) => 21001,
            HotelServiceError::InvalidDateRangeMessage(_) => 21001,
            HotelServiceError::InvalidRating(_) => 21002,
            HotelServiceError::CommentLengthExceed { .. } => 21003,
            HotelServiceError::CommentCountExceed => 21004,
            HotelServiceError::TargetNotFound(_) => 404,
        }
    }

    fn error_message(&self) -> String {
        self.to_string()
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct HotelCommentQuotaDTO {
    pub quota: i32,
    pub used: i32,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NewHotelCommentDTO {
    pub hotel_id: Uuid,
    pub rating: f64,
    pub comment: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HotelGeneralInfoDTO {
    pub hotel_id: Uuid,
    pub name: String,
    pub picture: Option<String>,
    pub rating: f64,
    pub rating_count: i32,
    pub total_bookings: i32,
    pub price: f64,
    pub info: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HotelCommentDTO {
    pub user_name: String,
    pub comment_time: String,
    pub rating: f64,
    pub comment: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HotelDetailInfoDTO {
    pub hotel_id: String,
    pub name: String,
    pub address: String,
    pub phone: Vec<String>,
    pub info: String,

    pub picture: Option<Vec<String>>,
    pub rating: f64,
    pub rating_count: i32,
    pub total_bookings: i32,
    pub comments: Vec<HotelCommentDTO>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HotelRoomDetailInfoDTO {
    pub capacity: i32,
    pub remain_count: i32,
    pub price: f64,
}

#[async_trait]
pub trait HotelService: 'static + Send + Sync {
    async fn get_quota(
        &self,
        query: QuotaQuery,
    ) -> Result<HotelCommentQuotaDTO, Box<dyn ApplicationError>>;

    async fn new_comment(
        &self,
        command: NewCommentCommand,
    ) -> Result<(), Box<dyn ApplicationError>>;

    async fn query_hotels(
        &self,
        query: HotelQuery,
    ) -> Result<Vec<HotelGeneralInfoDTO>, Box<dyn ApplicationError>>;

    async fn query_hotel_info(
        &self,
        query: HotelInfoQuery,
    ) -> Result<HotelDetailInfoDTO, Box<dyn ApplicationError>>;

    async fn query_hotel_order_info(
        &self,
        query: HotelOrderInfoQuery,
    ) -> Result<HashMap<String, HotelRoomDetailInfoDTO>, Box<dyn ApplicationError>>;
}


#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use uuid::Uuid;
    use crate::application::commands::hotel::TargetType;

    // ---- 测试专用实现 ----
    struct TestHotelService {
        fail_quota: bool,
        fail_comment: bool,
        fail_hotels: bool,
        fail_info: bool,
        fail_order_info: bool,
    }

    #[async_trait]
    impl HotelService for TestHotelService {
        async fn get_quota(
            &self,
            _query: QuotaQuery,
        ) -> Result<HotelCommentQuotaDTO, Box<dyn ApplicationError>> {
            if self.fail_quota {
                Err(Box::new(HotelServiceError::TargetNotFound("quota".into())))
            } else {
                Ok(HotelCommentQuotaDTO { quota: 10, used: 3 })
            }
        }

        async fn new_comment(
            &self,
            _command: NewCommentCommand,
        ) -> Result<(), Box<dyn ApplicationError>> {
            if self.fail_comment {
                Err(Box::new(HotelServiceError::InvalidRating(-1.0)))
            } else {
                Ok(())
            }
        }

        async fn query_hotels(
            &self,
            _query: HotelQuery,
        ) -> Result<Vec<HotelGeneralInfoDTO>, Box<dyn ApplicationError>> {
            if self.fail_hotels {
                Err(Box::new(HotelServiceError::TargetNotFound("hotels".into())))
            } else {
                Ok(vec![HotelGeneralInfoDTO {
                    hotel_id: Uuid::new_v4(),
                    name: "Test Hotel".into(),
                    picture: None,
                    rating: 4.5,
                    rating_count: 100,
                    total_bookings: 200,
                    price: 500.0,
                    info: "Nice hotel".into(),
                }])
            }
        }

        async fn query_hotel_info(
            &self,
            _query: HotelInfoQuery,
        ) -> Result<HotelDetailInfoDTO, Box<dyn ApplicationError>> {
            if self.fail_info {
                Err(Box::new(HotelServiceError::TargetNotFound("hotel info".into())))
            } else {
                Ok(HotelDetailInfoDTO {
                    hotel_id: "1".into(),
                    name: "Test Hotel".into(),
                    address: "Somewhere".into(),
                    phone: vec!["123456".into()],
                    info: "Great place".into(),
                    picture: Some(vec!["pic.png".into()]),
                    rating: 4.2,
                    rating_count: 50,
                    total_bookings: 120,
                    comments: vec![HotelCommentDTO {
                        user_name: "Alice".into(),
                        comment_time: "2025-01-01".into(),
                        rating: 4.0,
                        comment: "Nice!".into(),
                    }],
                })
            }
        }

        async fn query_hotel_order_info(
            &self,
            _query: HotelOrderInfoQuery,
        ) -> Result<HashMap<String, HotelRoomDetailInfoDTO>, Box<dyn ApplicationError>> {
            if self.fail_order_info {
                Err(Box::new(HotelServiceError::TargetNotFound("order info".into())))
            } else {
                let mut map = HashMap::new();
                map.insert(
                    "Deluxe".into(),
                    HotelRoomDetailInfoDTO { capacity: 2, remain_count: 5, price: 1000.0 },
                );
                Ok(map)
            }
        }
    }

    // ---- Default 实现 ----
    impl Default for TestHotelService {
        fn default() -> Self {
            Self {
                fail_quota: false,
                fail_comment: false,
                fail_hotels: false,
                fail_info: false,
                fail_order_info: false,
            }
        }
    }

    // ---------------- 测试 -----------------

    #[tokio::test]
    async fn test_get_quota_success() {
        let service = TestHotelService::default();
        let query = QuotaQuery { session_id: "s1".into(), hotel_id: Uuid::new_v4() };
        let res = service.get_quota(query).await;
        assert!(res.is_ok());
        assert_eq!(res.unwrap().used, 3);
    }

    #[tokio::test]
    async fn test_get_quota_failure() {
        let service = TestHotelService { fail_quota: true, ..Default::default() };
        let query = QuotaQuery { session_id: "s2".into(), hotel_id: Uuid::new_v4() };
        let res = service.get_quota(query).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_new_comment_success() {
        let service = TestHotelService::default();
        // 这里假设 NewCommentCommand 至少有 hotel_id, rating, comment 字段
        let cmd = NewCommentCommand {
            session_id: "s1".into(),
            hotel_id: Uuid::new_v4(),
            rating: 4.5,
            comment: "Great hotel!".into(),
        };
        let res = service.new_comment(cmd).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_new_comment_failure() {
        let service = TestHotelService { fail_comment: true, ..Default::default() };
        let cmd = NewCommentCommand {
            session_id: "s1".into(),
            hotel_id: Uuid::new_v4(),
            rating: -1.0,
            comment: "bad".into(),
        };
        let res = service.new_comment(cmd).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_query_hotels_success() {
        let service = TestHotelService::default();
        let query = HotelQuery {
            session_id: "s3".into(),
            target: "Guangzhou".into(),
            target_type: TargetType::City,  // 这里用实际的枚举值
            search: Some("5-star".into()),
            begin_date: None,
            end_date: None,
        };
        let res = service.query_hotels(query).await;
        assert!(res.is_ok());
        assert_eq!(res.unwrap()[0].name, "Test Hotel");
    }

    #[tokio::test]
    async fn test_query_hotels_failure() {
        let service = TestHotelService { fail_hotels: true, ..Default::default() };
        let query = HotelQuery {
            session_id: "s4".into(),
            target: "Shenzhen".into(),
            target_type: TargetType::City,
            search: None,
            begin_date: None,
            end_date: None,
        };
        let res = service.query_hotels(query).await;
        assert!(res.is_err());
    }


    #[tokio::test]
    async fn test_query_hotel_info_success() {
        let service = TestHotelService::default();
        let query = HotelInfoQuery {
            session_id: "s5".into(),
            hotel_id: Uuid::new_v4(),
        };
        let res = service.query_hotel_info(query).await;
        assert!(res.is_ok());
        assert_eq!(res.unwrap().name, "Test Hotel");
    }

    #[tokio::test]
    async fn test_query_hotel_info_failure() {
        let service = TestHotelService { fail_info: true, ..Default::default() };
        let query = HotelInfoQuery {
            session_id: "s6".into(),
            hotel_id: Uuid::new_v4(),
        };
        let res = service.query_hotel_info(query).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_query_hotel_order_info_success() {
        let service = TestHotelService::default();
        let query = HotelOrderInfoQuery {
            session_id: "s7".into(),
            hotel_id: Uuid::new_v4(),
            begin_date: None,
            end_date: None,
        };
        let res = service.query_hotel_order_info(query).await;
        assert!(res.is_ok());
        assert!(res.unwrap().contains_key("Deluxe"));
    }

    #[tokio::test]
    async fn test_query_hotel_order_info_failure() {
        let service = TestHotelService { fail_order_info: true, ..Default::default() };
        let query = HotelOrderInfoQuery {
            session_id: "s8".into(),
            hotel_id: Uuid::new_v4(),
            begin_date: None,
            end_date: None,
        };
        let res = service.query_hotel_order_info(query).await;
        assert!(res.is_err());
    }

}

