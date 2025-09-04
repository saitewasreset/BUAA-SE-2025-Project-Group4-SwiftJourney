use crate::application::service::internal::DishInternalService;
use crate::domain::repository::dish::DishRepository;
use crate::domain::repository::takeaway::TakeawayShopRepository;
use anyhow::{Context, anyhow};
use async_trait::async_trait;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use sea_orm::{ActiveValue, DatabaseConnection, EntityTrait, TransactionTrait};
use shared::domain::model::station::StationId;
use shared::domain::model::train::TrainId;
use shared::event::queue::EventService;
use shared::event::{DishUpdatedEvent, EventPackage, TakeawayDishUpdatedEvent};
use shared::internal::dish::command::{SaveRawDishCommand, SaveRawTakeawayCommand};
use shared::internal::dish::dto::{DbDishDTO, DbTakeawayDishDTO};
use shared::{
    DB_CHUNK_SIZE, MicroService,
    domain::{
        RepositoryError,
        model::takeaway::{TakeawayDish, TakeawayShop},
    },
    internal::object_storage::{command::PutObjectCommand, dto::ObjectCategory},
    ports::{geo::GeoPort, object_storage::ObjectStoragePort, train::TrainPort},
};
use std::path::PathBuf;
use std::{collections::HashMap, fs, sync::Arc};
use tracing::error;
use uuid::Uuid;

pub struct DishInternalServiceImpl<DR, TSR, TP, GP, OSP, ES>
where
    DR: DishRepository,
    TSR: TakeawayShopRepository,
    TP: TrainPort,
    GP: GeoPort,
    OSP: ObjectStoragePort,
    ES: EventService,
{
    dish_repository: Arc<DR>,
    takeaway_shop_repository: Arc<TSR>,
    train_port: Arc<TP>,
    geo_port: Arc<GP>,
    object_storage_port: Arc<OSP>,
    data_path: PathBuf,
    db: DatabaseConnection,
    event_service: Arc<ES>,
}

impl<DR, TSR, TP, GP, OSP, ES> DishInternalServiceImpl<DR, TSR, TP, GP, OSP, ES>
where
    DR: DishRepository,
    TSR: TakeawayShopRepository,
    TP: TrainPort,
    GP: GeoPort,
    OSP: ObjectStoragePort,
    ES: EventService,
{
    pub fn new(
        dish_repository: Arc<DR>,
        takeaway_shop_repository: Arc<TSR>,
        train_port: Arc<TP>,
        geo_port: Arc<GP>,
        object_storage_port: Arc<OSP>,
        data_path: PathBuf,
        db: DatabaseConnection,
        event_service: Arc<ES>,
    ) -> Self {
        Self {
            dish_repository,
            takeaway_shop_repository,
            train_port,
            geo_port,
            object_storage_port,
            data_path,
            db,
            event_service,
        }
    }
}

#[async_trait]
impl<DR, TSR, TP, GP, OSP, ES> DishInternalService
    for DishInternalServiceImpl<DR, TSR, TP, GP, OSP, ES>
