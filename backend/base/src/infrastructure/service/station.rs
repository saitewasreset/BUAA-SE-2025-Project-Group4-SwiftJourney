//! 火车站服务基础设施实现模块
//!
//! 提供`StationService` trait的具体实现，将火车站领域逻辑与底层仓储和地理服务连接起来。
//! 本实现是泛型的，可以适配不同的仓储和地理服务实现。
//!
//! # 主要功能
//! - 火车站的增删改查
//! - 车站与城市的关联管理
//! - 车站名称验证和唯一性检查
use crate::domain::Identifiable;
use crate::domain::model::city::CityId;
use crate::domain::model::station::{Station, StationId};
use crate::domain::repository::station::StationRepository;
use crate::domain::service::geo::{GeoService, GeoServiceError};
use crate::domain::service::station::{StationService, StationServiceError};
use async_trait::async_trait;
use std::sync::Arc;

/// 火车站服务具体实现
///
/// 泛型参数：
/// - `R`: 车站仓储实现
/// - `C`: 地理服务实现
///
/// # 类型约束
/// - `R`必须实现`StationRepository` trait
/// - `C`必须实现`GeoService` trait
pub struct StationServiceImpl<R, C>
where
    R: StationRepository,
    C: GeoService,
{
    /// 车站仓储实例
    station_repository: Arc<R>,
    /// 地理服务实例
    geo_service: Arc<C>,
}

impl<R, C> StationServiceImpl<R, C>
where
    R: StationRepository,
    C: GeoService,
{
    /// 创建新的火车站服务实例
    ///
    /// # Arguments
    /// * `station_repository` - 车站仓储实现
    /// * `geo_service` - 地理服务实现
    pub fn new(station_repository: Arc<R>, geo_service: Arc<C>) -> Self {
        StationServiceImpl {
            station_repository,
            geo_service,
        }
    }
}

