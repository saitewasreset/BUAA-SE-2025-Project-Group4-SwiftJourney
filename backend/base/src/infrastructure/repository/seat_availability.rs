//! 座位可用性仓储实现模块
//!
//! 本模块提供座位可用性(`SeatAvailability`)及其相关实体(如`OccupiedSeat`)的数据库持久化实现。
//! 主要功能包括：
//! - 座位可用性记录的CRUD操作
//! - 已占用座位记录的CRUD操作
//! - 座位类型和位置映射的关联查询
//! - 聚合根变更检测与同步
//!
//! 模块使用SeaORM作为ORM框架，通过事务保证数据一致性。
//!
//! # 主要结构体
//! - [`SeatAvailabilityRepositoryImpl`][]: 座位可用性资源库实现
//! - [`SeatAvailabilityDataConverter`][]: 座位可用性数据转换器
//! - [`OccupiedSeatDataConverter`][]: 已占用座位数据转换器
//!
//! # 数据模型
//! 模块涉及以下数据库表：
//! - `seat_availability`: 座位可用性主表
//! - `occupied_seat`: 已占用座位表
//! - `seat_type`: 座位类型表
//! - `seat_type_mapping`: 座位类型映射表
use crate::domain::model::personal_info::PersonalInfoId;
use crate::domain::model::station::StationId;
use crate::domain::model::train::{SeatType, SeatTypeId, SeatTypeName};
use crate::domain::model::train_schedule::{
    OccupiedSeat, Seat, SeatAvailability, SeatAvailabilityId, SeatId, SeatLocationInfo, SeatStatus,
    StationRange, TrainScheduleId,
};
use crate::domain::repository::seat_availability::{
    OccupiedSeatInfoMap, SeatAvailabilityRepository,
};
use crate::domain::service::{AggregateManagerImpl, DiffInfo};
use crate::domain::{
    DbId, DbRepositorySupport, DiffType, Identifiable, MultiEntityDiff, RepositoryError, TypedDiff,
};
use anyhow::{Context, anyhow};
use async_trait::async_trait;
use sea_orm::{
    ActiveValue, DatabaseBackend, DatabaseConnection, EntityTrait, FromQueryResult, QueryFilter,
    Statement, TransactionTrait,
};
use sea_orm::{ColumnTrait, DbErr};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

impl_db_id_from_u64!(SeatAvailabilityId, i32, "seat availability");

/// 座位可用性数据转换器
///
/// 负责`SeatAvailability`领域对象与数据库模型之间的转换
pub struct SeatAvailabilityDataConverter;

/// 已占用座位数据转换器
///
/// 负责`OccupiedSeat`领域对象与数据库模型之间的转换
pub struct OccupiedSeatDataConverter;

/// 座位可用性数据库模型包
///
/// 包含从数据库查询出的相关模型数据，用于构建领域对象
struct SeatAvailabilityDoPack {
    seat_availability: crate::models::seat_availability::Model,
    /// 已占用座位列表
    occupied_seat: Vec<crate::models::occupied_seat::Model>,
    /// 座位类型字典(seat_type_id → 模型)
    seat_type: HashMap<i32, crate::models::seat_type::Model>,
    /// 座位位置映射字典(seat_type_id → seat_id → 映射模型)
    seat_type_mapping: HashMap<i32, HashMap<i64, crate::models::seat_type_mapping::Model>>,
}

/// 座位可用性活动模型包
///
/// 包含准备写入数据库的活动模型数据
struct SeatAvailabilityActiveModelPack {
    seat_availability: crate::models::seat_availability::ActiveModel,
    occupied_seat: Vec<crate::models::occupied_seat::ActiveModel>,
}

