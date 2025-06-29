use crate::domain::model::hotel::{
    HotelDateRange, HotelId, HotelRoomTypeId, OccupiedRoom, OccupiedRoomId,
};
use crate::domain::model::personal_info::PersonalInfoId;
use crate::domain::repository::occupied_room::OccupiedRoomRepository;
use crate::domain::{DbId, Identifiable, Repository, RepositoryError};
use anyhow::Context;
use async_trait::async_trait;
use sea_orm::{ActiveValue, DatabaseBackend, DatabaseConnection, EntityTrait, Statement};
use sea_orm::{ColumnTrait, Select};
use sea_orm::{QueryFilter, TransactionTrait};
use uuid::Uuid;

impl_db_id_from_u64!(OccupiedRoomId, i32, "occupied room");

pub struct OccupiedRoomDataConverter {}

impl OccupiedRoomDataConverter {
    pub fn make_from_do(
        occupied_room_do: crate::models::occupied_room::Model,
    ) -> Result<OccupiedRoom, anyhow::Error> {
        let occupied_room_id = OccupiedRoomId::from_db_value(occupied_room_do.id)?;
        let hotel_id = HotelId::from_db_value(occupied_room_do.hotel_id)?;
        let hotel_room_type_id = HotelRoomTypeId::from_db_value(occupied_room_do.room_type_id)?;

        let hotel_date_range =
            HotelDateRange::new(occupied_room_do.begin_date, occupied_room_do.end_date)
                .context("Invalid date range")?;

        let personal_info_id = PersonalInfoId::from_db_value(occupied_room_do.person_info_id)?;

        Ok(OccupiedRoom::new(
            Some(occupied_room_id),
            hotel_id,
            hotel_room_type_id,
            hotel_date_range,
            personal_info_id,
        ))
    }

    pub fn transform_to_do(
        occupied_room: &OccupiedRoom,
    ) -> crate::models::occupied_room::ActiveModel {
        let mut model = crate::models::occupied_room::ActiveModel {
            id: ActiveValue::NotSet,
            hotel_id: ActiveValue::Set(occupied_room.hotel_id().to_db_value()),
            room_type_id: ActiveValue::Set(occupied_room.hotel_room_type_id().to_db_value()),
            begin_date: ActiveValue::Set(occupied_room.booking_date_range().begin_date()),
            end_date: ActiveValue::Set(occupied_room.booking_date_range().end_date()),
            person_info_id: ActiveValue::Set(occupied_room.personal_info().to_db_value()),
        };

        if let Some(id) = occupied_room.get_id() {
            model.id = ActiveValue::Set(id.to_db_value());
        }

        model
    }
}

pub struct OccupiedRoomRepositoryImpl {
    db: DatabaseConnection,
}

impl OccupiedRoomRepositoryImpl {
    pub async fn query_occupied_room(
        &self,
        builder: impl FnOnce(
            Select<crate::models::occupied_room::Entity>,
        ) -> Select<crate::models::occupied_room::Entity>,
    ) -> Result<Vec<OccupiedRoom>, RepositoryError> {
        let model_list = builder(crate::models::occupied_room::Entity::find())
            .all(&self.db)
            .await
            .context("Failed to query occupied room")?;

        let mut result = Vec::with_capacity(model_list.len());

        for model in model_list {
            result.push(
                OccupiedRoomDataConverter::make_from_do(model)
                    .map_err(RepositoryError::ValidationError)?,
            );
        }

        Ok(result)
    }

    pub fn new(db: DatabaseConnection) -> Self {
        OccupiedRoomRepositoryImpl { db }
    }
}

#[async_trait]
impl Repository<OccupiedRoom> for OccupiedRoomRepositoryImpl {
    async fn find(&self, id: OccupiedRoomId) -> Result<Option<OccupiedRoom>, RepositoryError> {
        crate::models::occupied_room::Entity::find_by_id(id.to_db_value())
            .one(&self.db)
            .await
            .context(format!("Failed to find occupied room with id: {}", id))?
            .map(|occupied_room_do| {
                OccupiedRoomDataConverter::make_from_do(occupied_room_do)
                    .map_err(RepositoryError::ValidationError)
            })
            .transpose()
    }

    async fn remove(&self, aggregate: OccupiedRoom) -> Result<(), RepositoryError> {
        if let Some(id) = aggregate.get_id() {
            crate::models::occupied_room::Entity::delete_by_id(id.to_db_value())
                .exec(&self.db)
                .await
                .context(format!("Failed to delete occupied room with id: {}", id))?;
        }

        Ok(())
    }

