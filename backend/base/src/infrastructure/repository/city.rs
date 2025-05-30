//! 城市仓储实现模块
//!
//! 提供基于SeaORM的城市仓储实现，负责城市数据的持久化操作。
//!
//! 主要功能：
//! - 城市实体的CRUD操作
//! - 按名称或省份查询城市
//! - 批量导入城市数据
//!
//! # 实现细节
//! - 使用`CityRepositoryImpl`作为主要实现结构体
//! - 通过`CityDataConverter`处理领域模型与数据库模型的转换
//! - 基于SeaORM实现数据库操作
//!
//! # 数据转换
//! - 领域对象(`City`)与数据库模型(`city::Model`)之间的双向转换
//! - 自动处理ID类型转换
//! - 字段验证和错误处理

use crate::domain::model::city::{City, CityId, ProvinceName};
use crate::domain::repository::city::CityRepository;
use crate::domain::{DbId, Identifiable, Repository, RepositoryError};
use crate::infrastructure::repository::transform_list;
use anyhow::Context;
use async_trait::async_trait;
use sea_orm::sea_query::OnConflict;
use sea_orm::{ActiveValue, DatabaseConnection, EntityTrait, QueryFilter, TransactionTrait};
use sea_orm::{ColumnTrait, Select};
use shared::data::CityData;
use tracing::{debug, error, instrument, trace};

/// 城市仓储实现
///
/// 使用SeaORM作为底层ORM框架，提供城市数据的持久化能力。
///
/// # 特性
/// - 线程安全的数据库连接管理
/// - 自动事务处理
/// - 详细的日志记录
///
pub struct CityRepositoryImpl {
    /// 数据库连接池
    db: DatabaseConnection,
}

/// 城市数据转换器
///
/// 负责领域对象(`City`)与数据库模型(`city::Model`)之间的转换。
///
/// # 方法
/// - `make_from_do`: 从数据库模型创建领域对象
/// - `transform_to_do`: 将领域对象转换为数据库模型
///
/// # Errors
/// - 当ID转换失败时返回错误
/// - 当字段验证失败时返回错误
pub struct CityDataConverter;

impl_db_id_from_u64!(CityId, i32, "city");

impl CityDataConverter {
    /// 从数据库模型创建城市领域对象
    ///
    /// # Arguments
    /// - `city_do`: 数据库查询结果模型
    ///
    /// # Returns
    /// 转换后的城市领域对象
    ///
    /// # Errors
    /// - 当ID转换失败时返回错误
    #[instrument]
    pub fn make_from_do(city_do: crate::models::city::Model) -> anyhow::Result<City> {
        let city_id = CityId::from_db_value(city_do.id)?;
        let name = city_do.name.into();
        let province = city_do.province.into();

        Ok(City::new(Some(city_id), name, province))
    }

    /// 将城市领域对象转换为数据库模型
    ///
    /// # Arguments
    /// - `city`: 要转换的城市领域对象
    ///
    /// # Returns
    /// 可用于数据库操作的ActiveModel
    #[instrument]
    pub fn transform_to_do(city: City) -> crate::models::city::ActiveModel {
        let mut model = crate::models::city::ActiveModel {
            id: ActiveValue::NotSet,
            name: ActiveValue::Set(city.name().to_string()),
            province: ActiveValue::Set(city.province().to_string()),
        };

        if let Some(id) = city.get_id() {
            model.id = ActiveValue::Set(id.to_db_value());
        }

        model
    }
}

#[async_trait]
impl Repository<City> for CityRepositoryImpl {
    /// 根据ID查找城市
    ///
    /// # Arguments
    /// - `id`: 城市ID
    ///
    /// # Returns
    /// - `Some(City)`: 找到的城市对象
    /// - `None`: 城市不存在
    ///
    /// # Errors
    /// - 数据库查询错误
    /// - 数据转换错误
    #[instrument(skip(self))]
    async fn find(&self, id: CityId) -> Result<Option<City>, RepositoryError> {
        let result = crate::models::city::Entity::find_by_id(id.to_db_value())
            .one(&self.db)
            .await
            .context(format!("Failed to find city with id: {}", id.to_db_value()))
            .map_err(|e| {
                error!("Failed to find city with id: {}: {:?}", id.to_db_value(), e);
                RepositoryError::Db(e)
            })?;

        result
            .map(CityDataConverter::make_from_do)
            .transpose()
            .context(format!(
                "Failed to validate city with id: {}",
                id.to_db_value()
            ))
            .map_err(RepositoryError::ValidationError)
    }

    /// 删除城市记录
    ///
    /// # Arguments
    /// - `aggregate`: 要删除的城市对象
    ///
    /// # Errors
    /// - 数据库操作错误
    #[instrument(skip(self))]
    async fn remove(&self, aggregate: City) -> Result<(), RepositoryError> {
        if let Some(id) = aggregate.get_id() {
            let id = id.to_db_value();

            crate::models::city::Entity::delete_by_id(id)
                .exec(&self.db)
                .await
                .context(format!("Failed to remove city with id: {}", id))
                .map_err(|e| {
                    error!("Failed to remove city with id: {}: {:?}", id, e);
                    RepositoryError::Db(e)
                })?;
        }

        Ok(())
    }