impl OccupiedSeatDataConverter {
    /// 从数据库模型创建领域对象
    ///
    /// # Arguments
    /// * `occupied_seat_do` - 数据库中的已占用座位记录
    /// * `seat_availability` - 关联的座位可用性记录
    /// * `seat_type` - 座位类型字典
    /// * `seat_type_mapping` - 座位位置映射字典
    ///
    /// # Returns
    /// 返回转换后的`OccupiedSeat`领域对象
    ///
    /// # Errors
    /// 当数据不一致或转换失败时返回错误
    pub fn make_from_do(
        occupied_seat_do: crate::models::occupied_seat::Model,
        seat_availability: &crate::models::seat_availability::Model,
        seat_type: &HashMap<i32, crate::models::seat_type::Model>,
        seat_type_mapping: &HashMap<i32, HashMap<i64, crate::models::seat_type_mapping::Model>>,
    ) -> Result<OccupiedSeat, anyhow::Error> {
        let seat_id = SeatId::try_from(occupied_seat_do.seat_id)?;

        let personal_info_id = PersonalInfoId::try_from(occupied_seat_do.person_info_id)?;

        let seat_type_id = seat_availability.seat_type_id;

        let seat_type_do = seat_type.get(&seat_type_id).context(format!(
            "Inconsistent: cannot find seat type id: {}",
            seat_type_id
        ))?;

        let seat_type = SeatType::new(
            Some(SeatTypeId::try_from(seat_type_do.id)?),
            SeatTypeName::from_unchecked(seat_type_do.type_name.clone()),
            seat_type_do.capacity as u32,
            seat_type_do.price,
        );

        let seat_type_mapping_do = seat_type_mapping
            .get(&seat_type_id)
            .context(format!(
                "Inconsistent: cannot find seat type mapping with seat type id: {}",
                seat_type_id
            ))?
            .get(&occupied_seat_do.seat_id)
            .context(format!(
                "Inconsistent: cannot find seat type mapping with seat id: {}",
                occupied_seat_do.seat_id
            ))?;

        let seat_location_info = SeatLocationInfo {
            carriage: seat_type_mapping_do.carriage,
            row: seat_type_mapping_do.row,
            location: seat_type_mapping_do
                .location
                .chars()
                .next()
                .expect("location should not be empty"),
        };

        let seat = Seat::new(seat_id, seat_type, seat_location_info, SeatStatus::Occupied);

        Ok(OccupiedSeat::new(
            SeatAvailabilityId::try_from(seat_availability.id)?,
            seat,
            personal_info_id,
        ))
    }

    /// 将领域对象转换为数据库活动模型
    ///
    /// # Arguments
    /// * `occupied_seat` - 要转换的已占用座位领域对象
    ///
    /// # Returns
    /// 返回可用于数据库操作的ActiveModel
    pub fn transform_to_do(
        occupied_seat: OccupiedSeat,
    ) -> crate::models::occupied_seat::ActiveModel {
        crate::models::occupied_seat::ActiveModel {
            seat_availability_id: ActiveValue::Set(
                occupied_seat
                    .get_id()
                    .unwrap()
                    .seat_availability_id()
                    .to_db_value(),
            ),
            seat_id: ActiveValue::Set(occupied_seat.seat().get_id().unwrap().to_db_value()),
            person_info_id: ActiveValue::Set(occupied_seat.passenger_id().to_db_value()),
        }
    }
}

