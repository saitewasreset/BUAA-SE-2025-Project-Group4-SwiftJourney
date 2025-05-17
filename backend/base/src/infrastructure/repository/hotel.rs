use crate::domain::model::city::CityId;
use crate::domain::model::hotel::{Hotel, HotelId, HotelRoomType, HotelRoomTypeId};
use crate::domain::model::station::StationId;
use crate::domain::repository::hotel::HotelRepository;
use crate::domain::{DbId, Identifiable, Repository, RepositoryError};
use crate::infrastructure::repository::city::CityDataConverter;
use crate::infrastructure::repository::station::StationDataConverter;
use anyhow::{Context, anyhow};
use async_trait::async_trait;
use sea_orm::{ActiveValue, DatabaseConnection, EntityTrait};
use sea_orm::{ColumnTrait, Select};
use sea_orm::{QueryFilter, QuerySelect};
use std::collections::HashMap;
use std::result;
use uuid::Uuid;

impl_db_id_from_u64!(HotelId, i32, "hotel id");
impl_db_id_from_u64!(HotelRoomTypeId, i32, "hotel room type id");

pub struct HotelDoPack {
    hotel: crate::models::hotel::Model,
    city: crate::models::city::Model,
    station: crate::models::station::Model,
    room_type_list: Vec<crate::models::hotel_room_type::Model>,
}

pub struct HotelRoomTypeDataConverter;

impl HotelRoomTypeDataConverter {
    pub fn make_from_do(
        room_type_do: crate::models::hotel_room_type::Model,
    ) -> Result<HotelRoomType, anyhow::Error> {
        Ok(HotelRoomType::new(
            Some(HotelRoomTypeId::from_db_value(room_type_do.id)?),
            HotelId::from_db_value(room_type_do.hotel_id)?,
            room_type_do.type_name,
            room_type_do.capacity,
            room_type_do.price,
        ))
    }

    pub fn transform_to_do(
        room_type: &HotelRoomType,
    ) -> crate::models::hotel_room_type::ActiveModel {
        let mut model = crate::models::hotel_room_type::ActiveModel {
            id: ActiveValue::NotSet,
            type_name: ActiveValue::Set(room_type.type_name().clone()),
            capacity: ActiveValue::Set(room_type.capacity()),
            price: ActiveValue::Set(room_type.price()),
            hotel_id: ActiveValue::Set(room_type.hotel_id().to_db_value()),
        };

        if let Some(id) = room_type.get_id() {
            model.id = ActiveValue::Set(id.to_db_value());
        }

        model
    }
}

pub struct HotelDataConverter;

impl HotelDataConverter {
    pub fn make_from_do(hotel_do_pack: HotelDoPack) -> Result<Hotel, anyhow::Error> {
        let mut room_type_list = Vec::with_capacity(hotel_do_pack.room_type_list.len());

        for model in hotel_do_pack.room_type_list {
            room_type_list.push(HotelRoomTypeDataConverter::make_from_do(model)?);
        }

        let city = CityDataConverter::make_from_do(hotel_do_pack.city)?;

        let station = StationDataConverter::make_from_do(hotel_do_pack.station)?;

        let phone: Vec<String> =
            serde_json::from_value(hotel_do_pack.hotel.phone).context(format!(
                "failed to parse phone for hotel id: {}",
                hotel_do_pack.hotel.id
            ))?;

        let images: Vec<String> =
            serde_json::from_value(hotel_do_pack.hotel.images).context(format!(
                "failed to parse images for hotel id: {}",
                hotel_do_pack.hotel.id
            ))?;

        Ok(Hotel::new_full_unchecked(
            Some(HotelId::from_db_value(hotel_do_pack.hotel.id)?),
            hotel_do_pack.hotel.uuid,
            hotel_do_pack.hotel.name,
            city,
            station,
            hotel_do_pack.hotel.address,
            phone,
            images,
            hotel_do_pack.hotel.total_rating_count,
            hotel_do_pack.hotel.total_booking_count,
            room_type_list,
            hotel_do_pack.hotel.info,
        ))
    }

