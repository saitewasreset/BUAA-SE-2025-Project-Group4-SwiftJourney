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
//! - 自动变更检测与跟踪
//! - 联表查询优化（班次+占用座位+座位类型+座位映射）
//!
//! # 数据模型转换
//! 本模块实现了领域模型与数据库模型的互相转换：
//! - `TrainScheduleDataConverter`: 处理班次聚合根的转换
//! - `OccupiedSeatConverter`: 处理占用座位实体的转换
//!
//! # 并发安全
//! - 使用`Arc<Mutex<AggregateManager>>`保证变更检测的线程安全
//! - 所有数据库操作在事务中执行
use crate::domain::model::personal_info::PersonalInfoId;
use crate::domain::model::route::RouteId;
use crate::domain::model::station::StationId;
use crate::domain::model::train::{SeatType, SeatTypeId, SeatTypeName, TrainId};
use crate::domain::model::train_schedule::{
    OccupiedSeat, Seat, SeatAvailability, SeatAvailabilityMap, SeatId, SeatLocationInfo,
    StationRange, TrainSchedule, TrainScheduleId,
};
use crate::domain::repository::train_schedule::TrainScheduleRepository;
use crate::domain::service::{AggregateManagerImpl, DiffInfo};
use crate::domain::{
    DbId, DbRepositorySupport, DiffType, Identifiable, MultiEntityDiff, RepositoryError, TypedDiff,
};
use anyhow::{Context, anyhow};
use async_trait::async_trait;
use chrono::NaiveDate;
use sea_orm::sea_query::OnConflict;
use sea_orm::{ActiveModelTrait, ColumnTrait};
use sea_orm::{ActiveValue, DatabaseConnection};
use sea_orm::{EntityTrait, TransactionTrait};
use sea_orm::{QueryFilter, Select};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

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

/// 占用座位数据转换器
///
/// 负责领域模型 `OccupiedSeat` 与数据库模型 `occupied_seat` 之间的双向转换。
pub struct OccupiedSeatConverter;

/// 数据库模型打包结构
struct TrainScheduleDoPack {
    /// 班次主表数据
    train_schedule: crate::models::train_schedule::Model,
    /// 已占用座位列表
    occupied_seat: Vec<crate::models::occupied_seat::Model>,
    /// 座位类型字典(seat_type_id → 模型)
    seat_type: HashMap<i32, crate::models::seat_type::Model>,
    /// 座位位置映射字典(seat_type_id → seat_id → 映射模型)
    seat_type_mapping: HashMap<i32, HashMap<i64, crate::models::seat_type_mapping::Model>>,
}

impl TrainScheduleDataConverter {
    /// 从数据库模型创建领域模型
    ///
    /// # Arguments
    /// * `pack`: 从数据库查询出的打包数据
    ///
    /// # Process
    /// 1. 转换基础班次信息
    /// 2. 构建座位可用性映射
    /// 3. 组装为聚合根
    ///
    /// # Errors
    /// 在以下情况返回错误：
    /// - 数据不一致（如缺失关联记录）
    /// - 无效的ID格式
    /// - 字段验证失败
    fn make_from_do(pack: TrainScheduleDoPack) -> anyhow::Result<TrainSchedule> {
        let mut seat_availability_map: SeatAvailabilityMap = HashMap::new();
        let train_schedule_id = TrainScheduleId::from_db_value(pack.train_schedule.id)?;
        let train_id = TrainId::from_db_value(pack.train_schedule.train_id)?;
        let route_id = RouteId::from_db_value(pack.train_schedule.line_id)?;

        for occupied_seat in pack.occupied_seat {
            let seat_id = SeatId::from_db_value(occupied_seat.seat_id)?;

            let seat_type = {
                let model = pack
                    .seat_type
                    .get(&occupied_seat.seat_type_id)
                    .cloned()
                    .ok_or(anyhow!(
                        "inconsistent: no seat type for seat type id: {}",
                        occupied_seat.seat_type_id
                    ))?;

                let seat_type_id = SeatTypeId::from_db_value(model.id)?;

                SeatType::new(
                    Some(seat_type_id),
                    SeatTypeName::from_unchecked(model.type_name),
                    model.capacity as u32,
                    model.price,
                )
            };

            let seat_location = {
                let model = pack
                    .seat_type_mapping
                    .get(&occupied_seat.seat_type_id)
                    .ok_or(anyhow!(
                        "inconsistent: no seat type for seat type id: {}",
                        occupied_seat.seat_type_id
                    ))?
                    .get(&occupied_seat.seat_id)
                    .ok_or(anyhow!(
                        "inconsistent: no seat mapping for seat type id: {} seat id: {}",
                        occupied_seat.seat_type_id,
                        occupied_seat.seat_id
                    ))?;

                SeatLocationInfo {
                    carriage: model.carriage,
                    row: model.row,
                    location: model.location.chars().next().ok_or(anyhow!("inconsistent: seat location should not be null for seat type id: {}, seat id: {}", occupied_seat.seat_type_id, occupied_seat.seat_id))?,
                }
            };

            let seat = Seat::new(seat_id, seat_type.clone(), seat_location);

            let personal_info_id = PersonalInfoId::from_db_value(occupied_seat.person_info_id)?;

            let station_range = {
                let from_station_id = StationId::from_db_value(occupied_seat.begin_station_id)?;
                let to_station_id = StationId::from_db_value(occupied_seat.end_station_id)?;

                StationRange::from_unchecked(from_station_id, to_station_id)
            };

            let entry = seat_availability_map
                .entry(station_range)
                .or_default()
                .entry(seat_type.clone())
                .or_insert(SeatAvailability::new(seat_type.clone(), station_range));

            entry.add_occupied_seat(Some(train_schedule_id), seat, personal_info_id);
        }

        Ok(TrainSchedule::new(
            Some(train_schedule_id),
            train_id,
            pack.train_schedule.departure_date,
            route_id,
            seat_availability_map,
        ))
    }

