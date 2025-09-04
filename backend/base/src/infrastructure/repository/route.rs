//! 车次路线仓储实现模块
//!
//! 该模块提供了基于SeaORM的车次路线仓储实现，负责车次路线数据的持久化操作。
//! 路线作为火车票订购系统的核心数据，包含车次的所有停靠站点信息。
//!
//! # 主要功能
//! - 车次路线的CRUD操作
//! - 批量导入路线数据
//! - 按路线ID查询完整路线信息
//!
//! # 实现特点
//! - 使用事务保证操作的原子性
//! - 自动生成路线ID
//! - 严格的数据一致性检查
use crate::domain::model::route::{Route, RouteId, StopId};
use crate::domain::model::station::StationId;
use crate::domain::model::train_schedule::TrainScheduleId;
use crate::domain::repository::route::RouteRepository;
use crate::domain::{DbId, Identifiable, Repository, RepositoryError};
use anyhow::{anyhow, Context};
use async_trait::async_trait;
use sea_orm::{ActiveValue, DatabaseBackend, DatabaseConnection, Statement};
use sea_orm::{ColumnTrait, TransactionTrait};
use sea_orm::{DatabaseTransaction, QueryFilter};
use sea_orm::{EntityTrait, QueryOrder};
use shared::data::RouteStationInfo;
use std::collections::HashMap;
use tracing::{error, instrument, trace};

impl_db_id_from_u64!(RouteId, i64, "route");
impl_db_id_from_u64!(StopId, i32, "stop");

/// 车次路线数据转换器
///
/// 负责领域对象(`Route`)与数据库模型(`route::Model`)之间的双向转换。
///
/// # 转换逻辑
/// - 一个Route对应数据库中的多条记录(每个停靠站一条)
/// - 转换时自动处理ID类型转换
/// - 严格检查数据一致性
pub struct RouteDataConverter;

impl RouteDataConverter {
    /// 从数据库模型创建路线领域对象
    ///
    /// # Arguments
    /// - `route_model_list`: 数据库查询结果(同一路线的所有停靠站记录)
    ///
    /// # Returns
    /// 转换后的Route领域对象
    ///
    /// # Errors
    /// - 当停靠站列表为空时返回错误
    /// - 当路线ID不一致时返回错误
    /// - 当ID转换失败时返回错误
    #[instrument(skip_all)]
    pub fn make_from_do(
        route_model_list: Vec<crate::models::route::Model>,
    ) -> Result<Route, anyhow::Error> {
        if route_model_list.is_empty() {
            error!("Route model list is empty");
            return Err(anyhow::anyhow!("Route model list is empty"));
        }

        let route_id = route_model_list[0].line_id;

        for stop in &route_model_list {
            if stop.line_id != route_id {
                error!(
                    "Inconsistent: Route ID mismatch, current: {}, first: {}",
                    stop.line_id, route_id
                );
                return Err(anyhow::anyhow!("Inconsistent: Route ID mismatch"));
            }
        }

        let route_id = RouteId::from_db_value(route_id)?;

        let mut route = Route::new(Some(route_id));

        for stop in route_model_list {
            route.add_stop(
                Some(StopId::from_db_value(stop.id)?),
                StationId::from_db_value(stop.station_id)?,
                stop.arrival_time as u32,
                stop.departure_time as u32,
                stop.order as u32,
            );
        }

        Ok(route)
    }

    /// 将路线领域对象转换为数据库模型列表
    ///
    /// # Arguments
    /// - `route`: 要转换的Route领域对象
    ///
    /// # Returns
    /// 可用于数据库操作的ActiveModel列表(每个停靠站一个)
    #[instrument]
    pub fn transform_to_do(route: Route) -> Vec<crate::models::route::ActiveModel> {
        let route_id = route.get_id();

        let mut result = Vec::new();

        for stop in route.into_stops() {
            let stop_id = stop.get_id();
            let station_id = stop.station_id();
            let arrival_time = stop.arrival_time();
            let departure_time = stop.departure_time();
            let order = stop.order();

            // Convert to ActiveModel
            let mut model = crate::models::route::ActiveModel {
                id: ActiveValue::NotSet,
                line_id: ActiveValue::NotSet,
                station_id: ActiveValue::Set(station_id.to_db_value()),
                arrival_time: ActiveValue::Set(arrival_time as i32),
                departure_time: ActiveValue::Set(departure_time as i32),
                order: ActiveValue::Set(order as i32),
            };

            if let Some(stop_id) = stop_id {
                model.id = ActiveValue::Set(stop_id.to_db_value());
            }

            result.push(model);
        }

        if let Some(route_id) = route_id {
            for model in &mut result {
                model.line_id = ActiveValue::Set(route_id.to_db_value());
            }
        }

        result
    }
}