#[async_trait]
impl<R, C> StationService for StationServiceImpl<R, C>
where
    R: StationRepository,
    C: GeoService,
{
    /// 获取所有火车站实现
    ///
    /// # Returns
    /// * `Ok(Vec<Station>)` - 所有火车站的列表
    /// * `Err(StationServiceError)` - 获取失败及原因
    ///
    /// # Errors
    /// * `InfrastructureError` - 仓储访问错误
    async fn get_stations(&self) -> Result<Vec<Station>, StationServiceError> {
        let result = self.station_repository.load().await?;

        Ok(result)
    }

    /// 根据城市ID获取火车站实现
    ///
    /// # Arguments
    /// * `city_id` - 城市ID
    ///
    /// # Returns
    /// * `Ok(Vec<Station>)` - 该城市下的所有火车站的列表
    /// * `Err(StationServiceError)` - 获取失败及原因
    ///
    /// # Errors
    /// * `InfrastructureError` - 仓储访问错误
    async fn get_station_by_city(
        &self,
        city_id: CityId,
    ) -> Result<Vec<Station>, StationServiceError> {
        let result = self.station_repository.find_by_city(city_id).await?;

        Ok(result)
    }

    /// 根据车站名称获取火车站实现
    ///
    /// # Arguments
    /// * `station_name` - 车站名称字符串
    ///
    /// # Returns
    /// * `Ok(Option<Station>)` - 匹配的火车站（如果有）
    /// * `Err(StationServiceError)` - 获取失败及原因
    ///
    /// # Notes
    /// 假设车站名称是唯一的，如果找到多个同名车站，只返回第一个
    ///
    /// # Errors
    /// * `InfrastructureError` - 仓储访问错误
    async fn get_station_by_name(
        &self,
        station_name: String,
    ) -> Result<Option<Station>, StationServiceError> {
        let result = self.station_repository.find_by_name(&station_name).await?;

        Ok(result)
    }

    /// 添加新火车站实现
    ///
    /// # Arguments
    /// * `station_name` - 车站名称
    /// * `city_name` - 所属城市名称
    ///
    /// # Returns
    /// * `Ok(StationId)` - 新添加的车站ID
    /// * `Err(StationServiceError)` - 添加失败及原因
    ///
    /// # Notes
    /// 添加前会验证城市是否存在
    ///
    /// # Errors
    /// * `InvalidGeoInfo` - 城市不存在或无效
    /// * `InfrastructureError` - 仓储访问错误
    async fn add_station(
        &self,
        station_name: String,
        city_name: String,
    ) -> Result<StationId, StationServiceError> {
        if let Some(city) = self.geo_service.get_city_by_name(&city_name).await? {
            let mut station = Station::new(
                None,
                station_name.clone(),
                city.get_id().expect("saved city should have id"),
            );
            self.station_repository.save(&mut station).await?;
            Ok(station.get_id().expect("new station should have id"))
        } else {
            Err(StationServiceError::InvalidGeoInfo(
                GeoServiceError::InvalidCityName(city_name),
            ))
        }
    }

    /// 修改火车站信息实现
    ///
    /// # Arguments
    /// * `station_id` - 车站ID
    /// * `station_name` - 新的车站名称
    /// * `city_name` - 新的所属城市名称
    ///
    /// # Returns
    /// * `Ok(())` - 修改成功
    /// * `Err(StationServiceError)` - 修改失败及原因
    ///
    /// # Notes
    /// 修改前会验证车站和城市是否存在
    ///
    /// # Errors
    /// * `NoSuchStationId` - 车站不存在
    /// * `InvalidGeoInfo` - 城市不存在或无效
    /// * `InfrastructureError` - 仓储访问错误
    async fn modify_station(
        &self,
        station_id: StationId,
        station_name: String,
        city_name: String,
    ) -> Result<(), StationServiceError> {
        if let Some(city) = self.geo_service.get_city_by_name(&city_name).await? {
            if self.station_repository.find(station_id).await?.is_some() {
                let mut station = Station::new(
                    Some(station_id),
                    station_name,
                    city.get_id().expect("saved city should have id"),
                );
                self.station_repository.save(&mut station).await?;

                Ok(())
            } else {
                Err(StationServiceError::NoSuchStationId(station_id.into()))
            }
        } else {
            Err(StationServiceError::InvalidGeoInfo(
                GeoServiceError::InvalidCityName(city_name),
            ))
        }
    }

    /// 删除火车站实现
    ///
    /// # Arguments
    /// * `station` - 要删除的车站实体
    ///
    /// # Returns
    /// * `Ok(())` - 删除成功
    /// * `Err(StationServiceError)` - 删除失败及原因
    ///
    /// # Errors
    /// * `InfrastructureError` - 仓储访问错误
    async fn delete_station(&self, station: Station) -> Result<(), StationServiceError> {
        self.station_repository.remove(station).await?;

        Ok(())
    }

    async fn get_station_by_city_name(
        &self,
        city_name: &str,
    ) -> Result<Vec<Station>, StationServiceError> {
        if let Some(city) = self.geo_service.get_city_by_name(city_name).await? {
            self.station_repository
                .find_by_city(city.get_id().unwrap())
                .await
                .map_err(Into::into)
        } else {
            Err(StationServiceError::InvalidGeoInfo(
                GeoServiceError::InvalidCityName(city_name.to_owned()),
            ))
        }
    }

    async fn station_pairs_by_city(
        &self,
        from_city: &str,
        to_city: &str,
    ) -> Result<Vec<(StationId, StationId)>, StationServiceError> {
        let from_list = self.get_station_by_city_name(from_city).await?;
        let to_list   = self.get_station_by_city_name(to_city).await?;
        Ok(from_list
            .iter()
            .flat_map(|f| to_list.iter().map(move |t| (f.get_id().unwrap(), t.get_id().unwrap())))
            .collect())
    }
}
