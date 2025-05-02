//! 车站仓储实现模块
//!
//! 提供基于SeaORM的车站仓储实现，负责车站数据的持久化操作。
//! 车站作为火车票系统的核心数据，包含名称和所属城市信息。
//!
//! # 主要功能
//! - 车站实体的CRUD操作
//! - 按城市或名称查询车站
//! - 批量导入车站数据
//!
//! # 实现特点
//! - 使用SeaORM实现数据库操作
//! - 事务支持保证数据一致性
//! - 详细的错误处理和日志记录
use crate::domain::DbId;
use crate::domain::model::city::CityId;
use crate::domain::model::station::{Station, StationId};
use crate::domain::repository::station::StationRepository;
use crate::domain::{Identifiable, Repository, RepositoryError};
use crate::infrastructure::repository::transform_list;
use anyhow::{Context, anyhow};
use async_trait::async_trait;
use sea_orm::sea_query::OnConflict;
use sea_orm::{ActiveValue, DatabaseConnection, EntityTrait, QueryFilter, TransactionTrait};
use sea_orm::{ColumnTrait, Select};
use shared::data::StationData;
use std::collections::HashMap;
use tracing::{debug, error, instrument, trace};

/// 车站仓储实现
///
/// 使用SeaORM作为底层ORM框架，提供车站数据的持久化能力。
///
/// # 特性
/// - 线程安全的数据库连接管理
/// - 自动ID生成
/// - 详细的日志记录
pub struct StationRepositoryImpl {
    /// 数据库连接池
    db: DatabaseConnection,
}

impl_db_id_from_u64!(StationId, i32, "station");

/// 车站数据转换器
///
/// 负责领域对象(`Station`)与数据库模型(`station::Model`)之间的转换。
///
/// # 转换逻辑
/// - 处理ID类型转换
/// - 字段验证和错误处理
pub struct StationDataConverter;

impl StationDataConverter {
    /// 从数据库模型创建车站领域对象
    ///
    /// # Arguments
    /// - `station_do`: 数据库查询结果模型
    ///
    /// # Returns
    /// 转换后的Station领域对象
    ///
    /// # Errors
    /// - 当ID转换失败时返回错误
    #[instrument]
    pub fn make_from_do(station_do: crate::models::station::Model) -> anyhow::Result<Station> {
        let station_id = StationId::from_db_value(station_do.id)?;
        let name = station_do.name;
        let city_id = CityId::from_db_value(station_do.city_id)?;

        Ok(Station::new(Some(station_id), name, city_id))
    }

    /// 将车站领域对象转换为数据库模型
    ///
    /// # Arguments
    /// - `station`: 要转换的车站领域对象
    ///
    /// # Returns
    /// 可用于数据库操作的ActiveModel
    #[instrument]
    pub fn transform_to_do(station: &Station) -> crate::models::station::ActiveModel {
        let mut model = crate::models::station::ActiveModel {
            id: ActiveValue::NotSet,
            name: ActiveValue::Set(station.name().to_string()),
            city_id: ActiveValue::Set(station.city_id().to_db_value()),
        };

        if let Some(id) = station.get_id() {
            model.id = ActiveValue::Set(id.to_db_value());
        }

        model
    }
}

#[async_trait]
impl Repository<Station> for StationRepositoryImpl {
    /// 根据ID查找车站
    ///
    /// # Arguments
    /// - `id`: 车站ID
    ///
    /// # Returns
    /// - `Some(Station)`: 找到的车站对象
    /// - `None`: 车站不存在
    ///
    /// # Errors
    /// - 数据库查询错误
    /// - 数据转换错误
    #[instrument(skip(self))]
    async fn find(&self, id: StationId) -> Result<Option<Station>, RepositoryError> {
        let result = crate::models::station::Entity::find_by_id(u64::from(id) as i32)
            .one(&self.db)
            .await
            .context(format!("Failed to find station with id: {}", u64::from(id)))
            .map_err(|e| {
                error!("Failed to find station with id: {}: {:?}", u64::from(id), e);
                RepositoryError::Db(e)
            })?;

        result
            .map(StationDataConverter::make_from_do)
            .transpose()
            .context(format!(
                "Failed to validate station with id: {}",
                u64::from(id)
            ))
            .map_err(|e| {
                error!(
                    "Failed to validate station with id: {}: {:?}",
                    u64::from(id),
                    e
                );
                RepositoryError::ValidationError(e)
            })
    }

