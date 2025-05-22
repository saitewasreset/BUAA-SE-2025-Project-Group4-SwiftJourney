use crate::application::{ApplicationError, commands::train_order::CreateTrainOrderCommand};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

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
    async fn create_train_order(
        &self,
        dto: CreateTrainOrderDTO,
    ) -> Result<CreateTrainOrderCommand, TrainOrderServiceError>;

    /// 基于交易ID和订单UUID列表进行退款
    async fn refund_order_transaction(
        &self,
        transaction_id: Uuid,
        order_uuids: Vec<Uuid>,
    ) -> Result<Uuid, TrainOrderServiceError>;
}