impl SeatAvailabilityDataConverter {
    /// 从数据库模型包创建座位可用性领域对象
    ///
    /// # Arguments
    /// * `seat_availability_do_pack` - 包含所有相关数据的模型包
    ///
    /// # Returns
    /// 返回转换后的`SeatAvailability`领域对象
    ///
    /// # Errors
    /// 当数据不一致或转换失败时返回错误
    fn make_from_do(
        seat_availability_do_pack: SeatAvailabilityDoPack,
    ) -> Result<SeatAvailability, anyhow::Error> {
        let seat_availability_id = seat_availability_do_pack.seat_availability.id;

        for seat in &seat_availability_do_pack.occupied_seat {
            if seat.seat_availability_id != seat_availability_id {
                return Err(anyhow!(
                    "Inconsistent: seat availability id {} not match with occupied seat id {}",
                    seat_availability_id,
                    seat.seat_availability_id
                ));
            }
        }

        let train_schedule_id = TrainScheduleId::from_db_value(
            seat_availability_do_pack
                .seat_availability
                .train_schedule_id,
        )?;

        let seat_type_do = seat_availability_do_pack
            .seat_type
            .get(&seat_availability_do_pack.seat_availability.seat_type_id)
            .context(format!(
                "Inconsistent: cannot find seat type id: {}",
                seat_availability_do_pack.seat_availability.seat_type_id
            ))?;

        let seat_type = SeatType::new(
            Some(SeatTypeId::try_from(seat_type_do.id)?),
            SeatTypeName::from_unchecked(seat_type_do.type_name.clone()),
            seat_type_do.capacity as u32,
            seat_type_do.price,
        );

        let station_range = StationRange::from_unchecked(
            StationId::from_db_value(seat_availability_do_pack.seat_availability.begin_station_id)?,
            StationId::from_db_value(seat_availability_do_pack.seat_availability.end_station_id)?,
        );

        let mut occupied_seat_map = HashMap::new();

        for seat in seat_availability_do_pack.occupied_seat {
            let seat_id = SeatId::from_db_value(seat.seat_id)?;

            let occupied_set = OccupiedSeatDataConverter::make_from_do(
                seat,
                &seat_availability_do_pack.seat_availability,
                &seat_availability_do_pack.seat_type,
                &seat_availability_do_pack.seat_type_mapping,
            )?;

            occupied_seat_map.insert(seat_id, occupied_set);
        }

        Ok(SeatAvailability::new(
            Some(SeatAvailabilityId::try_from(
                seat_availability_do_pack.seat_availability.id,
            )?),
            train_schedule_id,
            seat_type,
            station_range,
        ))
    }

    /// 将座位可用性领域对象转换为仅包含可用性数据的数据库活动模型
    ///
    /// # Arguments
    /// * `seat_availability` - 要转换的座位可用性领域对象
    ///
    /// # Returns
    /// 返回仅包含座位可用性数据的ActiveModel
    fn transform_to_do_availability_only(
        seat_availability: &SeatAvailability,
    ) -> crate::models::seat_availability::ActiveModel {
        let mut seat_availability_active_model = crate::models::seat_availability::ActiveModel {
            id: ActiveValue::NotSet,
            train_schedule_id: ActiveValue::Set(
                seat_availability.train_schedule_id().to_db_value(),
            ),
            seat_type_id: ActiveValue::Set(
                seat_availability
                    .seat_type()
                    .get_id()
                    .expect("seat type id should be set")
                    .to_db_value(),
            ),
            begin_station_id: ActiveValue::Set(
                seat_availability
                    .station_range()
                    .get_from_station_id()
                    .to_db_value(),
            ),
            end_station_id: ActiveValue::Set(
                seat_availability
                    .station_range()
                    .get_to_station_id()
                    .to_db_value(),
            ),
            occupied_seats: ActiveValue::Set(seat_availability.occupied_seats_count() as i32),
        };

        if let Some(id) = seat_availability.get_id() {
            seat_availability_active_model.id = ActiveValue::Set(id.to_db_value());
        }

        seat_availability_active_model
    }

    /// 将座位可用性领域对象完整转换为数据库活动模型包
    ///
    /// 包含座位可用性主记录和所有关联的已占用座位记录
    ///
    /// # Arguments
    /// * `seat_availability` - 要转换的座位可用性领域对象
    ///
    /// # Returns
    /// 返回包含所有相关数据的ActiveModel包
    fn transform_to_do(seat_availability: SeatAvailability) -> SeatAvailabilityActiveModelPack {
        let seat_availability_active_model =
            SeatAvailabilityDataConverter::transform_to_do_availability_only(&seat_availability);

        let occupied_seat_active_model_list = seat_availability
            .into_occupied_seat()
            .into_values()
            .map(OccupiedSeatDataConverter::transform_to_do)
            .collect::<Vec<_>>();

        SeatAvailabilityActiveModelPack {
            seat_availability: seat_availability_active_model,
            occupied_seat: occupied_seat_active_model_list,
        }
    }
}