/// 车次路线仓储实现
///
/// 使用SeaORM作为底层ORM框架，提供路线数据的持久化能力。
///
/// # 特性
/// - 线程安全的数据库连接管理
/// - 自动事务处理
/// - 详细的日志记录
pub struct RouteRepositoryImpl {
    /// 数据库连接池
    db: DatabaseConnection,
}

impl RouteRepositoryImpl {
    /// 创建新的路线仓储实例
    ///
    /// # Arguments
    /// - `db`: SeaORM数据库连接
    pub fn new(db: DatabaseConnection) -> Self {
        RouteRepositoryImpl { db }
    }

    /// 获取新的路线ID
    ///
    /// # Arguments
    /// - `txn`: 数据库事务
    ///
    /// # Returns
    /// 新的路线ID(当前最大ID+1)
    ///
    /// # Errors
    /// - 数据库查询错误
    #[instrument(skip_all)]
    pub async fn get_new_line_id(&self, txn: &DatabaseTransaction) -> Result<i64, RepositoryError> {
        let last_line = crate::models::route::Entity::find()
            .order_by_desc(crate::models::route::Column::LineId)
            .one(txn)
            .await
            .context("failed to get last line id")
            .map_err(|e| {
                error!("Failed to get last line id: {}", e);
                e
            })?;

        Ok(last_line.map(|model| model.line_id + 1).unwrap_or(1))
    }
}

#[async_trait]
impl Repository<Route> for RouteRepositoryImpl {
    /// 根据ID查找路线
    ///
    /// # Arguments
    /// - `id`: 路线ID
    ///
    /// # Returns
    /// - `Some(Route)`: 找到的路线对象(包含所有停靠站)
    /// - `None`: 路线不存在
    ///
    /// # Errors
    /// - 数据库查询错误
    /// - 数据转换错误
    #[instrument(skip(self))]
    async fn find(&self, id: RouteId) -> Result<Option<Route>, RepositoryError> {
        let model = crate::models::route::Entity::find()
            .filter(crate::models::route::Column::LineId.eq(id.to_db_value()))
            .all(&self.db)
            .await
            .context(format!(
                "failed to find route with id: {}",
                id.to_db_value()
            ))
            .map_err(|e| {
                error!("Failed to find route with id: {}: {}", id.to_db_value(), e);
                e
            })?;

        if !model.is_empty() {
            let route = RouteDataConverter::make_from_do(model)
                .context(format!(
                    "failed to convert route with id: {}",
                    id.to_db_value()
                ))
                .map_err(|e| {
                    error!(
                        "Failed to convert route with id: {}: {}",
                        id.to_db_value(),
                        e
                    );
                    e
                })
                .map_err(RepositoryError::ValidationError)?;
            Ok(Some(route))
        } else {
            Ok(None)
        }
    }

    /// 删除路线记录
    ///
    /// # Arguments
    /// - `aggregate`: 要删除的路线对象
    ///
    /// # Notes
    /// - 使用事务保证删除所有相关停靠站记录
    ///
    /// # Errors
    /// - 数据库操作错误
    #[instrument(skip(self))]
    async fn remove(&self, aggregate: Route) -> Result<(), RepositoryError> {
        if let Some(route_id) = aggregate.get_id() {
            trace!("Begin Transaction");

            let txn = self
                .db
                .begin()
                .await
                .context("Failed to start transaction")
                .map_err(|e| {
                    error!("Failed to start transaction: {}", e);
                    e
                })?;

            crate::models::route::Entity::delete_many()
                .filter(crate::models::route::Column::LineId.eq(route_id.to_db_value()))
                .exec(&txn)
                .await
                .context(format!(
                    "failed to delete route with id: {}",
                    route_id.to_db_value()
                ))
                .map_err(|e| {
                    error!(
                        "Failed to delete route with id: {}: {}",
                        route_id.to_db_value(),
                        e
                    );
                    e
                })?;

            trace!("Commit Transaction");
            txn.commit().await.context("Failed to commit transaction")?;
        }

        Ok(())
    }

