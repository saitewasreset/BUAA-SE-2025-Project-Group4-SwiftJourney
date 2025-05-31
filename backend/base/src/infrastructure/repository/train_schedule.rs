//! 列车班次仓储实现模块
//!
//! 提供基于SeaORM的列车班次(聚合根)仓储实现，负责：
//! - 列车班次数据的持久化存储
//! - 座位占用状态管理
//! - 复杂查询支持（按日期/车次查询）
//!
//! # 实现特性
//! - 使用SeaORM进行数据库访问
//! - 支持事务性操作
//!
//! # 数据模型转换
//! 本模块实现了领域模型与数据库模型的互相转换：
//! - `TrainScheduleDataConverter`: 处理班次的转换
//!
//! # 并发安全
//! - 使用`Arc<Mutex<AggregateManager>>`保证变更检测的线程安全
//! - 所有数据库操作在事务中执行
use crate::domain::model::route::RouteId;
use crate::domain::model::station::StationId;
use crate::domain::model::train::{SeatType, SeatTypeId, SeatTypeName, TrainId};
use crate::domain::model::train_schedule::{
    SeatAvailabilityId, SeatId, StationRange, TrainSchedule, TrainScheduleId,
};

use crate::domain::repository::train_schedule::TrainScheduleRepository;
use crate::domain::{DbId, Identifiable, Repository, RepositoryError};
use anyhow::Context;
use async_trait::async_trait;
use chrono::{DateTime, FixedOffset, NaiveDate, Timelike};
use sea_orm::ColumnTrait;
use sea_orm::{ActiveValue, DatabaseConnection};
use sea_orm::{EntityTrait, TransactionTrait};
use sea_orm::{QueryFilter, Select};
use std::collections::HashMap;
use tracing::{error, instrument};

impl_db_id_from_u64!(TrainScheduleId, i32, "train schedule");
impl_db_id_from_u64!(SeatId, i64, "seat id");

/// 列车班次数据转换器
///
/// 负责领域模型 `TrainSchedule` 与数据库模型之间的双向转换。
///
/// # 转换逻辑
/// - 班次主体信息 ↔ `train_schedule` 表
/// - 座位占用信息 ↔ `occupied_seat` + `seat_type` + `seat_type_mapping` 表
pub struct TrainScheduleDataConverter;

pub struct TrainScheduleDoPack {
    train_schedule_do: crate::models::train_schedule::Model,
    seat_type: HashMap<i32, crate::models::seat_type::Model>,
    seat_availability_do_list: Vec<crate::models::seat_availability::Model>,
}

impl TrainScheduleDataConverter {
    fn make_from_do(
        train_schedule_do_pack: TrainScheduleDoPack,
    ) -> Result<TrainSchedule, anyhow::Error> {
        let train_schedule_do = train_schedule_do_pack.train_schedule_do;

        let mut train_schedule = TrainSchedule::new(
            Some(TrainScheduleId::from_db_value(train_schedule_do.id)?),
            TrainId::from_db_value(train_schedule_do.train_id)?,
            train_schedule_do.departure_date,
            train_schedule_do.origin_departure_time,
            RouteId::from_db_value(train_schedule_do.line_id)?,
        );

        for item in train_schedule_do_pack.seat_availability_do_list {
            let station_range = StationRange::from_unchecked(
                StationId::from_db_value(item.begin_station_id)?,
                StationId::from_db_value(item.end_station_id)?,
            );

            let seat_type_do = train_schedule_do_pack
                .seat_type
                .get(&item.seat_type_id)
                .context(format!(
                    "Inconsistent: cannot find seat type id: {}",
                    item.seat_type_id
                ))?;

            let seat_type = SeatType::new(
                Some(SeatTypeId::try_from(seat_type_do.id)?),
                SeatTypeName::from_unchecked(seat_type_do.type_name.clone()),
                seat_type_do.capacity as u32,
                seat_type_do.price,
            );

            train_schedule.add_seat_availability(
                station_range,
                seat_type,
                SeatAvailabilityId::from_db_value(item.id)?,
            );
        }

        Ok(train_schedule)
    }

    fn transform_to_do(
        train_schedule: TrainSchedule,
    ) -> crate::models::train_schedule::ActiveModel {
        let mut result = crate::models::train_schedule::ActiveModel {
            id: ActiveValue::NotSet,
            train_id: ActiveValue::Set(train_schedule.train_id().to_db_value()),
            line_id: ActiveValue::Set(train_schedule.route_id().to_db_value()),
            departure_date: ActiveValue::Set(train_schedule.date()),
            origin_departure_time: ActiveValue::Set(train_schedule.origin_departure_time()),
        };

        if let Some(id) = train_schedule.get_id() {
            result.id = ActiveValue::Set(id.to_db_value());
        }

        result
    }
}

/// 列车班次仓储实现
///
/// # 架构说明
/// 实现两种仓储接口：
/// - `TrainScheduleRepository`: 领域特定仓储接口
/// - `DbRepositorySupport`: 基础设施层仓储支持接口
///
/// # 并发设计
/// - 使用`Arc<Mutex<...>>`封装聚合管理器
/// - 所有数据库操作在事务中执行
pub struct TrainScheduleRepositoryImpl {
    /// 数据库连接池
    db: DatabaseConnection,
}

