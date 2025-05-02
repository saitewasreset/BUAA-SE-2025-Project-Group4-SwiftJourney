//! 列车类型管理领域服务模块
//!
//! 提供与列车类型核心业务逻辑相关的服务接口定义和错误类型。
//! 这些服务操作涉及列车类型的验证、管理和配置，包括添加、修改和删除列车类型等功能。
use crate::domain::RepositoryError;
use crate::domain::model::route::RouteId;
use crate::domain::model::train::{SeatType, SeatTypeName, Train, TrainId, TrainNumber, TrainType};
use crate::domain::model::train_schedule::SeatId;
use crate::domain::service::ServiceError;
use crate::{Unverified, Verified};
use async_trait::async_trait;
use std::collections::HashMap;
use thiserror::Error;

/// 列车类型配置服务操作可能产生的错误类型
///
/// 包含了基础设施错误、业务规则违反等各种错误情况
#[derive(Error, Debug)]
pub enum TrainTypeConfigurationServiceError {
    /// 底层基础设施错误（如数据库访问失败）
    #[error("an infrastructure error occurred: {0}")]
    InfrastructureError(ServiceError),
    #[error("invalid seat type {1} for train id: {0}")]
    InvalidSeatType(u64, String),
    #[error("invalid train number: {0}")]
    InvalidTrainNumber(String),
    #[error("invalid train type: {0}")]
    InvalidTrainType(String),
    #[error("train type {0} already exists")]
    TrainTypeExists(u64),
    #[error("no such train id: {0}")]
    NoSuchTrainId(u64),
}

impl From<RepositoryError> for TrainTypeConfigurationServiceError {
    fn from(value: RepositoryError) -> Self {
        Self::InfrastructureError(ServiceError::RepositoryError(value))
    }
}

/// 列车类型配置领域服务接口
///
/// 定义了对列车类型实体进行业务操作的核心契约。
/// 所有方法都是异步的，返回实现了`Future` trait的结果。
#[async_trait]
pub trait TrainTypeConfigurationService {
    /// 验证座位类型名称
    ///
    /// # Arguments
    /// * `train_id` - 车次ID
    /// * `seat_type_name` - 未验证的座位类型名称
    ///
    /// # Returns
    /// * `Ok(SeatTypeName<Verified>)` - 验证成功的座位类型名称
    /// * `Err(TrainTypeConfigurationServiceError)` - 验证失败及原因
    ///
    /// # Errors
    /// * `InvalidSeatType` - 座位类型名称无效
    /// * `InfrastructureError` - 基础设施错误（如数据库访问失败）
    async fn verify_seat_type_name(
        &self,
        train_id: TrainId,
        seat_type_name: SeatTypeName<Unverified>,
    ) -> Result<SeatTypeName<Verified>, TrainTypeConfigurationServiceError>;

    /// 验证车次编号
    ///
    /// # Arguments
    /// * `train_number` - 未验证的车次编号
    ///
    /// # Returns
    /// * `Ok(TrainNumber<Verified>)` - 验证成功的车次编号
    /// * `Err(TrainTypeConfigurationServiceError)` - 验证失败及原因
    ///
    /// # Errors
    /// * `InvalidTrainNumber` - 车次编号无效
    /// * `InfrastructureError` - 基础设施错误（如数据库访问失败）
    async fn verify_train_number(
        &self,
        train_number: TrainNumber<Unverified>,
    ) -> Result<TrainNumber<Verified>, TrainTypeConfigurationServiceError>;

    /// 验证列车类型
    ///
    /// # Arguments
    /// * `train_type` - 未验证的列车类型
    ///
    /// # Returns
    /// * `Ok(TrainType<Verified>)` - 验证成功的列车类型
    /// * `Err(TrainTypeConfigurationServiceError)` - 验证失败及原因
    ///
    /// # Errors
    /// * `InvalidTrainType` - 列车类型无效
    /// * `InfrastructureError` - 基础设施错误（如数据库访问失败）
    async fn verify_train_type(
        &self,
        train_type: TrainType<Unverified>,
    ) -> Result<TrainType<Verified>, TrainTypeConfigurationServiceError>;