    /// 保存路线记录
    ///
    /// # Arguments
    /// - `aggregate`: 要保存的路线对象(可变引用)
    ///
    /// # Returns
    /// 保存后的路线ID
    ///
    /// # Notes
    /// - 使用事务保证原子性
    /// - 自动生成新的路线ID(如果是新增)
    /// - 保存后会重新加载完整路线数据
    ///
    /// # Errors
    /// - 数据库操作错误
    #[instrument(skip(self))]
    async fn save(&self, aggregate: &mut Route) -> Result<RouteId, RepositoryError> {
        trace!("Begin Transaction");
        let txn = self
            .db
            .begin()
            .await
            .context("Failed to start transaction")
            .map_err(|e| {
                error!("Failed to start transaction: {}", e);
                e
            })?;

        let models = RouteDataConverter::transform_to_do(aggregate.clone());

        let last_line_id = self.get_new_line_id(&txn).await?;

        crate::models::route::Entity::insert_many(models)
            .exec(&txn)
            .await
            .context(format!("failed to insert route with id: {}", last_line_id))
            .map_err(|e| {
                error!("Failed to insert route with id: {}: {}", last_line_id, e);
                e
            })?;

        trace!("Commit Transaction");
        txn.commit().await.context("Failed to commit transaction")?;

        *aggregate = self
            .find(RouteId::from_db_value(last_line_id)?)
            .await?
            .expect("should found already inserted route");

        Ok(aggregate
            .get_id()
            .expect("already inserted route should have id"))
    }
}

#[async_trait]
impl RouteRepository for RouteRepositoryImpl {
    /// 加载所有路线
    ///
    /// # Returns
    /// 所有路线列表
    ///
    /// # Errors
    /// - 数据库查询错误
    /// - 数据转换错误
    #[instrument(skip_all)]
    async fn load(&self) -> Result<Vec<Route>, RepositoryError> {
        let route_models = crate::models::route::Entity::find()
            .all(&self.db)
            .await
            .context("failed to load routes")
            .map_err(|e| {
                error!("Failed to load routes: {}", e);
                e
            })?;

        let mut route_id_to_stops: HashMap<i64, Vec<crate::models::route::Model>> = HashMap::new();

        for model in route_models {
            route_id_to_stops
                .entry(model.line_id)
                .or_default()
                .push(model);
        }

        let mut result = Vec::with_capacity(route_id_to_stops.len());

        for v in route_id_to_stops.into_values() {
            result.push(
                RouteDataConverter::make_from_do(v)
                    .map_err(RepositoryError::ValidationError)
                    .map_err(|e| {
                        error!("Failed to convert route: {}", e);
                        e
                    })?,
            );
        }

        Ok(result)
    }

    #[instrument(skip(self))]
    async fn get_by_train_schedule(
        &self,
        train_schedule_id: TrainScheduleId,
    ) -> Result<Option<Route>, RepositoryError> {
        let models = crate::models::route::Entity::find()
            .from_raw_sql(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                r#"SELECT
    "route"."id",
    "route"."line_id",
    "route"."station_id",
    "route"."arrival_time",
    "route"."departure_time",
    "route"."order"
FROM "route"
    INNER JOIN "train_schedule"
        ON "train_schedule"."line_id" = "route"."line_id"
WHERE "train_schedule"."id" = $1;"#,
                [train_schedule_id.to_db_value().into()],
            ))
            .all(&self.db)
            .await
            .context(format!(
                "failed to get route by train schedule id: {}",
                train_schedule_id.to_db_value()
            ))
            .map_err(|e| {
                error!(
                    "Failed to get route by train schedule id: {}: {}",
                    train_schedule_id.to_db_value(),
                    e
                );
                e
            })?;

