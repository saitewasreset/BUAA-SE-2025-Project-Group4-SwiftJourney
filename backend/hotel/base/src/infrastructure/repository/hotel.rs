use crate::DbId;
use crate::domain::repository::hotel::HotelRepository;
use anyhow::{Context, anyhow};
use async_trait::async_trait;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use sea_orm::{ActiveValue, DatabaseConnection, EntityTrait, TransactionTrait};
use sea_orm::{ColumnTrait, Select};
use sea_orm::{QueryFilter, QuerySelect};
use shared::api::InternalApiError;
use shared::data::HotelData;
use shared::domain::model::city::{City, CityId, CityName, ProvinceName};
use shared::domain::model::hotel::{Hotel, HotelId, HotelRoomType, HotelRoomTypeId};
use shared::domain::model::station::{Station, StationId};
use shared::domain::{AggregateManagerImpl, DiffInfo};
use shared::domain::{
    DbRepositorySupport, Diff, DiffType, Identifiable, MultiEntityDiff, RepositoryError, TypedDiff,
};
use shared::internal::object_storage::command::PutObjectCommand;
use shared::internal::object_storage::dto::ObjectCategory;
use shared::ports::geo::GeoPort;
use shared::ports::object_storage::ObjectStoragePort;
use shared::{DB_CHUNK_SIZE, impl_db_id_from_u64};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tracing::{error, instrument, warn};
use uuid::Uuid;

impl_db_id_from_u64!(HotelId, i32, "hotel id");
impl_db_id_from_u64!(HotelRoomTypeId, i32, "hotel room type id");
impl_db_id_from_u64!(CityId, i32, "city id");
impl_db_id_from_u64!(StationId, i32, "station id");

pub struct CityStationPack {
    pub city_entity_map: HashMap<CityId, City>,
    pub station_entity_map: HashMap<StationId, Station>,
}

pub struct HotelDoPack {
    hotel: crate::models::hotel::Model,
    room_type_list: Vec<crate::models::hotel_room_type::Model>,
}

pub struct HotelRoomTypeDataConverter;