    /// 删除车站记录
    ///
    /// # Arguments
    /// - `aggregate`: 要删除的车站对象
    ///
    /// # Errors
    /// - 数据库操作错误
    #[instrument(skip(self))]
    async fn remove(&self, aggregate: Station) -> Result<(), RepositoryError> {
        if let Some(id) = aggregate.get_id() {
            let id = u64::from(id) as i32;

            crate::models::station::Entity::delete_by_id(id)
                .exec(&self.db)
                .await
                .context(format!("Failed to remove station with id: {}", id))
                .map_err(|e| {
                    error!("Failed to remove station with id: {}: {:?}", id, e);
                    RepositoryError::Db(e)
                })?;
        }

        Ok(())
    }

    /// 保存车站记录
    ///
    /// # Arguments
    /// - `aggregate`: 要保存的车站对象(可变引用)
    ///
    /// # Returns
    /// 保存后的车站ID
    ///
    /// # Notes
    /// - 如果车站ID已存在，则执行更新操作
    /// - 如果车站ID不存在，则执行插入操作并设置新ID
    ///
    /// # Errors
    /// - 数据库操作错误
    #[instrument(skip(self))]
    async fn save(&self, aggregate: &mut Station) -> Result<StationId, RepositoryError> {
        let station_do = StationDataConverter::transform_to_do(aggregate);
        let result = crate::models::station::Entity::insert(station_do)
            .on_conflict(
                OnConflict::column(crate::models::station::Column::Id)
                    .update_columns([
                        crate::models::station::Column::Name,
                        crate::models::station::Column::CityId,
                    ])
                    .to_owned(),
            )
            .exec(&self.db)
            .await
            .context(format!(
                "Failed to save station with id: {:?}",
                aggregate.get_id()
            ))
            .map_err(|e| {
                error!(
                    "Failed to save station with id: {:?}: {:?}",
                    aggregate.get_id(),
                    e
                );
                RepositoryError::Db(e)
            })?;

        let id = result.last_insert_id as u64;

        debug!("Station saved with id: {}", id);

        aggregate.set_id(id.into());

        Ok(id.into())
    }
}

#[async_trait]
impl StationRepository for StationRepositoryImpl {
    /// 加载所有车站记录
    ///
    /// # Returns
    /// 车站列表
    ///
    /// # Errors
    /// - 数据库查询错误
    /// - 数据转换错误
    #[instrument(skip(self))]
    async fn load(&self) -> Result<Vec<Station>, RepositoryError> {
        self.query_stations(|q| q).await
    }

    /// 按城市ID查询车站
    ///
    /// # Arguments
    /// - `city_id`: 城市ID
    ///
    /// # Returns
    /// 该城市下的车站列表
    ///
    /// # Errors
    /// - 数据库查询错误
    /// - 数据转换错误
    #[instrument(skip(self))]
    async fn find_by_city(&self, city_id: CityId) -> Result<Vec<Station>, RepositoryError> {
        self.query_stations(|q| {
            q.filter(crate::models::station::Column::CityId.eq(u64::from(city_id) as i32))
        })
        .await
    }