    /// 转换领域模型到ActiveModel模型（班次主表）
    ///
    /// # Arguments
    /// * `train_schedule`: 要转换的班次聚合根
    ///
    /// # Note
    /// 不包含关联的座位占用数据转换
    pub fn transform_to_train_schedule_do(
        train_schedule: &TrainSchedule,
    ) -> crate::models::train_schedule::ActiveModel {
        let mut train_schedule_active_model = crate::models::train_schedule::ActiveModel {
            id: ActiveValue::NotSet,
            train_id: ActiveValue::Set(train_schedule.train_id().to_db_value()),
            departure_date: ActiveValue::Set(train_schedule.date()),
            line_id: ActiveValue::Set(train_schedule.route_id().to_db_value()),
        };

        if let Some(id) = train_schedule.get_id() {
            train_schedule_active_model.id = ActiveValue::Set(id.to_db_value());
        }

        train_schedule_active_model
    }

    /// 转换领域模型到ActiveModel模型列表（占用座位表）
    ///
    /// # Arguments
    /// * `train_schedule`: 要转换的班次聚合根
    /// * `train_schedule_id`: 关联的班次数据库ID
    ///
    /// # Returns
    /// 返回可以批量插入的ActiveRecord列表
    pub fn transform_to_occupied_seat_do(
        train_schedule: TrainSchedule,
        train_schedule_id: i32,
    ) -> Vec<crate::models::occupied_seat::ActiveModel> {
        let mut result = Vec::new();

        for seat_availability in train_schedule.into_seat_availability() {
            let station_range = seat_availability.station_range();
            let seat_type = seat_availability.seat_type().clone();

            for (seat_id, occupied_seat) in seat_availability.into_occupied_seat() {
                if let Some(seat_type_id) = seat_type.get_id() {
                    let occupied_seat_active_model = crate::models::occupied_seat::ActiveModel {
                        train_schedule_id: ActiveValue::Set(train_schedule_id),
                        seat_type_id: ActiveValue::Set(seat_type_id.to_db_value()),
                        seat_id: ActiveValue::Set(seat_id.to_db_value()),
                        begin_station_id: ActiveValue::Set(
                            station_range.get_from_station_id().to_db_value(),
                        ),
                        end_station_id: ActiveValue::Set(
                            station_range.get_to_station_id().to_db_value(),
                        ),
                        person_info_id: ActiveValue::Set(
                            occupied_seat.passenger_id().to_db_value(),
                        ),
                    };

                    result.push(occupied_seat_active_model);
                }
            }
        }

        result
    }
}

