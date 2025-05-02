//! 地理信息管理领域服务模块
//!
//! 提供与地理信息核心业务逻辑相关的服务接口定义和错误类型。
//! 这些服务操作涉及城市、省份以及城市到省的映射，包括添加、修改和删除城市等功能。
use crate::domain::RepositoryError;
use crate::domain::model::city::{City, CityId, CityName, ProvinceName};
use crate::domain::service::ServiceError;
use async_trait::async_trait;
use std::collections::HashMap;
use thiserror::Error;

/// 地理信息服务操作可能产生的错误类型
///
/// 包含了基础设施错误、业务规则违反等各种错误情况
#[derive(Error, Debug)]
pub enum GeoServiceError {
    /// 底层基础设施错误（如数据库访问失败）
    #[error("an infrastructure error occurred: {0}")]
    InfrastructureError(ServiceError),
    #[error("invalid city name: {0}")]
    InvalidCityName(String),
    #[error("no such city id: {0}")]
    NoSuchCityId(u64),
    #[error("city {0} already exists")]
    CityExists(String),
}

impl From<RepositoryError> for GeoServiceError {
    fn from(value: RepositoryError) -> Self {
        GeoServiceError::InfrastructureError(value.into())
    }
}

/// 地理信息领域服务接口
///
/// 定义了对地理信息实体进行业务操作的核心契约。
/// 所有方法都是异步的，返回实现了`Future` trait的结果。
#[async_trait]
pub trait GeoService: 'static + Send + Sync {
    /// 获取所有城市到省份的映射
    ///
    /// # Returns
    /// * `Ok(HashMap<ProvinceName, City>)` - 省份名称到城市实体的映射
    /// * `Err(GeoServiceError)` - 获取失败及原因
    ///
    /// # Errors
    /// * `InfrastructureError` - 基础设施错误（如数据库访问失败）
    async fn get_city_map(&self) -> Result<HashMap<ProvinceName, City>, GeoServiceError>;

    /// 根据城市名称查找城市
    ///
    /// # Arguments
    /// * `name` - 城市名称
    ///
    /// # Returns
    /// * `Ok(Option<City>)` - 匹配的城市实体（如果有）
    /// * `Err(GeoServiceError)` - 查找失败及原因
    ///
    /// # Errors
    /// * `InfrastructureError` - 基础设施错误（如数据库访问失败）
    async fn get_city_by_name(&self, name: &str) -> Result<Option<City>, GeoServiceError>;

    /// 添加新的城市
    ///
    /// # Arguments
    /// * `city` - 新的城市实体
    ///
    /// # Returns
    /// * `Ok(CityId)` - 新添加的城市ID
    /// * `Err(GeoServiceError)` - 添加失败及原因
    ///
    /// # Errors
    /// * `CityExists` - 城市已存在
    /// * `InfrastructureError` - 基础设施错误（如数据库访问失败）
    async fn add_city(&self, city: City) -> Result<CityId, GeoServiceError>;

    /// 删除指定的城市
    ///
    /// # Arguments
    /// * `city` - 要删除的城市实体
    ///
    /// # Returns
    /// * `Ok(())` - 删除成功
    /// * `Err(GeoServiceError)` - 删除失败及原因
    ///
    /// # Errors
    /// * `NoSuchCityId` - 指定城市ID的城市不存在
    /// * `InfrastructureError` - 基础设施错误（如数据库访问失败）
    async fn remove_city(&self, city: City) -> Result<(), GeoServiceError>;

    /// 修改现有的城市
    ///
    /// # Arguments
    /// * `city_id` - 城市ID
    /// * `city_name` - 新的城市名称
    /// * `province` - 新的省份名称
    ///
    /// # Returns
    /// * `Ok(())` - 修改成功
    /// * `Err(GeoServiceError)` - 修改失败及原因
    ///
    /// # Errors
    /// * `NoSuchCityId` - 指定城市ID的城市不存在
    /// * `InfrastructureError` - 基础设施错误（如数据库访问失败）
    async fn modify_city(
        &self,
        city_id: CityId,
        city_name: CityName,
        province: ProvinceName,
    ) -> Result<(), GeoServiceError>;
}