impl HotelRoomTypeDataConverter {
    pub fn make_from_do(
        room_type_do: crate::models::hotel_room_type::Model,
    ) -> Result<HotelRoomType, anyhow::Error> {
        Ok(HotelRoomType::new(
            Some(HotelRoomTypeId::from_db_value(room_type_do.id)?),
            Some(HotelId::from_db_value(room_type_do.hotel_id)?),
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
            hotel_id: ActiveValue::NotSet,
        };

        if let Some(id) = room_type.get_id() {
            model.id = ActiveValue::Set(id.to_db_value());
        }

        if let Some(hotel_id) = room_type.hotel_id() {
            model.hotel_id = ActiveValue::Set(hotel_id.to_db_value());
        }

        model
    }
}

pub struct HotelDataConverter;

impl HotelDataConverter {
    pub fn make_from_do(
        hotel_do_pack: HotelDoPack,
        city_station_pack: &CityStationPack,
    ) -> Result<Hotel, anyhow::Error> {
        let mut room_type_list = Vec::with_capacity(hotel_do_pack.room_type_list.len());

        for model in hotel_do_pack.room_type_list {
            room_type_list.push(HotelRoomTypeDataConverter::make_from_do(model)?);
        }

        let city = city_station_pack
            .city_entity_map
            .get(&CityId::from(hotel_do_pack.hotel.city_id as u64))
            .cloned()
            .ok_or(RepositoryError::InconsistentState(anyhow!(
                "no city found for city id = {} for hotel id = {}",
                hotel_do_pack.hotel.city_id,
                hotel_do_pack.hotel.id
            )))?;

        let station = city_station_pack
            .station_entity_map
            .get(&StationId::from(hotel_do_pack.hotel.station_id as u64))
            .cloned()
            .ok_or(RepositoryError::InconsistentState(anyhow!(
                "no city found for station id = {} for hotel id = {}",
                hotel_do_pack.hotel.station_id,
                hotel_do_pack.hotel.id
            )))?;

        let phone: Vec<String> =
            serde_json::from_value(hotel_do_pack.hotel.phone).context(format!(
                "failed to parse phone for hotel id: {}",
                hotel_do_pack.hotel.id
            ))?;

        let images: Vec<Uuid> =
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
            uuid: ActiveValue::Set(hotel.uuid()),
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

pub struct HotelRepositoryImpl<GP>
where
    GP: GeoPort + 'static + Send + Sync,
{
    db: DatabaseConnection,
    aggregate_manager: Arc<Mutex<AggregateManagerImpl<Hotel>>>,
    geo_port: Arc<GP>,
}

impl<GP> HotelRepositoryImpl<GP>
where
    GP: GeoPort + 'static + Send + Sync,
{
    pub fn new(db: DatabaseConnection, geo_port: Arc<GP>) -> Self {
        let detect_changes_fn = |diff: DiffInfo<Hotel>| {
            let mut result = MultiEntityDiff::new();

            let old = diff.old;
            let new = diff.new;

            if let (Some(old), Some(new)) = (old, new) {
                if !(old.name() == new.name()
                    && old.uuid() == new.uuid()
                    && old.city() == new.city()
                    && old.station() == new.station()
                    && old.address() == new.address()
                    && old.phone() == new.phone()
                    && old.images() == new.images()
                    && old.total_booking_count() == new.total_booking_count()
                    && old.total_rating_count() == new.total_rating_count()
                    && old.info() == new.info())
                {
                    result.add_change(TypedDiff::new(
                        DiffType::Modified,
                        Some(old.clone()),
                        Some(new.clone()),
                    ));
                }

                let old_room_type_id_to_room_type = old
                    .room_type_list()
                    .iter()
                    .map(|v| (v.get_id().unwrap(), v.clone()))
                    .collect::<HashMap<_, _>>();

                let new_room_type_id_to_room_type = new
                    .room_type_list()
                    .iter()
                    .map(|v| (v.get_id().unwrap(), v.clone()))
                    .collect::<HashMap<_, _>>();

                for (room_type_id, old_data) in &old_room_type_id_to_room_type {
                    if let Some(new_data) = new_room_type_id_to_room_type.get(room_type_id) {
                        if old_data != new_data {
                            result.add_change(TypedDiff::new(
                                DiffType::Modified,
                                Some(old_data.clone()),
                                Some(new_data.clone()),
                            ));
                        }
                    } else {
                        result.add_change(TypedDiff::new(
                            DiffType::Removed,
                            Some(old_data.clone()),
                            None,
                        ));
                    }
                }

                for (room_type_id, new_data) in &new_room_type_id_to_room_type {
                    if !old_room_type_id_to_room_type.contains_key(room_type_id) {
                        result.add_change(TypedDiff::new(
                            DiffType::Added,
                            None,
                            Some(new_data.clone()),
                        ));
                    }
                }
            }

            result
        };

        Self {
            db,
            aggregate_manager: Arc::new(Mutex::new(AggregateManagerImpl::new(Box::new(
                detect_changes_fn,
            )))),
            geo_port,
        }
    }

    async fn internal_build_city_station_pack(&self) -> Result<CityStationPack, InternalApiError> {
        let db_city_list = self.geo_port.db_get_cities().await?;

        let db_station_list = self.geo_port.db_get_stations().await?;

        let city_entity_map = db_city_list
            .into_iter()
            .map(|x| {
                (
                    CityId::from(x.id as u64),
                    City::new(
                        Some(CityId::from(x.id as u64)),
                        CityName::from(x.name.clone()),
                        ProvinceName::from(x.province.clone()),
                    ),
                )
            })
            .collect::<HashMap<_, _>>();

        let station_entity_map = db_station_list
            .into_iter()
            .map(|x| {
                (
                    StationId::from(x.id as u64),
                    Station::new(
                        Some(StationId::from(x.id as u64)),
                        x.name,
                        CityId::from(x.city_id as u64),
                    ),
                )
            })
            .collect::<HashMap<_, _>>();

        Ok(CityStationPack {
            city_entity_map,
            station_entity_map,
        })
    }

    pub async fn query_hotel_eagerly(
        &self,
        builder: impl FnOnce(
            Select<crate::models::hotel::Entity>,
        ) -> Select<crate::models::hotel::Entity>,
    ) -> Result<Vec<Hotel>, RepositoryError> {
        let city_station_pack = self
            .internal_build_city_station_pack()
            .await
            .inspect_err(|e| error!("Failed to build city station pack {:?}", e))
            .map_err(|e| RepositoryError::Db(e.into()))?;

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
            .all(&self.db)
            .await
            .context("Failed to load hotel")?;

        let mut result = Vec::with_capacity(r.len());

        for hotel_model in r {
            let room_type_list = room_type_list_by_hotel_id
                .get(&hotel_model.id)
                .cloned()
                .ok_or(RepositoryError::InconsistentState(anyhow!(
                    "Room types not found for hotel id: {}",
                    hotel_model.id
                )))?;

            let hotel_do_pack = HotelDoPack {
                hotel: hotel_model,
                room_type_list,
            };

            result.push(HotelDataConverter::make_from_do(
                hotel_do_pack,
                &city_station_pack,
            )?);
        }

        Ok(result)
    }
}

#[async_trait]
impl<GP> DbRepositorySupport<Hotel> for HotelRepositoryImpl<GP>
where
    GP: GeoPort + 'static + Send + Sync,
{
    type Manager = AggregateManagerImpl<Hotel>;
    fn get_aggregate_manager(&self) -> Arc<Mutex<Self::Manager>> {
        Arc::clone(&self.aggregate_manager)
    }

    #[instrument(skip(self))]
    async fn on_insert(&self, aggregate: Hotel) -> Result<HotelId, RepositoryError> {
        let hotel_do = HotelDataConverter::transform_to_do(&aggregate);

        let result = crate::models::hotel::Entity::insert(hotel_do)
            .exec(&self.db)
            .await
            .inspect_err(|e| error!("Failed to insert hotel: {}", e))
            .context("Failed to insert hotel")?;

        let hotel_id = HotelId::from_db_value(result.last_insert_id)?;

        let mut hotel_room_type_list = aggregate.room_type_list().clone();

        for hotel_room_type in &mut hotel_room_type_list {
            hotel_room_type.set_hotel_id(hotel_id);
        }

        let hotel_room_type_do_list = hotel_room_type_list
            .iter()
            .map(HotelRoomTypeDataConverter::transform_to_do)
            .collect::<Vec<_>>();

        crate::models::hotel_room_type::Entity::insert_many(hotel_room_type_do_list)
            .exec(&self.db)
            .await
            .inspect_err(|e| error!("Failed to insert hotel room types: {}", e))
            .context("Failed to insert hotel room types")?;

        Ok(hotel_id)
    }

    async fn on_select(&self, id: HotelId) -> Result<Option<Hotel>, RepositoryError> {
        let city_station_pack = self
            .internal_build_city_station_pack()
            .await
            .inspect_err(|e| error!("Failed to build city station pack {:?}", e))
            .map_err(|e| RepositoryError::Db(e.into()))?;

        let r = crate::models::hotel::Entity::find_by_id(id.to_db_value())
            .one(&self.db)
            .await
            .context(format!("Failed to find hotel for id: {}", id.to_db_value()))?;

        if let Some(hotel_model) = r {
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
                room_type_list,
            };

            let r = HotelDataConverter::make_from_do(hotel_do_pack, &city_station_pack)?;

            Ok(Some(r))
        } else {
            Ok(None)
        }
    }

    async fn on_update(&self, diff: MultiEntityDiff) -> Result<(), RepositoryError> {
        for change in diff.get_changes::<Hotel>() {
            match change.diff_type() {
                DiffType::Unchanged => {}
                DiffType::Added => {
                    unreachable!("Added hotel should be handled in on_insert");
                }
                DiffType::Modified => {
                    let hotel = change.new_value.unwrap();
                    let hotel_do = HotelDataConverter::transform_to_do(&hotel);

                    crate::models::hotel::Entity::update(hotel_do)
                        .filter(
                            crate::models::hotel::Column::Id
                                .eq(hotel.get_id().unwrap().to_db_value()),
                        )
                        .exec(&self.db)
                        .await
                        .context(format!(
                            "Failed to update hotel with id: {}",
                            hotel.get_id().unwrap().to_db_value()
                        ))?;
                }
                DiffType::Removed => {
                    unreachable!("Removed hotel should be handled in on_delete");
                }
            }
        }

        for change in diff.get_changes::<HotelRoomType>() {
            match change.diff_type() {
                DiffType::Unchanged => {}
                DiffType::Added => {
                    let room_type = change.new_value.unwrap();
                    let room_type_do = HotelRoomTypeDataConverter::transform_to_do(&room_type);

                    let room_type_id = room_type.get_id().expect("room type should have id");

                    crate::models::hotel_room_type::Entity::insert(room_type_do)
                        .exec(&self.db)
                        .await
                        .context(format!(
                            "Failed to insert hotel room type with id: {}",
                            room_type_id.to_db_value()
                        ))?;
                }
                DiffType::Modified => {
                    let room_type = change.new_value.unwrap();
                    let room_type_do = HotelRoomTypeDataConverter::transform_to_do(&room_type);

                    let room_type_id = room_type.get_id().expect("room type should have id");

                    crate::models::hotel_room_type::Entity::update(room_type_do)
                        .filter(
                            crate::models::hotel_room_type::Column::Id
                                .eq(room_type_id.to_db_value()),
                        )
                        .exec(&self.db)
                        .await
                        .context(format!(
                            "Failed to update hotel room type with id: {}",
                            room_type_id.to_db_value()
                        ))?;
                }
                DiffType::Removed => {
                    let room_type = change.old_value.unwrap();

                    let room_type_id = room_type.get_id().expect("room type should have id");

                    crate::models::hotel_room_type::Entity::delete_by_id(
                        room_type_id.to_db_value(),
                    )
                    .exec(&self.db)
                    .await
                    .context(format!(
                        "Failed to update hotel room type with id: {}",
                        room_type_id.to_db_value()
                    ))?;
                }
            }
        }

        Ok(())
    }

    async fn on_delete(&self, aggregate: Hotel) -> Result<(), RepositoryError> {
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
}

#[async_trait]
impl<GP> HotelRepository for HotelRepositoryImpl<GP>
where
    GP: GeoPort + 'static + Send + Sync,
{
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
            .map(HotelId::from_db_value)
            .transpose()
            .map_err(RepositoryError::ValidationError)
    }

    async fn find_by_uuid(&self, uuid: Uuid) -> Result<Option<Hotel>, RepositoryError> {
        self.query_hotel_eagerly(|q| q.filter(crate::models::hotel::Column::Uuid.eq(uuid)))
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

#[instrument(skip_all)]
pub async fn save_raw_hotel<OP: ObjectStoragePort, GP: GeoPort>(
    object_storage_port: Arc<OP>,
    geo_port: Arc<GP>,
    db: &DatabaseConnection,
    data_base_path: &Path,
    hotel_data: HotelData,
) -> Result<(), RepositoryError> {
    let db_city_list = geo_port
        .db_get_cities()
        .await
        .inspect_err(|e| error!("Failed to get db cities: {:?}", e))
        .map_err(|e| RepositoryError::Db(e.into()))?;

    let db_station_list = geo_port
        .db_get_stations()
        .await
        .inspect_err(|e| error!("Failed to get db stations: {:?}", e))
        .map_err(|e| RepositoryError::Db(e.into()))?;

    let tx = db.begin().await.context("Failed to begin transaction")?;

    let city_name_to_city = db_city_list
        .into_iter()
        .map(|c| (c.name.clone(), c))
        .collect::<HashMap<_, _>>();

    let station_name_to_station = db_station_list
        .into_iter()
        .map(|s| (s.name.clone(), s))
        .collect::<HashMap<_, _>>();

    let mut images_cache: HashMap<PathBuf, Uuid> = HashMap::new();

    let mut hotel_do_list = Vec::new();
    let mut hotel_uuid_to_room_type: HashMap<
        Uuid,
        Vec<crate::models::hotel_room_type::ActiveModel>,
    > = HashMap::new();

    for hotel_info in hotel_data {
        let city = city_name_to_city
            .get(&hotel_info.city)
            .cloned()
            .ok_or(RepositoryError::InconsistentState(anyhow!(
                "Invalid city: {}",
                &hotel_info.city
            )))
            .inspect_err(|e| {
                error!("Invalid city: {}: {}", &hotel_info.city, e);
            })?;

        if let Some(station) = hotel_info.station {
            let station = station_name_to_station
                .get(&station)
                .cloned()
                .ok_or(RepositoryError::InconsistentState(anyhow!(
                    "Invalid station: {}",
                    &station
                )))
                .inspect_err(|e| {
                    error!("Invalid station: {}: {}", &station, e);
                })?;

            let mut hotel = Hotel::new(
                hotel_info.name,
                City::new(
                    Some(CityId::from(city.id as u64)),
                    CityName::from(city.name),
                    ProvinceName::from(city.province),
                ),
                Station::new(
                    Some(StationId::from(station.id as u64)),
                    station.name,
                    CityId::from(station.city_id as u64),
                ),
                hotel_info.address,
                hotel_info.info,
            );

            for phone in hotel_info.phone {
                hotel.add_phone(phone);
            }

            for image in hotel_info.images {
                let image_path = data_base_path.join(image);

                let image_uuid = if let Some(uuid) = images_cache.get(&image_path) {
                    *uuid
                } else {
                    let image_data = fs::read(&image_path)
                        .context(format!("cannot read from: {:?}", &image_path))
                        .inspect_err(|e| {
                            error!("failed load hotel image: {}", e);
                        })?;

                    let put_object_command = PutObjectCommand {
                        object_category: ObjectCategory::Hotel,
                        content_type: "image/jpeg".to_string(),
                        object: image_data,
                    };

                    let uuid = object_storage_port
                        .put_object(put_object_command)
                        .await
                        .map_err(|e| {
                            error!("failed save image: {:?}", e);

                            RepositoryError::Db(e.into())
                        })?;

                    images_cache.insert(image_path, uuid);

                    uuid
                };

                hotel.add_image(image_uuid);
            }

            for (room_type_name, room_type_info) in hotel_info.room_info {
                let price = Decimal::from_f64(room_type_info.price).unwrap();
                let hotel_room_type =
                    HotelRoomType::new(None, None, room_type_name, room_type_info.capacity, price);

                hotel.add_room_type(hotel_room_type);
            }

            hotel_do_list.push(HotelDataConverter::transform_to_do(&hotel));
            hotel_uuid_to_room_type.insert(
                hotel.uuid(),
                hotel
                    .room_type_list()
                    .iter()
                    .map(HotelRoomTypeDataConverter::transform_to_do)
                    .collect::<Vec<_>>(),
            );

            // 由于评论需要与用户关联，无法在此处加载评论
        } else {
            warn!("Skipping hotel info without a station: {}", hotel_info.name);
        }
    }

    for hotel_do_part in hotel_do_list.chunks(DB_CHUNK_SIZE) {
        crate::models::hotel::Entity::insert_many(hotel_do_part.to_vec())
            .exec(&tx)
            .await
            .inspect_err(|e| {
                error!("Failed to insert hotel: {}", e);
            })
            .context("Failed to insert hotel")?;
    }

    let hotel_id_uuid_list: Vec<(i32, Uuid)> = crate::models::hotel::Entity::find()
        .select_only()
        .columns([
            crate::models::hotel::Column::Id,
            crate::models::hotel::Column::Uuid,
        ])
        .into_tuple()
        .all(&tx)
        .await
        .inspect_err(|e| {
            error!("Failed to load hotel: {}", e);
        })
        .context("Failed to load hotel")?;

    let hotel_uuid_map = hotel_id_uuid_list
        .into_iter()
        .map(|(id, uuid)| (uuid, id))
        .collect::<HashMap<_, _>>();

    for (hotel_uuid, hotel_room_type_list) in &mut hotel_uuid_to_room_type {
        let hotel_id = *hotel_uuid_map.get(hotel_uuid).unwrap();

        hotel_room_type_list.iter_mut().for_each(|x| {
            x.hotel_id = ActiveValue::Set(hotel_id);
        })
    }

    let hotel_room_type_do_list = hotel_uuid_to_room_type
        .into_values()
        .flatten()
        .collect::<Vec<_>>();

    for hotel_room_type_do_part in hotel_room_type_do_list.chunks(DB_CHUNK_SIZE) {
        crate::models::hotel_room_type::Entity::insert_many(hotel_room_type_do_part.to_vec())
            .exec(&tx)
            .await
            .inspect_err(|e| {
                error!("Failed to insert hotel room type: {}", e);
            })
            .context("Failed to insert hotel room type")?;
    }

    tx.commit().await.context("Failed to commit transaction")?;

    Ok(())
}