impl OccupiedSeatConverter {
    /// 从数据库模型创建占用座位实体
    ///
    /// # Arguments
    /// * `occupied_seat`: 占用座位主表记录
    /// * `seat_type`: 关联的座位类型记录
    /// * `seat_type_mapping`: 座位位置映射记录
    ///
    /// # Validation
    /// - 检查座位位置字符不为空
    /// - 验证所有ID格式有效
    pub fn make_from_do(
        occupied_seat: crate::models::occupied_seat::Model,
        seat_type: crate::models::seat_type::Model,
        seat_type_mapping: crate::models::seat_type_mapping::Model,
    ) -> anyhow::Result<OccupiedSeat> {
        let seat_id = SeatId::from_db_value(occupied_seat.seat_id)?;
        let seat_type_id = SeatTypeId::from_db_value(seat_type.id)?;
        let personal_info_id = PersonalInfoId::from_db_value(occupied_seat.person_info_id)?;

        let seat_location = SeatLocationInfo {
            carriage: seat_type_mapping.carriage,
            row: seat_type_mapping.row,
            location: seat_type_mapping.location.chars().next().ok_or(anyhow!(
                "inconsistent: seat location should not be null for seat type id: {}, seat id: {}",
                occupied_seat.seat_type_id,
                occupied_seat.seat_id
            ))?,
        };

        let station_range = StationRange::from_unchecked(
            StationId::from_db_value(occupied_seat.begin_station_id)?,
            StationId::from_db_value(occupied_seat.end_station_id)?,
        );

        let seat_type = SeatType::new(
            Some(seat_type_id),
            SeatTypeName::from_unchecked(seat_type.type_name),
            seat_type.capacity as u32,
            seat_type.price,
        );

        let seat = Seat::new(seat_id, seat_type, seat_location);

        Ok(OccupiedSeat::new(
            Some(TrainScheduleId::from_db_value(
                occupied_seat.train_schedule_id,
            )?),
            seat_type_id,
            station_range,
            seat,
            personal_info_id,
        ))
    }

    /// 转换占用座位实体到ActiveModel模型
    ///
    /// # Arguments
    /// * `occupied_seat`: 要转换的占用座位实体
    pub fn transform_to_do(
        occupied_seat: &OccupiedSeat,
    ) -> crate::models::occupied_seat::ActiveModel {
        let mut model = crate::models::occupied_seat::ActiveModel {
            train_schedule_id: ActiveValue::NotSet,
            seat_type_id: ActiveValue::NotSet,
            seat_id: ActiveValue::NotSet,
            begin_station_id: ActiveValue::Set(
                occupied_seat
                    .station_range()
                    .get_from_station_id()
                    .to_db_value(),
            ),
            end_station_id: ActiveValue::Set(
                occupied_seat
                    .station_range()
                    .get_to_station_id()
                    .to_db_value(),
            ),
            person_info_id: ActiveValue::Set(occupied_seat.passenger_id().to_db_value()),
        };

        if let Some(id) = occupied_seat.get_id() {
            model.train_schedule_id = ActiveValue::Set(id.train_schedule_id().to_db_value());
            model.seat_type_id = ActiveValue::Set(id.seat_type_id().to_db_value());
            model.seat_id = ActiveValue::Set(id.seat_id().to_db_value());
        }

        model
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
    /// 聚合根变更管理器
    aggregate_manager: Arc<Mutex<AggregateManagerImpl<TrainSchedule>>>,
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
        let detect_changes_fn = |diff: DiffInfo<TrainSchedule>| {
            let mut result = MultiEntityDiff::new();

            let old = diff.old;
            let new = diff.new;

            // 实体第一次存入数据库后（有ID后）才能跟踪
            let old_occupied_seat_map = old
                .as_ref()
                .map(|x| x.occupied_entry_iter())
                .into_iter()
                .flatten()
                .cloned()
                .filter(|occupied_seat| occupied_seat.get_id().is_some())
                .map(|occupied_seat| (occupied_seat.get_id().unwrap(), occupied_seat))
                .collect::<HashMap<_, _>>();
            let new_occupied_seat_map = new
                .as_ref()
                .map(|x| x.occupied_entry_iter())
                .into_iter()
                .flatten()
                .cloned()
                .filter(|occupied_seat| occupied_seat.get_id().is_some())
                .map(|occupied_seat| (occupied_seat.get_id().unwrap(), occupied_seat))
                .collect::<HashMap<_, _>>();

            for (id, occupied_seat) in &old_occupied_seat_map {
                if let Some(new_occupied_seat) = new_occupied_seat_map.get(id) {
                    if new_occupied_seat != occupied_seat {
                        result.add_change::<OccupiedSeat>(TypedDiff::new(
                            DiffType::Modified,
                            Some(occupied_seat.clone()),
                            Some(new_occupied_seat.clone()),
                        ));
                    }
                } else {
                    result.add_change::<OccupiedSeat>(TypedDiff::new(
                        DiffType::Removed,
                        Some(occupied_seat.clone()),
                        None,
                    ));
                }
            }

            for (id, occupied_seat) in &new_occupied_seat_map {
                if !old_occupied_seat_map.contains_key(id) {
                    result.add_change::<OccupiedSeat>(TypedDiff::new(
                        DiffType::Added,
                        None,
                        Some(occupied_seat.clone()),
                    ));
                }
            }

            let aggregate_root_diff_type =
                DiffType::from_with_compare_fn(old.as_ref(), new.as_ref(), |old, new| {
                    old.train_id() == new.train_id()
                        && old.date() == new.date()
                        && old.route_id() == new.route_id()
                });

            result.add_change(TypedDiff::new(aggregate_root_diff_type, old, new));

            result
        };

        Self {
            db,
            aggregate_manager: Arc::new(Mutex::new(AggregateManagerImpl::new(Box::new(
                detect_changes_fn,
            )))),
        }
    }

