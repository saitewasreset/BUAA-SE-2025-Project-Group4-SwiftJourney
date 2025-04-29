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

pub struct TrainScheduleDataConverter;
pub struct OccupiedSeatConverter;

struct TrainScheduleDoPack {
    train_schedule: crate::models::train_schedule::Model,
    occupied_seat: Vec<crate::models::occupied_seat::Model>,
    seat_type: HashMap<i32, crate::models::seat_type::Model>,
    // seat_type_id -> seat_id -> seat_type_mapping
    seat_type_mapping: HashMap<i32, HashMap<i64, crate::models::seat_type_mapping::Model>>,
}

impl TrainScheduleDataConverter {
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

pub struct TrainScheduleRepositoryImpl {
    db: DatabaseConnection,
    aggregate_manager: Arc<Mutex<AggregateManagerImpl<TrainSchedule>>>,
}

impl TrainScheduleRepositoryImpl {
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
    fn get_aggregate_manager(&self) -> Arc<Mutex<Self::Manager>> {
        Arc::clone(&self.aggregate_manager)
    }

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
    async fn find_by_date(&self, date: NaiveDate) -> Result<Vec<TrainSchedule>, RepositoryError> {
        self.query_train_schedules_eagerly(|q| {
            q.filter(crate::models::train_schedule::Column::DepartureDate.eq(date))
        })
        .await
    }

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