impl TrainScheduleRepositoryImpl {
    /// 创建新的仓储实例
    ///
    /// # Arguments
    /// * `db`: 数据库连接池
    ///
    /// # Initialization
    /// 初始化时会配置自定义的变更检测函数，用于：
    /// - 跟踪座位占用状态变化
    /// - 识别班次基础信息修改
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn query_train_schedule(
        &self,
        builder: impl FnOnce(
            Select<crate::models::train_schedule::Entity>,
        ) -> Select<crate::models::train_schedule::Entity>,
    ) -> Result<Vec<TrainSchedule>, RepositoryError> {
        let result = builder(crate::models::train_schedule::Entity::find())
            .all(&self.db)
            .await
            .context("Failed to find train schedule")?;

        let mut r = Vec::new();

        for train_schedule_do in result {
            let seat_availability_do_list = crate::models::seat_availability::Entity::find()
                .filter(
                    crate::models::seat_availability::Column::TrainScheduleId
                        .eq(train_schedule_do.id),
                )
                .all(&self.db)
                .await
                .context(format!(
                    "Failed to find seat availability for train schedule id: {}",
                    train_schedule_do.id
                ))?;

            let seat_type_list = crate::models::seat_type::Entity::find()
                .all(&self.db)
                .await
                .context(format!(
                    "Failed to find seat type for train schedule id: {}",
                    train_schedule_do.id
                ))?;

            let seat_type_map = seat_type_list
                .into_iter()
                .map(|item| (item.id, item))
                .collect::<HashMap<_, _>>();

            let train_schedule_do_pack = TrainScheduleDoPack {
                train_schedule_do,
                seat_availability_do_list,
                seat_type: seat_type_map,
            };

            r.push(TrainScheduleDataConverter::make_from_do(
                train_schedule_do_pack,
            )?);
        }

        Ok(r)
    }
}

#[async_trait]
impl Repository<TrainSchedule> for TrainScheduleRepositoryImpl {
    async fn find(&self, id: TrainScheduleId) -> Result<Option<TrainSchedule>, RepositoryError> {
        let result = self
            .query_train_schedule(|q| {
                q.filter(crate::models::train_schedule::Column::Id.eq(id.to_db_value()))
            })
            .await
            .context(format!(
                "failed to find train schedule with id: {}",
                id.to_db_value()
            ))?;

        Ok(result.into_iter().next())
    }

    async fn remove(&self, aggregate: TrainSchedule) -> Result<(), RepositoryError> {
        if let Some(id) = aggregate.get_id() {
            crate::models::train_schedule::Entity::delete_by_id(id.to_db_value())
                .exec(&self.db)
                .await
                .context(format!("Failed to delete train schedule with id: {}", id))?;
        }

        Ok(())
    }

    async fn save(
        &self,
        aggregate: &mut TrainSchedule,
    ) -> Result<TrainScheduleId, RepositoryError> {
        let transaction = self
            .db
            .begin()
            .await
            .context("Failed to begin transaction")?;

        let train_schedule_do = TrainScheduleDataConverter::transform_to_do(aggregate.clone());

        let train_schedule_id = match aggregate.get_id() {
            Some(id) => {
                crate::models::train_schedule::Entity::update(train_schedule_do)
                    .exec(&self.db)
                    .await
                    .context(format!("Failed to update train schedule with id: {}", id))?;
                id
            }
            None => {
                let result = crate::models::train_schedule::Entity::insert(train_schedule_do)
                    .exec(&self.db)
                    .await
                    .context("Failed to insert new train schedule")?;
                TrainScheduleId::from_db_value(result.last_insert_id)?
            }
        };

        transaction
            .commit()
            .await
            .context("Failed to commit transaction")?;

        Ok(train_schedule_id)
    }
}

#[async_trait]
impl TrainScheduleRepository for TrainScheduleRepositoryImpl {
    #[instrument(skip(self))]
    async fn find_by_date(&self, date: NaiveDate) -> Result<Vec<TrainSchedule>, RepositoryError> {
        Ok(self
            .query_train_schedule(|q| {
                q.filter(crate::models::train_schedule::Column::DepartureDate.eq(date))
            })
            .await
            .inspect_err(|e| {
                error!("failed to find train schedule: {}", e);
            })
            .context(format!("Failed to find train schedule with date: {}", date))?)
    }

    #[instrument(skip(self))]
    async fn find_by_train_id(
        &self,
        train_id: TrainId,
    ) -> Result<Vec<TrainSchedule>, RepositoryError> {
        Ok(self
            .query_train_schedule(|q| {
                q.filter(crate::models::train_schedule::Column::TrainId.eq(train_id.to_db_value()))
            })
            .await
            .inspect_err(|e| {
                error!("failed to find train schedule: {}", e);
            })
            .context(format!(
                "Failed to find train schedule with train id: {}",
                train_id.to_db_value()
            ))?)
    }

    #[instrument(skip(self))]
    async fn find_by_train_id_and_origin_departure_time(
        &self,
        train_id: TrainId,
        origin_departure_time: DateTime<FixedOffset>,
    ) -> Result<Option<TrainSchedule>, RepositoryError> {
        let departure_date = origin_departure_time.date_naive();

        let origin_departure_time = origin_departure_time.time().num_seconds_from_midnight() as i32;

        let result = self
            .query_train_schedule(|q| {
                q.filter(
                    crate::models::train_schedule::Column::TrainId
                        .eq(train_id.to_db_value())
                        .and(
                            crate::models::train_schedule::Column::DepartureDate.eq(departure_date),
                        )
                        .and(
                            crate::models::train_schedule::Column::OriginDepartureTime
                                .eq(origin_departure_time),
                        ),
                )
            })
            .await
            .inspect_err(|e| {
                error!("failed to find train schedule: {}", e);
            })
            .context(format!(
                "Failed to find train schedule with train id: {} and origin departure time: {}",
                train_id.to_db_value(),
                origin_departure_time
            ))?;

        Ok(result.into_iter().next())
    }
}