/// 座位可用性仓储实现
///
/// 提供座位可用性聚合根的持久化操作，包括：
/// - 创建、查询、更新、删除座位可用性记录
/// - 管理已占用座位记录的变更
/// - 通过事务保证数据一致性
pub struct SeatAvailabilityRepositoryImpl {
    db: DatabaseConnection,
    aggregate_manager: Arc<Mutex<AggregateManagerImpl<SeatAvailability>>>,
}

impl SeatAvailabilityRepositoryImpl {
    pub fn new(db: DatabaseConnection) -> Self {
        let detect_change_fn = |diff: DiffInfo<SeatAvailability>| {
            let mut result = MultiEntityDiff::new();

            let old = diff.old;
            let new = diff.new;

            let old_occupied_seat_id_map = old
                .iter()
                .flat_map(|item| item.occupied_seat().values())
                .map(|seat| (seat.get_id().unwrap().seat_id().to_db_value(), seat.clone()))
                .collect::<HashMap<_, _>>();

            let new_occupied_seat_id_map = new
                .iter()
                .flat_map(|item| item.occupied_seat().values())
                .map(|seat| (seat.get_id().unwrap().seat_id().to_db_value(), seat.clone()))
                .collect::<HashMap<_, _>>();

            for (seat_id, seat) in &old_occupied_seat_id_map {
                if let Some(new_seat) = new_occupied_seat_id_map.get(seat_id) {
                    if seat != new_seat {
                        result.add_change(TypedDiff::new(
                            DiffType::Modified,
                            Some(seat.clone()),
                            Some(new_seat.clone()),
                        ));
                    }
                } else {
                    result.add_change(TypedDiff::new(DiffType::Removed, Some(seat.clone()), None));
                }
            }

            for (seat_id, seat) in &new_occupied_seat_id_map {
                if !old_occupied_seat_id_map.contains_key(seat_id) {
                    result.add_change(TypedDiff::new(DiffType::Added, None, Some(seat.clone())));
                }
            }

            result
        };
        SeatAvailabilityRepositoryImpl {
            db,
            aggregate_manager: Arc::new(Mutex::new(AggregateManagerImpl::new(Box::new(
                detect_change_fn,
            )))),
        }
    }
}

#[async_trait]
impl DbRepositorySupport<SeatAvailability> for SeatAvailabilityRepositoryImpl {
    type Manager = AggregateManagerImpl<SeatAvailability>;
    fn get_aggregate_manager(&self) -> Arc<Mutex<Self::Manager>> {
        Arc::clone(&self.aggregate_manager)
    }

    async fn on_insert(
        &self,
        aggregate: SeatAvailability,
    ) -> Result<SeatAvailabilityId, RepositoryError> {
        let txn = self
            .db
            .begin()
            .await
            .context("failed to start transaction")?;
        let seat_availability_active_model_pack =
            SeatAvailabilityDataConverter::transform_to_do(aggregate);

        let result = crate::models::seat_availability::Entity::insert(
            seat_availability_active_model_pack.seat_availability,
        )
        .exec(&self.db)
        .await
        .context("failed to insert seat availability")?;

        crate::models::occupied_seat::Entity::insert_many(
            seat_availability_active_model_pack.occupied_seat,
        )
        .exec(&self.db)
        .await
        .context("failed to insert occupied seat for seat availability")?;

        txn.commit().await.context("failed to commit transaction")?;

        Ok(SeatAvailabilityId::from_db_value(result.last_insert_id)?)
    }

