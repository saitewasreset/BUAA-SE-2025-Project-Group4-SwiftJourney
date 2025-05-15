use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::application::commands::train_query::{
    DirectTrainQueryCommand, TransferTrainQueryCommand,
};
use crate::application::{ApplicationError, GeneralError};

// Step 2: Define `TrainQueryServiceError` for possible errors
// HINT: You may refer to RFC4 "直达车次查询（US1.2.1）" and "中转车次查询（US3.1.1）" for possible errors
// HINT: You may refer to `UserManagerError` for example
// Exercise 1.2.1D - 4: Your code here. (1 / 6)

#[derive(Error, Debug)]
pub enum TrainQueryServiceError {
    /// 会话无效
    #[error("invalid session id:")]
    InvalidSessionId,
    /// 始发站或终到站不存在
    #[error("invalid station id")]
    InvalidStationId,
    /// 始发城市或终到城市不存在
    #[error("invalid city id")]
    InvalidCityId,
    /// 不满足查询一致性要求
    #[error("inconsistent query")]
    InconsistentQuery,
}

// Step 3: Implement `ApplicationError` trait for `TrainQueryServiceError`
// 实现了该特征的错误类型可在Web框架中自动生成JSON响应
// HINT: You may refer to RFC4 "直达车次查询（US1.2.1）" and "中转车次查询（US3.1.1）" for possible
// error code and error message
// HINT: You may refer to `UserManagerError` for example
// Exercise 1.2.1D - 4: Your code here. (2 / 6)
impl TrainQueryServiceError {
    fn error_code(&self) -> u32 {
        match self {
            TrainQueryServiceError::InvalidSessionId => 403,
            TrainQueryServiceError::InvalidStationId => 404,
            TrainQueryServiceError::InvalidCityId => 404,
            TrainQueryServiceError::InconsistentQuery => 12001,
        }
    }

    fn error_message(&self) -> String {
        self.to_string()
    }
}

// Thinking 1.2.1D - 2：尝试探究为何`Result<ApiResponse<T>, ApplicationErrorBox>`
// 可以转化为HTTP响应，且响应体为符合条件的JSON？
// HINT: You may refer to https://actix.rs/docs/handlers#response-with-custom-type
// A: 当Result<T, E>中的T实现了`Responder`特征且E实现了`ApplicationError`特征时，actix-web会自动将其转换为HTTP响应。

// Step 4: Implement From<UserServiceError> for Box<dyn ApplicationError>
// HINT: You may refer to `UserManagerError` for example
// Exercise 1.2.1D - 4: Your code here. (3 / 6)
impl From<TrainQueryServiceError> for Box<dyn ApplicationError> {
    fn from(value: TrainQueryServiceError) -> Self {
        match value {
            TrainQueryServiceError::InvalidSessionId => GeneralError::InvalidSessionId.into(),
            TrainQueryServiceError::InvalidStationId => GeneralError::NotFound.into(),
            TrainQueryServiceError::InvalidCityId => GeneralError::NotFound.into(),
            TrainQueryServiceError::InconsistentQuery => {
                let msg = value.error_message();
                GeneralError::BadRequest(msg).into()
            }
        }
    }
}

// Step 5: Choose proper CQRS struct for user request
// HINT: You may refer to https://mp.weixin.qq.com/s/1rdnkROdcNw5ro4ct99SqQ for definition of CQRS
// HINT: You may refer to `UserManagerService` for example
// Exercise 1.2.1D - 4: Your code here. (4 / 6)

// Step 6: Define proper DTO(Data Transfer Object) for request json and response json
// HINT: You may refer to `UserManagerService` for example
// Exercise 1.2.1D - 4: Your code here. (5 / 6)


/// 单个席别信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeatInfoDTO {
    /// 席别名称
    pub seat_type: String,
    /// 剩余张数
    pub left: u32,
    /// 票价
    pub price: u32,
}

/// 单趟车次（一次发车）信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainInfoDTO {
    pub train_code: String,
    pub from_station: String,
    pub depart_time: String, // "2025-05-01 08:30"
    pub to_station: String,
    pub arrive_time: String, // "2025-05-01 12:06"
    pub duration_minutes: u32,
    pub seats: Vec<SeatInfoDTO>,
}

/// 直达查询的响应 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectTrainQueryDTO {
    pub solutions: Vec<TrainInfoDTO>,
}

/// 中转方案（两段行程 + 中转站）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferSolutionDTO {
    pub first_leg: TrainInfoDTO,
    pub second_leg: TrainInfoDTO,
    pub transfer_station: String,
    pub total_duration_minutes: u32,
}

/// 中转查询的响应 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferTrainQueryDTO {
    pub solutions: Vec<TransferSolutionDTO>,
}

// Thinking 1.2.1D - 3：DTO和CQRS结构的区别与联系是什么？它们中的数据是经过校验的，还是未经过校验的？

#[async_trait]
// Step 1: Define `TrainQueryService` application service
// Thinking 1.2.1D - 1：你认为`async_trait`宏的作用是什么？为什么需要使用它？
pub trait TrainQueryService {
    // Step 5: Define service using `async fn xxx(&self, command: XXXCommand)
    //     -> Result<DTO, Box<dyn ApplicationError>>;`
    // HINT: You may refer to `UserManagerService` for example
    // HINT: application service function should only receive CQRS parameter and
    // always return `Result<DTO, Box<dyn ApplicationError>>`
    // Exercise 1.2.1D - 4: Your code here. (6 / 6)
    // Good! Next, implement `TrainQueryService` in `base::infrastructure::application::service`
    /// 查询直达方案
    async fn query_direct_trains(
        &self,
        cmd: DirectTrainQueryCommand,
    ) -> Result<DirectTrainQueryDTO, Box<dyn ApplicationError>>;

    /// 查询中转方案
    async fn query_transfer_trains(
        &self,
        cmd: TransferTrainQueryCommand,
    ) -> Result<TransferTrainQueryDTO, Box<dyn ApplicationError>>;
}