    /// 获取指定车次的座位ID映射
    ///
    /// # Arguments
    /// * `train_id` - 车次ID
    ///
    /// # Returns
    /// * `Ok(HashMap<SeatTypeName<Verified>, Vec<SeatId>>)` - 座位类型名到座位ID列表的映射
    /// * `Err(TrainTypeConfigurationServiceError)` - 获取失败及原因
    ///
    /// # Errors
    /// * `InfrastructureError` - 基础设施错误（如数据库访问失败）
    async fn get_seat_id_map(
        &self,
        train_id: TrainId,
    ) -> Result<HashMap<SeatTypeName<Verified>, Vec<SeatId>>, TrainTypeConfigurationServiceError>;

    /// 获取所有列车车次信息
    ///
    /// # Returns
    /// * `Ok(Vec<Train>)` - 所有列车车次的列表
    /// * `Err(TrainTypeConfigurationServiceError)` - 获取失败及原因
    ///
    /// # Errors
    /// * `InfrastructureError` - 基础设施错误（如数据库访问失败）
    async fn get_trains(&self) -> Result<Vec<Train>, TrainTypeConfigurationServiceError>;

    /// 根据车次编号查找列车车次
    ///
    /// # Arguments
    /// * `train_number` - 已验证的车次编号
    ///
    /// # Returns
    /// * `Ok(Train)` - 匹配的列车车次
    /// * `Err(TrainTypeConfigurationServiceError)` - 查找失败及原因
    ///
    /// # Errors
    /// * `InfrastructureError` - 基础设施错误（如数据库访问失败）
    async fn get_train_by_number(
        &self,
        train_number: TrainNumber<Verified>,
    ) -> Result<Train, TrainTypeConfigurationServiceError>;

    /// 添加新的列车类型
    ///
    /// # Arguments
    /// * `train_number` - 已验证的车次编号
    /// * `train_type` - 已验证的列车类型
    /// * `seat_configuration` - 座位配置列表
    /// * `default_route_id` - 默认路线ID
    ///
    /// # Returns
    /// * `Ok(TrainId)` - 新添加的列车车次ID
    /// * `Err(TrainTypeConfigurationServiceError)` - 添加失败及原因
    ///
    /// # Errors
    /// * `TrainTypeExists` - 列车类型已存在
    /// * `InfrastructureError` - 基础设施错误（如数据库访问失败）
    async fn add_train_type(
        &self,
        train_number: TrainNumber<Verified>,
        train_type: TrainType<Verified>,
        seat_configuration: Vec<SeatType>,
        default_route_id: RouteId,
    ) -> Result<TrainId, TrainTypeConfigurationServiceError>;

    /// 修改现有的列车类型
    ///
    /// # Arguments
    /// * `train_id` - 车次ID
    /// * `train_number` - 已验证的车次编号
    /// * `train_type` - 已验证的列车类型
    /// * `seat_configuration` - 座位配置列表
    /// * `default_route_id` - 默认路线ID
    ///
    /// # Returns
    /// * `Ok(())` - 修改成功
    /// * `Err(TrainTypeConfigurationServiceError)` - 修改失败及原因
    ///
    /// # Errors
    /// * `NoSuchTrainId` - 指定车次ID的列车车次不存在
    /// * `InfrastructureError` - 基础设施错误（如数据库访问失败）
    async fn modify_train_type(
        &self,
        train_id: TrainId,
        train_number: TrainNumber<Verified>,
        train_type: TrainType<Verified>,
        seat_configuration: Vec<SeatType>,
        default_route_id: RouteId,
    ) -> Result<(), TrainTypeConfigurationServiceError>;

    /// 删除指定的列车类型
    ///
    /// # Arguments
    /// * `train` - 要删除的列车车次实体
    ///
    /// # Returns
    /// * `Ok(())` - 删除成功
    /// * `Err(TrainTypeConfigurationServiceError)` - 删除失败及原因
    ///
    /// # Errors
    /// * `NoSuchTrainId` - 指定车次ID的列车车次不存在
    /// * `InfrastructureError` - 基础设施错误（如数据库访问失败）
    async fn remove_train_type(
        &self,
        train: Train,
    ) -> Result<(), TrainTypeConfigurationServiceError>;
}