    async fn on_select(
        &self,
        id: SeatAvailabilityId,
    ) -> Result<Option<SeatAvailability>, RepositoryError> {
        let seat_availability_do =
            crate::models::seat_availability::Entity::find_by_id(id.to_db_value())
                .one(&self.db)
                .await
                .context(format!("failed to find seat availability with id: {}", id))?;
        let result = match seat_availability_do {
            Some(seat_availability_do) => {
                let train_schedule_do = crate::models::train_schedule::Entity::find_by_id(
                    seat_availability_do.train_schedule_id,
                )
                .one(&self.db)
                .await
                .context(format!(
                    "failed to find train schedule with id: {}",
                    seat_availability_do.train_schedule_id
                ))?
                .ok_or(RepositoryError::InconsistentState(anyhow!(
                    "Inconsistent: cannot find train schedule with id: {}",
                    seat_availability_do.train_schedule_id
                )))?;

                let occupied_seat_do_list = crate::models::occupied_seat::Entity::find()
                    .filter(
                        crate::models::occupied_seat::Column::SeatAvailabilityId
                            .eq(seat_availability_do.id),
                    )
                    .all(&self.db)
                    .await
                    .context("failed to find occupied seat")?;

                let seat_type_do_list = crate::models::seat_type::Entity::find()
                    .all(&self.db)
                    .await
                    .context("failed to find seat type")?;

                let seat_type_mapping_do_list = crate::models::seat_type_mapping::Entity::find()
                    .filter(
                        crate::models::seat_type_mapping::Column::TrainTypeId
                            .eq(train_schedule_do.train_id),
                    )
                    .all(&self.db)
                    .await
                    .context("failed to find seat type mapping")?;

                let seat_type_map = seat_type_do_list
                    .into_iter()
                    .map(|item| (item.id, item))
                    .collect::<HashMap<_, _>>();

                let mut seat_type_mapping_map: HashMap<
                    i32,
                    HashMap<i64, crate::models::seat_type_mapping::Model>,
                > = HashMap::new();

                for item in seat_type_mapping_do_list {
                    seat_type_mapping_map
                        .entry(item.seat_type_id)
                        .or_default()
                        .insert(item.seat_id, item);
                }

                Some(SeatAvailabilityDataConverter::make_from_do(
                    SeatAvailabilityDoPack {
                        seat_availability: seat_availability_do,
                        occupied_seat: occupied_seat_do_list,
                        seat_type: seat_type_map,
                        seat_type_mapping: seat_type_mapping_map,
                    },
                ))
            }
            None => None,
        };

        result.transpose().map_err(RepositoryError::Db)
    }

    async fn on_update(&self, diff: MultiEntityDiff) -> Result<(), RepositoryError> {
        let txn = self
            .db
            .begin()
            .await
            .context("failed to start transaction")?;

        for changes in diff.get_changes::<SeatAvailability>() {
            match changes.diff_type {
                DiffType::Unchanged => {}
                DiffType::Added => {
                    let new = changes.new_value.unwrap();

                    let id = new.get_id();

                    crate::models::seat_availability::Entity::insert(
                        SeatAvailabilityDataConverter::transform_to_do_availability_only(&new),
                    )
                    .exec(&self.db)
                    .await
                    .context(format!("failed to add seat availability with id: {:?}", id))
                    .map_err(RepositoryError::Db)?;
                }
                DiffType::Modified => {
                    let new = changes.new_value.unwrap();

                    let id = new.get_id();

                    crate::models::seat_availability::Entity::update(
                        SeatAvailabilityDataConverter::transform_to_do_availability_only(&new),
                    )
                    .exec(&self.db)
                    .await
                    .context(format!(
                        "failed to update seat availability with id: {:?}",
                        id
                    ))
                    .map_err(RepositoryError::Db)?;
                }
                DiffType::Removed => {
                    if let Some(id) = changes.old_value.unwrap().get_id() {
                        crate::models::seat_availability::Entity::delete_by_id(id.to_db_value())
                            .exec(&self.db)
                            .await
                            .context(format!(
                                "failed to delete seat availability with id: {:?}",
                                id
                            ))
                            .map_err(RepositoryError::Db)?;
                    }
                }
            }
        }

        for changes in diff.get_changes::<OccupiedSeat>() {
            match changes.diff_type {
                DiffType::Unchanged => {}
                DiffType::Added => {
                    let new = changes.new_value.unwrap();

                    let id = new.get_id();

                    crate::models::occupied_seat::Entity::insert(
                        OccupiedSeatDataConverter::transform_to_do(new),
                    )
                    .exec(&self.db)
                    .await
                    .context(format!("failed to add occupied seat with id: {:?}", id))
                    .map_err(RepositoryError::Db)?;
                }
                DiffType::Modified => {
                    let new = changes.new_value.unwrap();

                    let id = new.get_id();

                    crate::models::occupied_seat::Entity::update(
                        OccupiedSeatDataConverter::transform_to_do(new),
                    )
                    .exec(&self.db)
                    .await
                    .context(format!("failed to add occupied seat with id: {:?}", id))
                    .map_err(RepositoryError::Db)?;
                }
                DiffType::Removed => {
                    if let Some(id) = changes.old_value.unwrap().get_id() {
                        crate::models::occupied_seat::Entity::delete_many()
                            .filter(
                                crate::models::occupied_seat::Column::SeatAvailabilityId
                                    .eq(id.seat_availability_id().to_db_value())
                                    .and(
                                        crate::models::occupied_seat::Column::SeatId
                                            .eq(id.seat_id().to_db_value()),
                                    ),
                            )
                            .exec(&self.db)
                            .await
                            .context(format!("failed to delete occupied seat with id: {:?}", id))
                            .map_err(RepositoryError::Db)?;
                    }
                }
            }
        }

        txn.commit().await.context("failed to commit transaction")?;

        Ok(())
    }