        if !models.is_empty() {
            let route = RouteDataConverter::make_from_do(models)
                .context(format!(
                    "failed to convert route with train schedule id: {}",
                    train_schedule_id.to_db_value()
                ))
                .map_err(|e| {
                    error!(
                        "Failed to convert route with train schedule id: {}: {}",
                        train_schedule_id.to_db_value(),
                        e
                    );
                    e
                })
                .map_err(RepositoryError::ValidationError)?;
            Ok(Some(route))
        } else {
            Ok(None)
        }
    }

    /// 批量导入原始路线数据
    ///
    /// # Arguments
    /// - `raw_routes`: 原始路线数据(车站名称和时间信息)
    ///
    /// # Returns
    /// 新创建的路线ID
    ///
    /// # Notes
    /// - 使用事务保证原子性
    /// - 自动将车站名称转换为ID
    /// - 自动生成新的路线ID
    ///
    /// # Errors
    /// - 数据库操作错误
    /// - 车站不存在错误
    #[instrument(skip_all)]
    async fn save_raw(
        &self,
        raw_routes: Vec<RouteStationInfo>,
    ) -> Result<RouteId, RepositoryError> {
        trace!("Begin Transaction");
        let txn = self
            .db
            .begin()
            .await
            .context("Failed to start transaction")
            .map_err(|e| {
                error!("Failed to start transaction: {}", e);
                e
            })?;

        let new_line_id = self.get_new_line_id(&txn).await?;

        let stations = crate::models::station::Entity::find()
            .all(&txn)
            .await
            .context("Failed to load stations")
            .map_err(|e| {
                error!("Failed to load stations: {}", e);
                e
            })?;

        let station_name_to_id = stations
            .into_iter()
            .map(|s| (s.name, s.id))
            .collect::<HashMap<_, _>>();

        let mut model_list = Vec::new();

        for stop in raw_routes {
            let station_id = station_name_to_id
                .get(&stop.station)
                .copied()
                .ok_or(RepositoryError::InconsistentState(anyhow!(
                    "Station not found: {}",
                    stop.station
                )))
                .map_err(|e| {
                    error!("Failed to find station: {}: {}", stop.station, e);
                    e
                })?;

            let model = crate::models::route::ActiveModel {
                id: ActiveValue::NotSet,
                line_id: ActiveValue::Set(new_line_id),
                station_id: ActiveValue::Set(station_id),
                arrival_time: ActiveValue::Set(stop.arrival_time as i32),
                departure_time: ActiveValue::Set(stop.departure_time as i32),
                order: ActiveValue::Set(stop.order as i32),
            };

            model_list.push(model);
        }

        crate::models::route::Entity::insert_many(model_list)
            .exec(&txn)
            .await
            .context("Failed to insert routes")
            .map_err(|e| {
                error!("Failed to insert routes: {}", e);
                e
            })?;

        trace!("Commit Transaction");
        txn.commit().await.context("Failed to commit transaction")?;

        Ok(RouteId::from_db_value(new_line_id)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::route::{Route, StopId};
    use crate::domain::model::station::StationId;
    use crate::domain::model::train_schedule::TrainScheduleId;
    use crate::domain::repository::route::RouteRepository;
    use chrono::NaiveDate;
    use sea_orm::{
        ActiveModelTrait, ActiveValue, ConnectionTrait, Database, DatabaseBackend,
        DatabaseConnection, Statement,
    };

    // 内存数据库初始化函数
    async fn setup_repo() -> RouteRepositoryImpl {
        let db: DatabaseConnection = Database::connect("sqlite::memory:").await.unwrap();

        // 创建 station 表（新增 city_id）
        db.execute(Statement::from_string(
            DatabaseBackend::Sqlite,
            r#"
            CREATE TABLE station (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                city_id INTEGER DEFAULT 0
            );
            "#
            .to_string(),
        ))
        .await
        .unwrap();

        // 创建 route 表
        db.execute(Statement::from_string(
            DatabaseBackend::Sqlite,
            r#"
            CREATE TABLE route (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                line_id INTEGER NOT NULL,
                station_id INTEGER NOT NULL,
                arrival_time INTEGER NOT NULL,
                departure_time INTEGER NOT NULL,
                "order" INTEGER NOT NULL
            );
            "#
            .to_string(),
        ))
        .await
        .unwrap();

        // 创建 train_schedule 表
        db.execute(Statement::from_string(
            DatabaseBackend::Sqlite,
            r#"
    CREATE TABLE train_schedule (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        train_id INTEGER NOT NULL,
        departure_date INTEGER NOT NULL,
        origin_departure_time INTEGER NOT NULL,
        line_id INTEGER NOT NULL
    );
    "#
            .to_string(),
        ))
        .await
        .unwrap();

        // 插入测试站点
        let stations = vec!["A", "B", "C"];
        for name in stations {
            crate::models::station::ActiveModel {
                id: ActiveValue::NotSet,
                name: ActiveValue::Set(name.to_string()),
                city_id: ActiveValue::Set(1), // 给 city_id 一个默认值
            }
            .insert(&db)
            .await
            .unwrap();
        }

        RouteRepositoryImpl::new(db)
    }

    #[tokio::test]
    async fn test_save_and_find_route() {
        let repo = setup_repo().await;

        let mut route = Route::new(Some(1u64.into()));
        let stations = vec![1, 2, 3];
        for (i, s) in stations.iter().enumerate() {
            route.add_stop(
                Some(StopId::from_db_value(i as i32 + 1).unwrap()),
                StationId::from_db_value(*s).unwrap(),
                100 + i as u32,
                110 + i as u32,
                i as u32,
            );
        }

        let route_id = repo.save(&mut route).await.unwrap();
        let retrieved = repo.find(route_id).await.unwrap().unwrap();

        assert_eq!(retrieved.get_id(), Some(route_id));
        assert_eq!(retrieved.stops().len(), 3);
    }

    #[tokio::test]
    async fn test_remove_route() {
        let repo = setup_repo().await;

        let mut route = Route::new(Some(1u64.into()));
        route.add_stop(
            Some(StopId::from_db_value(1).unwrap()),
            StationId::from_db_value(1).unwrap(),
            100,
            110,
            0,
        );

        let route_id = repo.save(&mut route).await.unwrap();
        repo.remove(route).await.unwrap();

        let result = repo.find(route_id).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_load_routes() {
        let repo = setup_repo().await;

        let mut route = Route::new(Some(1u64.into()));
        for s in 1..=3 {
            route.add_stop(
                Some(StopId::from_db_value(s).unwrap()),
                StationId::from_db_value(s).unwrap(),
                100 + s as u32,
                110 + s as u32,
                s as u32,
            );
        }
        repo.save(&mut route).await.unwrap();

        let routes = repo.load().await.unwrap();
        assert_eq!(routes.len(), 1);
    }

    #[tokio::test]
    async fn test_get_by_train_schedule() {
        let repo = setup_repo().await;

        let mut route = Route::new(Some(1u64.into()));
        for s in 1..=3 {
            route.add_stop(
                Some(StopId::from_db_value(s).unwrap()),
                StationId::from_db_value(s).unwrap(),
                100 + s as u32,
                110 + s as u32,
                s as u32,
            );
        }
        let route_id = repo.save(&mut route).await.unwrap();

        crate::models::train_schedule::ActiveModel {
            id: ActiveValue::NotSet,
            train_id: ActiveValue::Set(1), // 默认值
            departure_date: ActiveValue::Set(NaiveDate::from_ymd_opt(2022, 1, 1).unwrap()),
            origin_departure_time: ActiveValue::Set(0),
            line_id: ActiveValue::Set(route_id.to_db_value()),
        }
        .insert(&repo.db)
        .await
        .unwrap();

        let route_by_schedule = repo
            .get_by_train_schedule(TrainScheduleId::from_db_value(1).unwrap())
            .await
            .unwrap();

        assert!(route_by_schedule.is_some());
        assert_eq!(route_by_schedule.unwrap().get_id(), Some(route_id));
    }

    #[tokio::test]
    async fn test_save_raw_routes() {
        let repo = setup_repo().await;

        let raw_routes = vec![
            RouteStationInfo {
                station: "A".to_string(),
                arrival_time: 100,
                departure_time: 110,
                order: 0,
            },
            RouteStationInfo {
                station: "B".to_string(),
                arrival_time: 120,
                departure_time: 130,
                order: 1,
            },
        ];

        let route_id = repo.save_raw(raw_routes).await.unwrap();
        let retrieved = repo.find(route_id).await.unwrap().unwrap();
        assert_eq!(retrieved.stops().len(), 2);
    }
}