    /// 执行联表查询并转换为领域模型
    ///
    /// # Arguments
    /// * `builder`: 构造查询条件的闭包
    ///
    /// # Query Strategy
    /// 1. 查询班次主表+列车信息
    /// 2. 批量查询所有关联座位类型
    /// 3. 批量查询所有座位位置映射
    /// 4. 按班次查询占用座位记录
    ///
    /// # Performance
    /// 使用批量查询减少数据库往返次数
    async fn query_train_schedules_eagerly(
        &self,
        builder: impl FnOnce(
            Select<crate::models::train_schedule::Entity>,
        ) -> Select<crate::models::train_schedule::Entity>,
    ) -> Result<Vec<TrainSchedule>, RepositoryError> {
        let mut result = Vec::new();

        let train_schedule_list = builder(crate::models::train_schedule::Entity::find())
            .find_also_related(crate::models::train::Entity)
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::Db(e.into()))?;

        let seat_type = crate::models::seat_type::Entity::find()
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::Db(e.into()))?;

        let seat_type_map: HashMap<_, _> = seat_type.into_iter().map(|x| (x.id, x)).collect();

        let seat_type_mapping = crate::models::seat_type_mapping::Entity::find()
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::Db(e.into()))?;

        // train_type_id -> seat_type_id -> seat_id
        let mut seat_type_mapping_map: HashMap<
            i32,
            HashMap<i32, HashMap<i64, crate::models::seat_type_mapping::Model>>,
        > = HashMap::new();

        for seat_type_mapping in seat_type_mapping {
            seat_type_mapping_map
                .entry(seat_type_mapping.train_type_id)
                .or_default()
                .entry(seat_type_mapping.seat_type_id)
                .or_default()
                .insert(seat_type_mapping.seat_id, seat_type_mapping);
        }

        for (train_schedule, train) in train_schedule_list {
            let train = train.ok_or(RepositoryError::InconsistentState(anyhow!(
                "no train for train schedule id: {}",
                train_schedule.id
            )))?;

            let occupied_seat = crate::models::occupied_seat::Entity::find()
                .filter(crate::models::occupied_seat::Column::TrainScheduleId.eq(train_schedule.id))
                .all(&self.db)
                .await
                .map_err(|e| RepositoryError::Db(e.into()))?;

            let pack = TrainScheduleDoPack {
                train_schedule,
                occupied_seat,
                seat_type: seat_type_map.clone(),
                seat_type_mapping: seat_type_mapping_map
                    .get(&train.id)
                    .cloned()
                    .unwrap_or_default(),
            };

            result.push(
                TrainScheduleDataConverter::make_from_do(pack)
                    .map_err(RepositoryError::ValidationError)?,
            )
        }

        Ok(result)
    }
}

#[async_trait]
impl DbRepositorySupport<TrainSchedule> for TrainScheduleRepositoryImpl {
    type Manager = AggregateManagerImpl<TrainSchedule>;
    /// 获取聚合管理器引用
    ///
    /// # Note
    /// 返回的Arc<Mutex<..>>保证线程安全访问
    fn get_aggregate_manager(&self) -> Arc<Mutex<Self::Manager>> {
        Arc::clone(&self.aggregate_manager)
    }