    /// 按车站名称查询
    ///
    /// # Arguments
    /// - `station_name`: 车站名称
    ///
    /// # Returns
    /// - `Some(Station)`: 找到的车站对象
    /// - `None`: 车站不存在
    ///
    /// # Errors
    /// - 数据库查询错误
    /// - 数据转换错误
    #[instrument(skip(self))]
    async fn find_by_name(&self, station_name: &str) -> Result<Option<Station>, RepositoryError> {
        let model = crate::models::station::Entity::find()
            .filter(crate::models::station::Column::Name.eq(station_name))
            .one(&self.db)
            .await
            .map_err(|e| RepositoryError::Db(e.into()))
            .map_err(|e| {
                error!("Failed to find station by name: {}: {:?}", station_name, e);
                e
            })?;

        let id = model.as_ref().map(|x| x.id);

        model
            .map(StationDataConverter::make_from_do)
            .transpose()
            .context(format!("Failed to validate station with id: {:?}", id))
            .map_err(RepositoryError::ValidationError)
            .map_err(|e| {
                error!("Failed to validate station with id: {:?}: {:?}", id, e);
                e
            })
    }

    /// 批量导入原始车站数据
    ///
    /// # Arguments
    /// - `station_data`: 车站数据列表(包含车站名称和所属城市名称)
    ///
    /// # Notes
    /// - 使用事务保证原子性
    /// - 自动将城市名称转换为ID
    ///
    /// # Errors
    /// - 数据库操作错误
    /// - 城市不存在错误
    async fn save_raw(&self, station_data: StationData) -> Result<(), RepositoryError> {
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

        let cities = crate::models::city::Entity::find()
            .all(&txn)
            .await
            .context("failed to load cities")
            .map_err(|e| {
                error!("Failed to load cities: {:?}", e);
                RepositoryError::Db(e)
            })?;

        let city_name_to_id = cities
            .into_iter()
            .map(|city| (city.name, city.id))
            .collect::<HashMap<_, _>>();

        for station in &station_data {
            if !city_name_to_id.contains_key(&station.city) {
                error!(
                    "City {} not found for station {}",
                    station.city, station.name
                );

                return Err(RepositoryError::InconsistentState(anyhow!(
                    "City not found: {}",
                    station.name
                )));
            }
        }

        let model_list = station_data
            .into_iter()
            .map(|station| {
                let city_id = city_name_to_id[&station.city];
                crate::models::station::ActiveModel {
                    id: ActiveValue::NotSet,
                    name: ActiveValue::Set(station.name),
                    city_id: ActiveValue::Set(city_id),
                }
            })
            .collect::<Vec<_>>();

        crate::models::station::Entity::insert_many(model_list)
            .on_conflict(
                OnConflict::columns([
                    crate::models::station::Column::Name,
                    crate::models::station::Column::CityId,
                ])
                .update_column(crate::models::station::Column::CityId)
                .to_owned(),
            )
            .exec(&txn)
            .await
            .context("failed to save raw station data")
            .map_err(|e| {
                error!("Failed to save raw station data: {:?}", e);
                RepositoryError::Db(e)
            })?;

        trace!("Commit transaction");
        txn.commit().await.context("failed to commit transaction")?;

        Ok(())
    }
}

impl StationRepositoryImpl {
    /// 创建新的车站仓储实例
    ///
    /// # Arguments
    /// - `db`: SeaORM数据库连接
    pub fn new(db: DatabaseConnection) -> Self {
        StationRepositoryImpl { db }
    }

    /// 通用车站查询方法
    ///
    /// # Arguments
    /// - `builder`: 查询构建器闭包
    ///
    /// # Returns
    /// 查询结果的车站列表
    ///
    /// # Errors
    /// - 数据库查询错误
    /// - 数据转换错误
    #[instrument(skip_all)]
    pub async fn query_stations(
        &self,
        builder: impl FnOnce(
            Select<crate::models::station::Entity>,
        ) -> Select<crate::models::station::Entity>,
    ) -> Result<Vec<Station>, RepositoryError> {
        let query = builder(crate::models::station::Entity::find());
        let stations = query
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::Db(e.into()))
            .map_err(|e| {
                error!("Failed to query stations: {:?}", e);
                e
            })?;

        transform_list(stations, StationDataConverter::make_from_do, |x| x.id)
            .context("Failed to transform station list")
            .map_err(|e| {
                error!("Failed to transform station list: {:?}", e);
                e
            })
            .map_err(RepositoryError::ValidationError)
    }
}