    async fn save(&self, aggregate: &mut OccupiedRoom) -> Result<OccupiedRoomId, RepositoryError> {
        if let Some(id) = aggregate.get_id() {
            let model = OccupiedRoomDataConverter::transform_to_do(aggregate);

            crate::models::occupied_room::Entity::update(model)
                .filter(crate::models::occupied_room::Column::Id.eq(id.to_db_value()))
                .exec(&self.db)
                .await
                .context(format!("Failed to update occupied room with id: {}", id))?;
            Ok(id)
        } else {
            let model = OccupiedRoomDataConverter::transform_to_do(aggregate);

            let result = crate::models::occupied_room::Entity::insert(model)
                .exec(&self.db)
                .await
                .context("Failed to insert new occupied room")?;

            let new_id = OccupiedRoomId::from_db_value(result.last_insert_id)?;

            aggregate.set_id(new_id);
            Ok(new_id)
        }
    }
}

#[async_trait]
impl OccupiedRoomRepository for OccupiedRoomRepositoryImpl {
    async fn find_by_date_range(
        &self,
        hotel_id: HotelId,
        booking_date_range: HotelDateRange,
    ) -> Result<Vec<OccupiedRoom>, RepositoryError> {
        self.query_occupied_room(|q| {
            q.filter(crate::models::occupied_room::Column::HotelId.eq(hotel_id.to_db_value()))
                .filter(
                    crate::models::occupied_room::Column::BeginDate
                        .gte(booking_date_range.begin_date()),
                )
                .filter(
                    crate::models::occupied_room::Column::EndDate
                        .lte(booking_date_range.end_date()),
                )
        })
        .await
    }

    async fn find_possible_occupied_range(
        &self,
        hotel_id: HotelId,
        booking_date_range: HotelDateRange,
    ) -> Result<Vec<OccupiedRoom>, RepositoryError> {
        self.query_occupied_room(|q| {
            q.filter(crate::models::occupied_room::Column::HotelId.eq(hotel_id.to_db_value()))
                .filter(
                    crate::models::occupied_room::Column::BeginDate
                        .lt(booking_date_range.end_date()),
                )
                .filter(
                    crate::models::occupied_room::Column::EndDate
                        .gt(booking_date_range.begin_date()),
                )
        })
        .await
    }

    async fn save_count(
        &self,
        occupied_room: &OccupiedRoom,
        count: i32,
    ) -> Result<(), RepositoryError> {
        let model = OccupiedRoomDataConverter::transform_to_do(occupied_room);

        let tx = self
            .db
            .begin()
            .await
            .context("Failed to start transaction")?;

        for _ in 0..count {
            crate::models::occupied_room::Entity::insert(model.clone())
                .exec(&tx)
                .await
                .context("Failed to insert new occupied room")?;
        }

        tx.commit().await.context("Failed to commit transaction")?;

        Ok(())
    }

    async fn find_by_order_uuid(
        &self,
        order_uuid: Uuid,
    ) -> Result<Vec<OccupiedRoom>, RepositoryError> {
        let occupied_do_list = crate::models::occupied_room::Entity::find()
            .from_raw_sql(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                r#"SELECT
    "occupied_room"."id",
    "occupied_room"."hotel_id",
    "occupied_room"."room_type_id",
    "occupied_room"."begin_date",
    "occupied_room"."end_date",
    "occupied_room"."person_info_id"
FROM "occupied_room" 
    INNER JOIN "hotel_order"
        ON "occupied_room"."hotel_id" = "hotel_order"."hotel_id"
               AND "occupied_room"."room_type_id" = "hotel_order"."hotel_room_type_id"
               AND "occupied_room"."begin_date" = "hotel_order"."begin_date"
               AND "occupied_room"."end_date" = "hotel_order"."end_date"
WHERE
    "hotel_order"."uuid" = $1"#,
                [order_uuid.into()],
            ))
            .all(&self.db)
            .await
            .context(format!(
                "Failed to find occupied room by order uuid: {}",
                order_uuid
            ))?;

        let mut result = Vec::with_capacity(occupied_do_list.len());

        for occupied_room_do in occupied_do_list {
            result.push(
                OccupiedRoomDataConverter::make_from_do(occupied_room_do)
                    .map_err(RepositoryError::ValidationError)?,
            );
        }

        Ok(result)
    }

    async fn remove_many(
        &self,
        occupied_room_list: Vec<OccupiedRoom>,
    ) -> Result<(), RepositoryError> {
        let tx = self
            .db
            .begin()
            .await
            .context("Failed to begin transaction")?;

        for occupied_room in occupied_room_list {
            if let Some(id) = occupied_room.get_id() {
                crate::models::occupied_room::Entity::delete_by_id(id.to_db_value())
                    .exec(&tx)
                    .await
                    .context(format!("Failed to delete occupied room id: {}", id))?;
            }
        }

        tx.commit().await.context("Failed to commit transaction")?;

        Ok(())
    }
}