    /// 插入新班次
    ///
    /// # Transaction
    /// 在单个事务中完成：
    /// 1. 插入班次主表记录
    /// 2. 批量插入占用座位记录
    ///
    /// # Conflict
    /// 采用ON CONFLICT UPDATE策略处理冲突
    async fn on_insert(
        &self,
        aggregate: TrainSchedule,
    ) -> Result<TrainScheduleId, RepositoryError> {
        let txn = self
            .db
            .begin()
            .await
            .context("failed to start transaction")?;

        let train_schedule_active_model =
            TrainScheduleDataConverter::transform_to_train_schedule_do(&aggregate);

        let train_schedule_id = if let Some(id) = aggregate.get_id() {
            crate::models::train_schedule::Entity::update_many()
                .filter(crate::models::train_schedule::Column::Id.eq(id.to_db_value()))
                .set(train_schedule_active_model)
                .exec(&txn)
                .await
                .context(format!(
                    "failed to update train schedule with id: {}",
                    id.to_db_value()
                ))?;
            id
        } else {
            let train_schedule =
                crate::models::train_schedule::Entity::insert(train_schedule_active_model)
                    .exec(&txn)
                    .await
                    .context("failed to insert train schedule")?;

            TrainScheduleId::from_db_value(train_schedule.last_insert_id)?
        };

        let occupied_seat = TrainScheduleDataConverter::transform_to_occupied_seat_do(
            aggregate.clone(),
            train_schedule_id.to_db_value(),
        );

        crate::models::occupied_seat::Entity::insert_many(occupied_seat)
            .on_conflict(
                OnConflict::columns([
                    crate::models::occupied_seat::Column::TrainScheduleId,
                    crate::models::occupied_seat::Column::SeatTypeId,
                    crate::models::occupied_seat::Column::SeatId,
                ])
                .update_columns([
                    crate::models::occupied_seat::Column::BeginStationId,
                    crate::models::occupied_seat::Column::EndStationId,
                    crate::models::occupied_seat::Column::PersonInfoId,
                ])
                .to_owned(),
            )
            .exec(&txn)
            .await
            .context("failed to insert occupied seat")?;

        txn.commit().await.context("failed to commit transaction")?;

        Ok(train_schedule_id)
    }

    /// 查询班次
    ///
    /// # Eager Loading
    /// 通过`query_train_schedules_eagerly`实现急切加载所有关联数据
    async fn on_select(
        &self,
        id: TrainScheduleId,
    ) -> Result<Option<TrainSchedule>, RepositoryError> {
        let train_schedule = crate::models::train_schedule::Entity::find_by_id(id.to_db_value())
            .find_also_related(crate::models::train::Entity)
            .one(&self.db)
            .await
            .context(format!(
                "failed to find train schedule with id: {}",
                id.to_db_value()
            ))?;

        if let Some((train_schedule, train)) = train_schedule {
            let train = train.ok_or(anyhow!(
                "inconsistent: train schedule id {} without train",
                id.to_db_value()
            ))?;

            let occupied_seat = crate::models::occupied_seat::Entity::find()
                .filter(crate::models::occupied_seat::Column::TrainScheduleId.eq(train_schedule.id))
                .all(&self.db)
                .await
                .context(format!(
                    "failed to find related occupied seat for train schedule with id: {}",
                    id.to_db_value()
                ))?;

            let seat_type = crate::models::seat_type::Entity::find()
                .all(&self.db)
                .await
                .context("failed to find seat type")?;

            let seat_type_map: HashMap<_, _> = seat_type.into_iter().map(|m| (m.id, m)).collect();

            let seat_type_mapping = crate::models::seat_type_mapping::Entity::find()
                .filter(crate::models::seat_type_mapping::Column::TrainTypeId.eq(train.type_id))
                .all(&self.db)
                .await
                .context(format!(
                    "failed to find related seat type mapping for train schedule with id: {}",
                    id.to_db_value()
                ))?;

            let mut seat_type_mapping_map: HashMap<
                i32,
                HashMap<i64, crate::models::seat_type_mapping::Model>,
            > = HashMap::new();

            for seat_type_mapping in seat_type_mapping {
                seat_type_mapping_map
                    .entry(seat_type_mapping.seat_type_id)
                    .or_default()
                    .insert(seat_type_mapping.seat_id, seat_type_mapping);
            }

            let pack = TrainScheduleDoPack {
                train_schedule,
                occupied_seat,
                seat_type: seat_type_map,
                seat_type_mapping: seat_type_mapping_map,
            };

            Ok(Some(
                TrainScheduleDataConverter::make_from_do(pack)
                    .map_err(RepositoryError::ValidationError)?,
            ))
        } else {
            Ok(None)
        }
    }