    pub fn transform_to_do(hotel: &Hotel) -> crate::models::hotel::ActiveModel {
        let mut model = crate::models::hotel::ActiveModel {
            id: ActiveValue::NotSet,
            uuid: ActiveValue::Set(hotel.uuid().clone()),
            name: ActiveValue::Set(hotel.name().to_string()),
            city_id: ActiveValue::Set(
                hotel
                    .city()
                    .get_id()
                    .expect("city should have id")
                    .to_db_value(),
            ),
            station_id: ActiveValue::Set(
                hotel
                    .station()
                    .get_id()
                    .expect("station should have id")
                    .to_db_value(),
            ),
            address: ActiveValue::Set(hotel.address().clone()),
            phone: ActiveValue::Set(serde_json::to_value(hotel.phone()).unwrap()),
            images: ActiveValue::Set(serde_json::to_value(hotel.images()).unwrap()),
            total_rating_count: ActiveValue::Set(hotel.total_rating_count()),
            total_booking_count: ActiveValue::Set(hotel.total_booking_count()),
            info: ActiveValue::Set(hotel.info().clone()),
        };

        if let Some(id) = hotel.get_id() {
            model.id = ActiveValue::Set(id.to_db_value());
        }

        model
    }
}

pub struct HotelRepositoryImpl {
    db: DatabaseConnection,
}

impl HotelRepositoryImpl {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn query_hotel_eagerly(
        &self,
        builder: impl FnOnce(
            Select<crate::models::hotel::Entity>,
        ) -> Select<crate::models::hotel::Entity>,
    ) -> Result<Vec<Hotel>, RepositoryError> {
        let room_type_list = crate::models::hotel_room_type::Entity::find()
            .all(&self.db)
            .await
            .context("Failed to load room types for hotel id")?;

        let mut room_type_list_by_hotel_id: HashMap<
            i32,
            Vec<crate::models::hotel_room_type::Model>,
        > = HashMap::new();

        for room_type in room_type_list {
            room_type_list_by_hotel_id
                .entry(room_type.hotel_id)
                .or_default()
                .push(room_type);
        }

        let r = builder(crate::models::hotel::Entity::find())
            .find_also_related(crate::models::city::Entity)
            .find_also_related(crate::models::station::Entity)
            .all(&self.db)
            .await
            .context("Failed to load hotel")?;

        let mut result = Vec::with_capacity(r.len());

        for (hotel_model, city_model, station_model) in r {
            let city_model = city_model.ok_or(RepositoryError::InconsistentState(anyhow!(
                "City not found for hotel id: {}",
                hotel_model.id
            )))?;

            let station_model = station_model.ok_or(RepositoryError::InconsistentState(
                anyhow!("Station not found for hotel id: {}", hotel_model.id),
            ))?;

            let room_type_list = room_type_list_by_hotel_id
                .get(&hotel_model.id)
                .cloned()
                .ok_or(RepositoryError::InconsistentState(anyhow!(
                    "Room types not found for hotel id: {}",
                    hotel_model.id
                )))?;

            let hotel_do_pack = HotelDoPack {
                hotel: hotel_model,
                city: city_model,
                station: station_model,
                room_type_list,
            };

            result.push(HotelDataConverter::make_from_do(hotel_do_pack)?);
        }

        Ok(result)
    }

    pub async fn query_hotel_lazily(
        &self,
        builder: impl FnOnce(
            Select<crate::models::hotel::Entity>,
        ) -> Select<crate::models::hotel::Entity>,
    ) -> Result<Vec<Hotel>, RepositoryError> {
        let r = builder(crate::models::hotel::Entity::find())
            .find_also_related(crate::models::city::Entity)
            .find_also_related(crate::models::station::Entity)
            .all(&self.db)
            .await
            .context("Failed to load hotel")?;

        let mut result = Vec::with_capacity(r.len());

        for (hotel_model, city_model, station_model) in r {
            let city_model = city_model.ok_or(RepositoryError::InconsistentState(anyhow!(
                "City not found for hotel id: {}",
                hotel_model.id
            )))?;

            let station_model = station_model.ok_or(RepositoryError::InconsistentState(
                anyhow!("Station not found for hotel id: {}", hotel_model.id),
            ))?;

            let room_type_list = crate::models::hotel_room_type::Entity::find()
                .filter(crate::models::hotel_room_type::Column::HotelId.eq(hotel_model.id))
                .all(&self.db)
                .await
                .context("Failed to load room types for hotel id")?;

            let hotel_do_pack = HotelDoPack {
                hotel: hotel_model,
                city: city_model,
                station: station_model,
                room_type_list,
            };

            result.push(HotelDataConverter::make_from_do(hotel_do_pack)?);
        }

        Ok(result)
    }
}

