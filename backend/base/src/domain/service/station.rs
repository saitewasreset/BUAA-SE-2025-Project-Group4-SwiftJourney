//! 火车站管理领域服务模块
//!
//! 提供与火车站核心业务逻辑相关的服务接口定义和错误类型。
//! 这些服务操作涉及火车站的验证、管理和配置，包括添加、修改和删除火车站等功能。
use crate::domain::RepositoryError;
use crate::domain::model::city::CityId;
use crate::domain::model::station::{Station, StationId};
use crate::domain::service::ServiceError;
use crate::domain::service::geo::GeoServiceError;
use async_trait::async_trait;
use thiserror::Error;

/// 火车站服务操作可能产生的错误类型
///
/// 包含了基础设施错误、业务规则违反等各种错误情况
#[derive(Error, Debug)]
pub enum StationServiceError {
    /// 底层基础设施错误（如数据库访问失败）
    #[error("an infrastructure error occurred: {0}")]
    InfrastructureError(ServiceError),

    #[error("invalid geo info: {0}")]
    InvalidGeoInfo(GeoServiceError),

    #[error("invalid station name: {0}")]
    InvalidStationName(String),

    #[error("no such station id: {0}")]
    NoSuchStationId(u64),

    #[error("station {0} already exists")]
    StationExists(String),
}

impl From<RepositoryError> for StationServiceError {
    fn from(value: RepositoryError) -> Self {
        StationServiceError::InfrastructureError(value.into())
    }
}

impl From<GeoServiceError> for StationServiceError {
    fn from(value: GeoServiceError) -> Self {
        match value {
            GeoServiceError::InfrastructureError(e) => StationServiceError::InfrastructureError(e),
            x => StationServiceError::InvalidGeoInfo(x),
        }
    }
}

/// 火车站管理领域服务接口
///
/// 定义了对火车站实体进行业务操作的核心契约。
/// 所有方法都是异步的，返回实现了`Future` trait的结果。
#[async_trait]
pub trait StationService {
    /// 获取所有火车站信息
    ///
    /// # Returns
    /// * `Ok(Vec<Station>)` - 所有火车站的列表
    /// * `Err(StationServiceError)` - 获取失败及原因
    ///
    /// # Errors
    /// * `InfrastructureError` - 基础设施错误（如数据库访问失败）
    async fn get_stations(&self) -> Result<Vec<Station>, StationServiceError>;

    /// 根据城市ID获取所有火车站
    ///
    /// # Arguments
    /// * `city_id` - 城市ID
    ///
    /// # Returns
    /// * `Ok(Vec<Station>)` - 该城市下的所有火车站的列表
    /// * `Err(StationServiceError)` - 获取失败及原因
    ///
    /// # Errors
    /// * `InfrastructureError` - 基础设施错误（如数据库访问失败）
    async fn get_station_by_city(
        &self,
        city_id: CityId,
    ) -> Result<Vec<Station>, StationServiceError>;

    /// 根据车站名称获取火车站
    ///
    /// # Notes
    ///
    /// 我们假定，没有重名的车站
    ///
    /// # Arguments
    /// * `station_name` - 车站名称
    ///
    /// # Returns
    /// * `Ok(Option<Station>)` - 匹配的火车站（如果有）
    /// * `Err(StationServiceError)` - 获取失败及原因
    ///
    /// # Errors
    /// * `InfrastructureError` - 基础设施错误（如数据库访问失败）
    async fn get_station_by_name(
        &self,
        station_name: String,
    ) -> Result<Option<Station>, StationServiceError>;

    /// 添加新的火车站
    ///
    /// # Arguments
    /// * `station_name` - 车站名称
    /// * `city_name` - 所属城市名称
    ///
    /// # Returns
    /// * `Ok(StationId)` - 新添加的火车站ID
    /// * `Err(StationServiceError)` - 添加失败及原因
    ///
    /// # Errors
    /// * `InvalidGeoInfo` - 地理信息无效
    /// * `StationExists` - 火车站已存在
    /// * `InfrastructureError` - 基础设施错误（如数据库访问失败）
    async fn add_station(
        &self,
        station_name: String,
        city_name: String,
    ) -> Result<StationId, StationServiceError>;

    /// 修改现有的火车站
    ///
    /// # Arguments
    /// * `station_id` - 火车站ID
    /// * `station_name` - 新的车站名称
    /// * `city_name` - 新的所属城市名称
    ///
    /// # Returns
    /// * `Ok(())` - 修改成功
    /// * `Err(StationServiceError)` - 修改失败及原因
    ///
    /// # Errors
    /// * `NoSuchStationId` - 指定车站ID的火车站不存在
    /// * `InvalidGeoInfo` - 地理信息无效
    /// * `InfrastructureError` - 基础设施错误（如数据库访问失败）
    async fn modify_station(
        &self,
        station_id: StationId,
        station_name: String,
        city_name: String,
    ) -> Result<(), StationServiceError>;

    /// 删除指定的火车站
    ///
    /// # Arguments
    /// * `station` - 要删除的火车站实体
    ///
    /// # Returns
    /// * `Ok(())` - 删除成功
    /// * `Err(StationServiceError)` - 删除失败及原因
    ///
    /// # Errors
    /// * `NoSuchStationId` - 指定车站ID的火车站不存在
    /// * `InfrastructureError` - 基础设施错误（如数据库访问失败）
    async fn delete_station(&self, station: Station) -> Result<(), StationServiceError>;

    async fn get_station_by_city_name(
        &self,
        city_name: &str,
    ) -> Result<Vec<Station>, StationServiceError>;

    /// 根据城市名组合出所有 `(from_station, to_station)` 对
    async fn station_pairs_by_city(
        &self,
        from_city: &str,
        to_city: &str,
    ) -> Result<Vec<(StationId, StationId)>, StationServiceError>;
}