    /// 保存城市记录
    ///
    /// # Arguments
    /// - `aggregate`: 要保存的城市对象（可变引用）
    ///
    /// # Returns
    /// 保存后的城市ID
    ///
    /// # Notes
    /// - 如果城市ID已存在，则执行更新操作
    /// - 如果城市ID不存在，则执行插入操作并设置新ID
    ///
    /// # Errors
    /// - 数据库操作错误
    #[instrument(skip(self))]
    async fn save(&self, aggregate: &mut City) -> Result<CityId, RepositoryError> {
        let city_do = CityDataConverter::transform_to_do(aggregate.clone());
        let result = crate::models::city::Entity::insert(city_do)
            .on_conflict(
                OnConflict::column(crate::models::city::Column::Id)
                    .update_columns([
                        crate::models::city::Column::Name,
                        crate::models::city::Column::Province,
                    ])
                    .to_owned(),
            )
            .exec(&self.db)
            .await
            .context(format!(
                "Failed to save city with id: {:?}",
                aggregate.get_id()
            ))
            .map_err(|e| {
                error!(
                    "Failed to save city with id: {:?}: {:?}",
                    aggregate.get_id(),
                    e
                );
                RepositoryError::Db(e)
            })?;

        let id = result.last_insert_id as u64;
        debug!("City saved with id: {}", id);
        aggregate.set_id(id.into());

        Ok(id.into())
    }
}

#[async_trait]
impl CityRepository for CityRepositoryImpl {
    /// 加载所有城市记录
    ///
    /// # Returns
    /// 城市列表
    ///
    /// # Errors
    /// - 数据库查询错误
    /// - 数据转换错误
    #[instrument(skip_all)]
    async fn load(&self) -> Result<Vec<City>, RepositoryError> {
        self.query_cities(|f| f).await
    }

    /// 按城市名称查询
    ///
    /// # Arguments
    /// - `city_name`: 城市名称
    ///
    /// # Returns
    /// 匹配的城市列表
    ///
    /// # Errors
    /// - 数据库查询错误
    /// - 数据转换错误
    #[instrument(skip(self))]
    async fn find_by_name(&self, city_name: &str) -> Result<Vec<City>, RepositoryError> {
        self.query_cities(|f| f.filter(crate::models::city::Column::Name.eq(city_name)))
            .await
    }

    /// 按省份名称查询
    ///
    /// # Arguments
    /// - `province_name`: 省份名称
    ///
    /// # Returns
    /// 匹配的城市列表
    ///
    /// # Errors
    /// - 数据库查询错误
    /// - 数据转换错误
    #[instrument(skip(self))]
    async fn find_by_province(
        &self,
        province_name: ProvinceName,
    ) -> Result<Vec<City>, RepositoryError> {
        self.query_cities(|f| {
            f.filter(crate::models::city::Column::Province.eq(province_name.to_string()))
        })
        .await
    }

    /// 批量导入原始城市数据
    ///
    /// # Arguments
    /// - `city_data`: 城市数据列表（城市名称和省份名称对）
    ///
    /// # Notes
    /// - 使用事务保证原子性
    /// - 冲突时更新省份名称
    ///
    /// # Errors
    /// - 数据库操作错误
    #[instrument(skip_all)]
    async fn save_raw(&self, city_data: CityData) -> Result<(), RepositoryError> {
        let model_list = city_data
            .into_iter()
            .map(|(city, province)| crate::models::city::ActiveModel {
                id: ActiveValue::NotSet,
                name: ActiveValue::Set(city),
                province: ActiveValue::Set(province),
            })
            .collect::<Vec<_>>();

        trace!("Begin transaction");
        let txn = self
            .db
            .begin()
            .await
            .context("failed to start transaction")
            .map_err(|e| {
                error!("Failed to start transaction: {:?}", e);
                RepositoryError::Db(e)
            })?;

        crate::models::city::Entity::insert_many(model_list)
            .on_conflict(
                OnConflict::column(crate::models::city::Column::Name)
                    .update_column(crate::models::city::Column::Province)
                    .to_owned(),
            )
            .exec(&txn)
            .await
            .context("failed to save raw city data")
            .map_err(|e| {
                error!("Failed to save raw city data: {:?}", e);
                RepositoryError::Db(e)
            })?;

        trace!("Commit transaction");
        txn.commit()
            .await
            .context("failed to commit transaction")
            .map_err(|e| {
                error!("Failed to commit transaction: {:?}", e);
                RepositoryError::Db(e)
            })?;

        Ok(())
    }
}

impl CityRepositoryImpl {
    /// 创建新的城市仓储实例
    ///
    /// # Arguments
    /// - `db`: SeaORM数据库连接
    pub fn new(db: DatabaseConnection) -> Self {
        CityRepositoryImpl { db }
    }

    /// 通用城市查询方法
    ///
    /// # Arguments
    /// - `builder`: 查询构建器闭包
    ///
    /// # Returns
    /// 查询结果的城市列表
    ///
    /// # Errors
    /// - 数据库查询错误
    /// - 数据转换错误
    #[instrument(skip_all)]
    pub async fn query_cities(
        &self,
        builder: impl FnOnce(Select<crate::models::city::Entity>) -> Select<crate::models::city::Entity>,
    ) -> Result<Vec<City>, RepositoryError> {
        let query = builder(crate::models::city::Entity::find());
        let stations = query.all(&self.db).await.map_err(|e| {
            error!("Failed to query cities: {:?}", e);
            RepositoryError::Db(e.into())
        })?;
        transform_list(stations, CityDataConverter::make_from_do, |x| x.id)
            .context("Failed to transform city list")
            .map_err(|e| {
                error!("Failed to transform city list: {:?}", e);
                RepositoryError::ValidationError(e)
            })
    }
}
