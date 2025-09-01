//! 地理信息服务基础设施实现模块
//!
//! 提供`GeoService` trait的具体实现，将地理信息领域逻辑与底层仓储连接起来。
//! 本实现是泛型的，可以适配不同的城市仓储实现。
//!
//! # 主要功能
//! - 城市到省份的映射管理
//! - 城市信息的增删改查
//! - 城市名称和省份名称的关联查询
use crate::domain::model::city::{City, CityId, CityName, ProvinceName};
use crate::domain::repository::city::CityRepository;
use crate::domain::service::geo::{GeoService, GeoServiceError};
use crate::domain::Identifiable;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

/// 地理信息服务具体实现
///
/// 泛型参数：
/// - `R`: 城市仓储实现
///
/// # 类型约束
/// - `R`必须实现`CityRepository` trait
pub struct GeoServiceImpl<R>
where
    R: CityRepository,
{
    /// 城市仓储实例
    city_repository: Arc<R>,
}

impl<R> GeoServiceImpl<R>
where
    R: CityRepository,
{
    /// 创建新的地理服务实例
    ///
    /// # Arguments
    /// * `city_repository` - 城市仓储实现
    pub fn new(city_repository: Arc<R>) -> Self {
        GeoServiceImpl { city_repository }
    }
}

#[async_trait]
impl<R> GeoService for GeoServiceImpl<R>
where
    R: CityRepository,
{
    /// 获取所有城市到省份的映射实现
    ///
    /// # Returns
    /// * `Ok(HashMap<ProvinceName, Vec<City>>)` - 省份名称到城市实体的映射
    /// * `Err(GeoServiceError)` - 获取失败及原因
    ///
    /// # Notes
    /// 返回的映射中，每个省份只包含一个代表城市（如果同一省份有多个城市，只取第一个）
    ///
    /// # Errors
    /// * `InfrastructureError` - 仓储访问错误
    async fn get_city_map(&self) -> Result<HashMap<ProvinceName, Vec<City>>, GeoServiceError> {
        let cities = self.city_repository.load().await?;

        let mut result: HashMap<ProvinceName, Vec<City>> = HashMap::new();

        for city in cities {
            let province = city.province().clone();
            result.entry(province).or_default().push(city);
        }

        Ok(result)
    }

    /// 根据城市名称查找城市实现
    ///
    /// # Arguments
    /// * `name` - 城市名称字符串
    ///
    /// # Returns
    /// * `Ok(Option<City>)` - 匹配的城市实体（如果有）
    /// * `Err(GeoServiceError)` - 查找失败及原因
    ///
    /// # Notes
    /// 我们假定不存在同名城市
    ///
    /// # Errors
    /// * `InfrastructureError` - 仓储访问错误
    async fn get_city_by_name(&self, name: &str) -> Result<Option<City>, GeoServiceError> {
        let cities = self.city_repository.find_by_name(name).await?;

        if cities.is_empty() {
            Ok(None)
        } else {
            Ok(Some(cities[0].clone()))
        }
    }

    /// 添加新城市实现
    ///
    /// # Arguments
    /// * `city` - 要添加的城市实体
    ///
    /// # Returns
    /// * `Ok(CityId)` - 新添加的城市ID
    /// * `Err(GeoServiceError)` - 添加失败及原因
    ///
    /// # Notes
    /// 添加前会检查城市名称是否已存在
    ///
    /// # Errors
    /// * `CityExists` - 城市已存在
    /// * `InfrastructureError` - 仓储访问错误
    async fn add_city(&self, city: City) -> Result<CityId, GeoServiceError> {
        if !self
            .city_repository
            .find_by_name(city.name())
            .await?
            .is_empty()
        {
            return Err(GeoServiceError::CityExists(city.name().to_string()));
        }

        let mut city = city;

        self.city_repository.save(&mut city).await?;

        Ok(city.get_id().expect("city should have an id after save"))
    }

    /// 删除城市实现
    ///
    /// # Arguments
    /// * `city` - 要删除的城市实体
    ///
    /// # Returns
    /// * `Ok(())` - 删除成功
    /// * `Err(GeoServiceError)` - 删除失败及原因
    ///
    /// # Errors
    /// * `InfrastructureError` - 仓储访问错误
    async fn remove_city(&self, city: City) -> Result<(), GeoServiceError> {
        self.city_repository.remove(city).await?;

        Ok(())
    }

    /// 修改城市信息实现
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
    /// # Notes
    /// 会先检查城市是否存在
    ///
    /// # Errors
    /// * `NoSuchCityId` - 指定ID的城市不存在
    /// * `InfrastructureError` - 仓储访问错误
    async fn modify_city(
        &self,
        city_id: CityId,
        city_name: CityName,
        province: ProvinceName,
    ) -> Result<(), GeoServiceError> {
        if let Some(city) = self.city_repository.find(city_id).await? {
            let mut city = City::new(city.get_id(), city_name, province);
            self.city_repository.save(&mut city).await?;

            Ok(())
        } else {
            Err(GeoServiceError::NoSuchCityId(city_id.into()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::mock::city::MockCityRepository;
    use crate::domain::service::geo::GeoService;
    use crate::domain::RepositoryError;
    use anyhow::anyhow;
    use mockall::predicate::*;
    use std::sync::Arc;

    fn sample_city(id: u64, name: String, province: String) -> City {
        City::new(
            Some(id.into()),
            CityName::from(name),
            ProvinceName::from(province),
        )
    }

    #[tokio::test]
    async fn test_get_city_map_success() {
        let mut repo = MockCityRepository::new();
        repo.expect_load().returning(|| {
            Ok(vec![
                sample_city(1, "Shanghai".to_string(), "Shanghai".to_string()),
                sample_city(2, "Hangzhou".to_string(), "Zhejiang".to_string()),
            ])
        });

        let service = GeoServiceImpl::new(Arc::new(repo));

        let result = service.get_city_map().await.unwrap();
        assert_eq!(result.len(), 2);
        assert!(result.contains_key(&ProvinceName::from("Shanghai".to_string())));
        assert!(result.contains_key(&ProvinceName::from("Zhejiang".to_string())));
    }

    #[tokio::test]
    async fn test_get_city_map_error() {
        let mut repo = MockCityRepository::new();
        repo.expect_load()
            .returning(|| Err(RepositoryError::Db(anyhow!("db error"))));

        let service = GeoServiceImpl::new(Arc::new(repo));

        let result = service.get_city_map().await;
        assert!(matches!(
            result,
            Err(GeoServiceError::InfrastructureError(_))
        ));
    }

    #[tokio::test]
    async fn test_get_city_by_name_found() {
        let mut repo = MockCityRepository::new();
        repo.expect_find_by_name()
            .with(eq("Shanghai"))
            .returning(|_| {
                Ok(vec![sample_city(
                    1u64,
                    "Shanghai".to_string(),
                    "Shanghai".to_string(),
                )])
            });

        let service = GeoServiceImpl::new(Arc::new(repo));

        let result = service.get_city_by_name("Shanghai").await.unwrap();
        assert!(result.is_some());
        assert_eq!(
            result.unwrap().name(),
            &CityName::from("Shanghai".to_string())
        );
    }

    #[tokio::test]
    async fn test_get_city_by_name_not_found() {
        let mut repo = MockCityRepository::new();
        repo.expect_find_by_name()
            .with(eq("Unknown"))
            .returning(|_| Ok(vec![]));

        let service = GeoServiceImpl::new(Arc::new(repo));

        let result = service.get_city_by_name("Unknown").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_add_city_success() {
        let mut repo = MockCityRepository::new();
        repo.expect_find_by_name()
            .with(eq("NewCity"))
            .returning(|_| Ok(vec![]));

        repo.expect_save().returning(|city: &mut City| {
            city.set_id(1u64.into());
            Ok(1u64.into())
        });

        let service = GeoServiceImpl::new(Arc::new(repo));

        let city = sample_city(0, "NewCity".to_string(), "SomeProvince".to_string());
        let id = service.add_city(city).await.unwrap();

        assert_eq!(id, 1u64.into());
    }

    #[tokio::test]
    async fn test_add_city_already_exists() {
        let mut repo = MockCityRepository::new();
        repo.expect_find_by_name()
            .with(eq("Shanghai"))
            .returning(|_| {
                Ok(vec![sample_city(
                    1,
                    "Shanghai".to_string(),
                    "Shanghai".to_string(),
                )])
            });

        let service = GeoServiceImpl::new(Arc::new(repo));

        let city = sample_city(0, "Shanghai".to_string(), "Shanghai".to_string());
        let result = service.add_city(city).await;

        assert!(matches!(result, Err(GeoServiceError::CityExists(_))));
    }

    #[tokio::test]
    async fn test_remove_city_success() {
        let mut repo = MockCityRepository::new();
        repo.expect_remove().returning(|_| Ok(()));

        let service = GeoServiceImpl::new(Arc::new(repo));

        let city = sample_city(1, "Shanghai".to_string(), "Shanghai".to_string());
        let result = service.remove_city(city).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_remove_city_error() {
        let mut repo = MockCityRepository::new();
        repo.expect_remove()
            .returning(|_| Err(RepositoryError::Db(anyhow!("db error"))));

        let service = GeoServiceImpl::new(Arc::new(repo));

        let city = sample_city(1, "Shanghai".to_string(), "Shanghai".to_string());
        let result = service.remove_city(city).await;

        assert!(matches!(
            result,
            Err(GeoServiceError::InfrastructureError(_))
        ));
    }

    #[tokio::test]
    async fn test_modify_city_success() {
        let city_id: CityId = 1u64.into();
        let mut repo = MockCityRepository::new();
        repo.expect_find().with(eq(city_id)).returning(|_| {
            Ok(Some(sample_city(
                1,
                "Old".to_string(),
                "OldProv".to_string(),
            )))
        });

        repo.expect_save().returning(|_| Ok(1u64.into()));

        let service = GeoServiceImpl::new(Arc::new(repo));

        let result = service
            .modify_city(
                1u64.into(),
                CityName::from("New".to_string()),
                ProvinceName::from("NewProv".to_string()),
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_modify_city_not_found() {
        let city_id: CityId = 999u64.into();
        let mut repo = MockCityRepository::new();
        repo.expect_find().with(eq(city_id)).returning(|_| Ok(None));

        let service = GeoServiceImpl::new(Arc::new(repo));

        let result = service
            .modify_city(
                999u64.into(),
                CityName::from("X".to_string()),
                ProvinceName::from("Y".to_string()),
            )
            .await;

        assert!(matches!(result, Err(GeoServiceError::NoSuchCityId(_))));
    }
}