    async fn on_delete(&self, aggregate: SeatAvailability) -> Result<(), RepositoryError> {
        if let Some(id) = aggregate.get_id() {
            crate::models::seat_availability::Entity::delete_by_id(id.to_db_value())
                .exec(&self.db)
                .await
                .map_err(|e| RepositoryError::Db(e.into()))?;
        }

        Ok(())
    }
}

#[async_trait]
impl SeatAvailabilityRepository for SeatAvailabilityRepositoryImpl {
    async fn get_train_schedule_seat_availability_list(
        &self,
        train_schedule_id: TrainScheduleId,
    ) -> Result<Vec<crate::models::seat_availability::Model>, RepositoryError> {
        let r = crate::models::seat_availability::Entity::find()
            .filter(
                crate::models::seat_availability::Column::TrainScheduleId
                    .eq(train_schedule_id.to_db_value()),
            )
            .all(&self.db)
            .await
            .context(format!(
                "failed to get seat availability list for train schedule id: {}",
                train_schedule_id
            ))?;

        Ok(r)
    }

    async fn get_train_schedule_occupied_seat(
        &self,
        train_schedule_id: TrainScheduleId,
    ) -> Result<OccupiedSeatInfoMap, RepositoryError> {
        #[derive(FromQueryResult)]
        struct OccupiedSeatInfo {
            seat_type_id: i32,
            begin_station_id: i32,
            end_station_id: i32,
            seat_id: i64,
        }

        let seat_info_list = OccupiedSeatInfo::find_by_statement(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            r#"SELECT
    "seat_availability"."seat_type_id" AS "seat_type_id",
    "seat_availability"."begin_station_id" AS "begin_station_id",
    "seat_availability"."end_station_id" AS "end_station_id",
    "occupied_seat"."seat_id" AS "seat_id"
FROM "seat_availability"
INNER JOIN "occupied_seat"
    ON "occupied_seat"."seat_availability_id" = "seat_availability"."id"
WHERE "seat_availability"."train_schedule_id" = $1;"#,
            [train_schedule_id.to_db_value().into()],
        ))
        .all(&self.db)
        .await
        .context(format!(
            "failed to get occupied seat info for train schedule id: {}",
            train_schedule_id
        ))?;

        let mut result: OccupiedSeatInfoMap = HashMap::new();

        for seat_info in seat_info_list {
            let entry = result
                .entry(seat_info.seat_type_id)
                .or_default()
                .entry((seat_info.begin_station_id, seat_info.end_station_id))
                .or_default();
            entry.push(seat_info.seat_id);
        }

        Ok(result)
    }
}