#[async_trait]
impl Repository<Hotel> for HotelRepositoryImpl {
    async fn find(&self, id: HotelId) -> Result<Option<Hotel>, RepositoryError> {
        let r = crate::models::hotel::Entity::find_by_id(id.to_db_value())
            .find_also_related(crate::models::city::Entity)
            .find_also_related(crate::models::station::Entity)
            .one(&self.db)
            .await
            .context(format!("Failed to find hotel for id: {}", id.to_db_value()))?;

        if let Some((hotel_model, city_model, station_model)) = r {
            let city_model = city_model.ok_or(RepositoryError::InconsistentState(anyhow!(
                "City not found for hotel id: {}",
                id.to_db_value()
            )))?;

            let station_model = station_model.ok_or(RepositoryError::InconsistentState(
                anyhow!("Station not found for hotel id: {}", id.to_db_value()),
            ))?;

            let room_type_list = crate::models::hotel_room_type::Entity::find()
                .filter(crate::models::hotel_room_type::Column::HotelId.eq(id.to_db_value()))
                .all(&self.db)
                .await
                .context(format!(
                    "Failed to find room types for hotel id: {}",
                    id.to_db_value()
                ))?;

            let hotel_do_pack = HotelDoPack {
                hotel: hotel_model,
                city: city_model,
                station: station_model,
                room_type_list,
            };

            let r = HotelDataConverter::make_from_do(hotel_do_pack)?;

            Ok(Some(r))
        } else {
            Ok(None)
        }
    }

    async fn remove(&self, aggregate: Hotel) -> Result<(), RepositoryError> {
        if let Some(id) = aggregate.get_id() {
            crate::models::hotel::Entity::delete_by_id(id.to_db_value())
                .exec(&self.db)
                .await
                .context(format!(
                    "Failed to delete hotel with id: {}",
                    id.to_db_value()
                ))?;
        }

        Ok(())
    }

    async fn save(&self, aggregate: &mut Hotel) -> Result<HotelId, RepositoryError> {
        let model = HotelDataConverter::transform_to_do(aggregate);

        if let Some(id) = aggregate.get_id() {
            crate::models::hotel::Entity::update(model)
                .filter(crate::models::hotel::Column::Id.eq(id.to_db_value()))
                .exec(&self.db)
                .await
                .context(format!(
                    "Failed to update hotel with id: {}",
                    id.to_db_value()
                ))?;

            Ok(id)
        } else {
            let r = crate::models::hotel::Entity::insert(model)
                .exec(&self.db)
                .await
                .context("Failed to insert hotel")?;

            Ok(HotelId::from_db_value(r.last_insert_id)?)
        }
    }
}

#[async_trait]
impl HotelRepository for HotelRepositoryImpl {
    async fn get_id_by_uuid(&self, uuid: Uuid) -> Result<Option<HotelId>, RepositoryError> {
        let result: Option<i32> = crate::models::hotel::Entity::find()
            .select_only()
            .column(crate::models::hotel::Column::Id)
            .filter(crate::models::hotel::Column::Uuid.eq(uuid))
            .into_tuple()
            .one(&self.db)
            .await
            .context(format!("Failed to get hotel for uuid: {}", uuid))?;

        result
            .map(|x| HotelId::from_db_value(x))
            .transpose()
            .map_err(RepositoryError::ValidationError)
    }

    async fn find_by_uuid(&self, uuid: Uuid) -> Result<Option<Hotel>, RepositoryError> {
        self.query_hotel_lazily(|q| q.filter(crate::models::hotel::Column::Uuid.eq(uuid)))
            .await
            .map(|x| x.into_iter().next())
    }

    async fn find_by_city(
        &self,
        city_id: CityId,
        name_pattern: Option<&str>,
    ) -> Result<Vec<Hotel>, RepositoryError> {
        if let Some(name_pattern) = name_pattern {
            self.query_hotel_eagerly(|q: Select<crate::models::hotel::Entity>| {
                q.filter(crate::models::hotel::Column::CityId.eq(city_id.to_db_value()))
                    .filter(crate::models::hotel::Column::Name.contains(name_pattern))
            })
            .await
        } else {
            self.query_hotel_eagerly(|q: Select<crate::models::hotel::Entity>| {
                q.filter(crate::models::hotel::Column::CityId.eq(city_id.to_db_value()))
            })
            .await
        }
    }

    async fn find_by_station(
        &self,
        station_id: StationId,
        name_pattern: Option<&str>,
    ) -> Result<Vec<Hotel>, RepositoryError> {
        if let Some(name_pattern) = name_pattern {
            self.query_hotel_eagerly(|q: Select<crate::models::hotel::Entity>| {
                q.filter(crate::models::hotel::Column::StationId.eq(station_id.to_db_value()))
                    .filter(crate::models::hotel::Column::Name.contains(name_pattern))
            })
            .await
        } else {
            self.query_hotel_eagerly(|q: Select<crate::models::hotel::Entity>| {
                q.filter(crate::models::hotel::Column::StationId.eq(station_id.to_db_value()))
            })
            .await
        }
    }
}