where
    DR: DishRepository,
    TSR: TakeawayShopRepository,
    TP: TrainPort,
    GP: GeoPort,
    OSP: ObjectStoragePort,
    ES: EventService,
{
    async fn save_raw_dish(&self, command: SaveRawDishCommand) -> Result<(), RepositoryError> {
        let tx = self
            .db
            .begin()
            .await
            .inspect_err(|e| {
                error!("failed to begin transaction: {}", e);
            })
            .map_err(|e| RepositoryError::Db(e.into()))?;

        let mut image_path_to_uuid: HashMap<String, Uuid> = HashMap::new();

        let train_list = self
            .train_port
            .get_trains()
            .await
            .inspect_err(|e| {
                error!("failed to get trains: {}", e);
            })
            .map_err(|e| RepositoryError::Db(e.into()))?;

        let train_number_str_to_id = train_list
            .into_iter()
            .map(|train| (train.number, TrainId::from(train.id)))
            .collect::<HashMap<_, _>>();

        let mut dish_model_list = Vec::new();

        for (train_number_str, dish_list) in command.dish {
            let train_id = *train_number_str_to_id
                .get(&train_number_str)
                .ok_or_else(|| {
                    RepositoryError::InconsistentState(anyhow!(
                        "train number {} not found in database",
                        train_number_str
                    ))
                })?;

            for dish in dish_list {
                let image_uuid = if let Some(uuid) = image_path_to_uuid.get(&dish.picture) {
                    *uuid
                } else {
                    let image_path = self.data_path.join(&dish.picture);

                    let image_data = fs::read(&image_path)
                        .context(format!("cannot read from: {:?}", &image_path))
                        .inspect_err(|e| {
                            error!("failed load dish image: {}", e);
                        })?;

                    let uuid = self
                        .object_storage_port
                        .put_object(PutObjectCommand {
                            object_category: ObjectCategory::Dish,
                            content_type: "image/jpeg".to_owned(),
                            object: image_data,
                        })
                        .await
                        .map_err(|e| {
                            error!("failed save image: {}", e);

                            RepositoryError::Db(e.into())
                        })?;

                    image_path_to_uuid.insert(dish.picture, uuid);

                    uuid
                };

                let images_value = serde_json::to_value(vec![image_uuid]).unwrap();

                for available_time in dish.available_time {
                    let model = crate::models::dish::ActiveModel {
                        id: ActiveValue::NotSet,
                        train_id: ActiveValue::Set(u64::from(train_id) as i32),
                        r#type: ActiveValue::Set(dish.dish_type.clone()),
                        time: ActiveValue::Set(available_time),
                        name: ActiveValue::Set(dish.name.clone()),
                        price: ActiveValue::Set(Decimal::from_f64(dish.price).ok_or(
                            RepositoryError::ValidationError(anyhow!(
                                "invalid price: {}",
                                dish.price
                            )),
                        )?),
                        images: ActiveValue::Set(images_value.clone()),
                    };

                    dish_model_list.push(model);
                }
            }
        }

        for dish_model_part in dish_model_list.chunks(DB_CHUNK_SIZE) {
            crate::models::dish::Entity::insert_many(dish_model_part.to_vec())
                .exec(&tx)
                .await
                .inspect_err(|e| {
                    error!("failed to insert dish: {}", e);
                })
                .context("failed to insert dish")
                .map_err(RepositoryError::Db)?;
        }

        tx.commit()
            .await
            .inspect_err(|e| {
                error!("failed to commit transaction: {}", e);
            })
            .map_err(|e| RepositoryError::Db(e.into()))?;

        if let Err(e) = self
            .event_service
            .publish_event(EventPackage::new(MicroService::Dish, DishUpdatedEvent))
            .await
        {
            error!("failed to publish dish event: {:?}", e);
        }

        Ok(())
    }

    async fn save_raw_takeaway(
        &self,
        command: SaveRawTakeawayCommand,
    ) -> Result<(), RepositoryError> {
        let mut image_path_to_uuid: HashMap<String, Uuid> = HashMap::new();

        let station_list = self
            .geo_port
            .db_get_stations()
            .await
            .inspect_err(|e| {
                error!("failed to get stations: {}", e);
            })
            .map_err(|e| RepositoryError::Db(e.into()))?;

        let station_name_to_id = station_list
            .into_iter()
            .map(|station| (station.name, StationId::from(station.id as u64)))
            .collect::<HashMap<_, _>>();

        let mut entity_list = Vec::new();

        for (station_name, inner_map) in command.takeaway {
            let station_id = *station_name_to_id.get(&station_name).ok_or_else(|| {
                RepositoryError::InconsistentState(anyhow!(
                    "station name {} not found in database",
                    station_name
                ))
            })?;

            for (shop_name, takeaway_list) in inner_map {
                let mut shop = TakeawayShop::new(shop_name, station_id);

                for takeaway in takeaway_list {
                    let image_uuid = if let Some(uuid) = image_path_to_uuid.get(&takeaway.picture) {
                        *uuid
                    } else {
                        let image_path = self.data_path.join(&takeaway.picture);

                        let image_data = fs::read(&image_path)
                            .context(format!("cannot read from: {:?}", &image_path))
                            .inspect_err(|e| {
                                error!("failed load takeaway image: {}", e);
                            })?;

                        let uuid = self
                            .object_storage_port
                            .put_object(PutObjectCommand {
                                object_category: ObjectCategory::Takeaway,
                                content_type: "image/jpeg".to_owned(),
                                object: image_data,
                            })
                            .await
                            .map_err(|e| {
                                error!("failed save image: {}", e);

                                RepositoryError::Db(e.into())
                            })?;

                        image_path_to_uuid.insert(takeaway.picture.clone(), uuid);
                        uuid
                    };

                    let takeaway_dish = TakeawayDish::new(
                        None,
                        None,
                        takeaway.name,
                        "".to_string(),
                        Decimal::from_f64(takeaway.price).ok_or(
                            RepositoryError::ValidationError(anyhow!(
                                "invalid price: {}",
                                takeaway.price
                            )),
                        )?,
                        vec![image_uuid],
                    );

                    shop.add_dish(takeaway_dish);
                }

                entity_list.push(shop);
            }
        }

        self.takeaway_shop_repository
            .save_many_atomic(entity_list)
            .await
            .inspect_err(|e| {
                error!("failed to save takeaway: {}", e);
            })?;

        if let Err(e) = self
            .event_service
            .publish_event(EventPackage::new(
                MicroService::Dish,
                TakeawayDishUpdatedEvent,
            ))
            .await
        {
            error!("failed to publish takeaway dish updated event: {:?}", e);
        }

        Ok(())
    }

    async fn db_get_dishes(&self) -> Result<Vec<DbDishDTO>, RepositoryError> {
        let result = self
            .dish_repository
            .load_all_raw()
            .await
            .inspect_err(|e| error!("Failed to load dish: {:?}", e))?;

        Ok(result
            .into_iter()
            .map(|x| DbDishDTO {
                id: x.id,
                train_id: x.train_id,
                r#type: x.r#type,
                time: x.time,
                name: x.name,
                price: x.price,
                images: x.images,
            })
            .collect())
    }

    async fn db_get_takeaway_dishes(&self) -> Result<Vec<DbTakeawayDishDTO>, RepositoryError> {
        let result = self
            .takeaway_shop_repository
            .load_all_raw()
            .await
            .inspect_err(|e| error!("Failed to load takeaway dish: {:?}", e))?;

        Ok(result
            .into_iter()
            .map(|x| DbTakeawayDishDTO {
                id: x.id,
                name: x.name,
                dish_type: x.dish_type,
                price: x.price,
                takeaway_shop_id: x.takeaway_shop_id,
                images: x.images,
            })
            .collect())
    }
}