    /// 处理变更集
    ///
    /// # Dispatch Logic
    /// 根据变更类型分发操作：
    /// - Added: 插入记录
    /// - Modified: 更新记录
    /// - Removed: 删除记录
    async fn on_update(&self, diff: MultiEntityDiff) -> Result<(), RepositoryError> {
        for changes in diff.get_changes::<OccupiedSeat>() {
            match changes.diff_type {
                DiffType::Unchanged => {}
                DiffType::Added => {
                    let new = changes.new_value.unwrap();
                    OccupiedSeatConverter::transform_to_do(&new)
                        .insert(&self.db)
                        .await
                        .context(format!(
                            "failed to add occupied seat with id: {:?}",
                            new.get_id()
                        ))
                        .map_err(RepositoryError::Db)?;
                }
                DiffType::Modified => {
                    let new = changes.new_value.unwrap();
                    OccupiedSeatConverter::transform_to_do(&new)
                        .update(&self.db)
                        .await
                        .context(format!(
                            "failed to add occupied seat with id: {:?}",
                            new.get_id()
                        ))
                        .map_err(RepositoryError::Db)?;
                }
                DiffType::Removed => {
                    let old = changes.old_value.unwrap();

                    if let Some(id) = old.get_id() {
                        crate::models::occupied_seat::Entity::delete_by_id((
                            id.train_schedule_id().to_db_value(),
                            id.seat_type_id().to_db_value(),
                            id.seat_id().to_db_value(),
                        ))
                        .exec(&self.db)
                        .await
                        .context(format!("failed to remove occupied seat with id: {:?}", id))?;
                    }
                }
            }
        }

        for changes in diff.get_changes::<TrainSchedule>() {
            match changes.diff_type {
                DiffType::Unchanged => {}
                DiffType::Added => {
                    let new = changes.new_value.unwrap();
                    TrainScheduleDataConverter::transform_to_train_schedule_do(&new)
                        .insert(&self.db)
                        .await
                        .context(format!(
                            "failed to add train schedule with id: {:?}",
                            new.get_id()
                        ))
                        .map_err(RepositoryError::Db)?;
                }
                DiffType::Modified => {
                    let new = changes.new_value.unwrap();
                    TrainScheduleDataConverter::transform_to_train_schedule_do(&new)
                        .update(&self.db)
                        .await
                        .context(format!(
                            "failed to add train schedule with id: {:?}",
                            new.get_id()
                        ))
                        .map_err(RepositoryError::Db)?;
                }
                DiffType::Removed => {
                    let old = changes.old_value.unwrap();

                    if let Some(id) = old.get_id() {
                        crate::models::train_schedule::Entity::delete_by_id(id.to_db_value())
                            .exec(&self.db)
                            .await
                            .context(format!(
                                "failed to remove train schedule with id: {:?}",
                                id
                            ))?;
                    }
                }
            }
        }

        Ok(())
    }

    /// 删除班次
    ///
    /// # Cascade
    /// 依赖数据库外键的ON DELETE CASCADE自动清理关联座位记录
    async fn on_delete(&self, aggregate: TrainSchedule) -> Result<(), RepositoryError> {
        if let Some(id) = aggregate.get_id() {
            crate::models::train_schedule::Entity::delete_by_id(id.to_db_value())
                .exec(&self.db)
                .await
                .context(format!(
                    "failed to remove train schedule with id: {}",
                    id.to_db_value()
                ))?;
        }

        Ok(())
    }
}

#[async_trait]
impl TrainScheduleRepository for TrainScheduleRepositoryImpl {
    /// 按日期查询班次
    ///
    /// # Query
    /// - 精确匹配出发日期
    /// - 返回完整聚合对象(包含所有关联数据)
    async fn find_by_date(&self, date: NaiveDate) -> Result<Vec<TrainSchedule>, RepositoryError> {
        self.query_train_schedules_eagerly(|q| {
            q.filter(crate::models::train_schedule::Column::DepartureDate.eq(date))
        })
        .await
    }

    /// 按列车ID查询班次
    ///
    /// # Query
    /// - 精确匹配列车模板ID
    /// - 返回该列车的所有运行班次
    async fn find_by_train_id(
        &self,
        train_id: TrainId,
    ) -> Result<Vec<TrainSchedule>, RepositoryError> {
        self.query_train_schedules_eagerly(|q| {
            q.filter(crate::models::train_schedule::Column::TrainId.eq(train_id.to_db_value()))
        })
        .await
    }
}
