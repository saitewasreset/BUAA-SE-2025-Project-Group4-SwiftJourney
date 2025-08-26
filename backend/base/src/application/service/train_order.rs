use crate::{application::ApplicationError, domain::service::ServiceError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::transaction::TransactionInfoDTO;

// 定义请求数据结构
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TrainOrderRequestDTO {
    /// 车次号，例如："G53"
    pub train_number: String,
    /// 离开"始发站"的日期时间
    pub origin_departure_time: String,
    /// 起始站
    pub departure_station: String,
    /// 到达站
    pub arrival_station: String,
    /// 乘车人 Id（见`PersonalInfo`）
    pub personal_id: String,
    /// 座位类别，如：二等座
    pub seat_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OrderPackDTO {
    /// 原子操作，若为 true，则`order_list`中任意订单失败将回滚已成功的订单
    pub atomic: bool,
    /// 订单列表
    pub order_list: Vec<TrainOrderRequestDTO>,
}

pub type OrderPacksDTO = Vec<OrderPackDTO>;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CreateTrainOrderDTO {
    pub train_number: String,
    pub origin_departure_time: String,
    pub departure_station: String,
    pub arrival_station: String,
    pub personal_id: String,
    pub seat_type: String,
}

#[derive(Error, Debug)]
pub enum TrainOrderServiceError {
    /// 底层基础设施错误（如数据库访问失败）
    #[error("an infrastructure error occurred: {0}")]
    InfrastructureError(ServiceError),
    /// 会话无效
    #[error("invalid session id:")]
    InvalidSessionId,
    /// 车次号不存在
    #[error("invalid train number")]
    InvalidTrainNumber,
    /// 始发站或终到站不存在
    #[error("invalid station id")]
    InvalidStationId,
    /// 乘车人 Id 不存在，或未与当前用户绑定
    #[error("invalid passenger id")]
    InvalidPassengerId,
}

impl ApplicationError for TrainOrderServiceError {
    fn error_code(&self) -> u32 {
        match self {
            TrainOrderServiceError::InfrastructureError(_) => 500,
            TrainOrderServiceError::InvalidSessionId => 403,
            TrainOrderServiceError::InvalidTrainNumber => 404,
            TrainOrderServiceError::InvalidStationId => 404,
            TrainOrderServiceError::InvalidPassengerId => 404,
        }
    }

    fn error_message(&self) -> String {
        self.to_string()
    }
}

#[async_trait]
pub trait TrainOrderService: 'static + Send + Sync {
    /// 处理火车票订单包列表
    ///
    /// 此方法接收会话ID和订单包列表，验证并创建订单，然后创建交易
    /// 注意会话ID用于获取用户ID，订单包中包含原子性设置
    async fn process_train_order_packs(
        &self,
        session_id: String,
        order_packs: Vec<OrderPackDTO>,
    ) -> Result<TransactionInfoDTO, Box<dyn ApplicationError>>;
}



#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use uuid::Uuid;

    // 简单 MockError 用于模拟失败
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

    // 模拟服务实现
    struct TestTrainOrderService {
        fail: bool,
    }

    #[async_trait]
    impl TrainOrderService for TestTrainOrderService {
        async fn process_train_order_packs(
            &self,
            _session_id: String,
            _order_packs: Vec<OrderPackDTO>,
        ) -> Result<TransactionInfoDTO, Box<dyn ApplicationError>> {
            if self.fail {
                Err(Box::new(MockError))
            } else {
                Ok(TransactionInfoDTO {
                    transaction_id: Uuid::new_v4(),
                    amount: 100f64,
                    status: "success".to_string(),
                })
            }
        }
    }

    #[tokio::test]
    async fn test_process_train_order_packs_success() {
        let service = TestTrainOrderService { fail: false };
        let orders = vec![OrderPackDTO {
            atomic: true,
            order_list: vec![TrainOrderRequestDTO {
                train_number: "G53".into(),
                origin_departure_time: "2025-08-26T08:00:00Z".into(),
                departure_station: "StationA".into(),
                arrival_station: "StationB".into(),
                personal_id: Uuid::new_v4().to_string(),
                seat_type: "SecondClass".into(),
            }],
        }];

        let result = service.process_train_order_packs("session123".into(), orders).await;
        assert!(result.is_ok());
        let tx = result.unwrap();
        assert!(tx.amount > 0f64);
    }

    #[tokio::test]
    async fn test_process_train_order_packs_failure() {
        let service = TestTrainOrderService { fail: true };
        let orders = vec![];

        let result = service.process_train_order_packs("session123".into(), orders).await;
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert_eq!(err.error_code(), 9999);
        assert_eq!(err.error_message(), "Mock error");
    }

    // 测试 TrainOrderServiceError 的 error_code 和 error_message
    #[test]
    fn test_train_order_service_error_codes() {
        let e1 = TrainOrderServiceError::InfrastructureError(ServiceError::RelatedServiceError(anyhow::Error::new(MockError)));
        assert_eq!(e1.error_code(), 500);

        let e2 = TrainOrderServiceError::InvalidSessionId;
        assert_eq!(e2.error_code(), 403);

        let e3 = TrainOrderServiceError::InvalidTrainNumber;
        assert_eq!(e3.error_code(), 404);

        let e4 = TrainOrderServiceError::InvalidStationId;
        assert_eq!(e4.error_code(), 404);

        let e5 = TrainOrderServiceError::InvalidPassengerId;
        assert_eq!(e5.error_code(), 404);
    }

    #[test]
    fn test_train_order_service_error_messages() {
        let e1 = TrainOrderServiceError::InfrastructureError(ServiceError::RelatedServiceError(anyhow::Error::new(MockError)));
        assert!(e1.error_message().contains("a related service returned an error"));

        let e2 = TrainOrderServiceError::InvalidSessionId;
        assert!(e2.error_message().contains("invalid session"));

        let e3 = TrainOrderServiceError::InvalidTrainNumber;
        assert!(e3.error_message().contains("invalid train number"));

        let e4 = TrainOrderServiceError::InvalidStationId;
        assert!(e4.error_message().contains("invalid station id"));

        let e5 = TrainOrderServiceError::InvalidPassengerId;
        assert!(e5.error_message().contains("invalid passenger id"));
    }
}
