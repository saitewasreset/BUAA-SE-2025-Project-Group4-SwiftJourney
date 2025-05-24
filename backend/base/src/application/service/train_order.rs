use crate::application::ApplicationError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

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
    /// 处理火车票订单包列表
    ///
    /// 此方法接收会话ID和订单包列表，验证并创建订单，然后创建交易
    /// 注意会话ID用于获取用户ID，订单包中包含原子性设置
    async fn process_train_order_packs(
        &self,
        session_id: String,
        order_packs: Vec<OrderPackDTO>,
    ) -> Result<String, TrainOrderServiceError>;
}
