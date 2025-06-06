use crate::domain::model::dish::{Dish, DishId, DishTime};
use crate::domain::model::train::{TrainId, TrainNumber};
use crate::domain::repository::dish::DishRepository;
use crate::domain::repository::train::TrainRepository;
use crate::domain::service::object_storage::{ObjectCategory, ObjectStorageService};
use crate::domain::{DbId, Identifiable, Repository, RepositoryError};
use crate::{DB_CHUNK_SIZE, Verified};
use anyhow::{Context, anyhow};
use async_trait::async_trait;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use sea_orm::QueryFilter;
use sea_orm::{ActiveValue, DatabaseConnection, EntityTrait};
use sea_orm::{ColumnTrait, TransactionTrait};
use shared::data::DishData;
use std::collections::HashMap;
use std::fs;
use std::ops::Deref;
use std::path::Path;
use std::sync::Arc;
use tracing::{error, instrument};
use uuid::Uuid;

pub struct DishDataConverter;

impl DishDataConverter {
    pub fn make_from_do(dish: crate::models::dish::Model) -> Result<Dish, anyhow::Error> {
        let id = DishId::from_db_value(dish.id)?;
        let train_id = TrainId::from_db_value(dish.train_id)?;
        let dish_type = dish.r#type;
        let dish_time = DishTime::try_from(dish.time.as_str())
            .map_err(|e| anyhow!("invalid dish time: {}", e))?;
        let name = dish.name;
        let unit_price = dish.price;
        let images: Vec<Uuid> = serde_json::from_value(dish.images)?;

        Ok(Dish::new(
            Some(id),
            train_id,
            dish_type,
            dish_time,
            name,
            unit_price,
            images,
        ))
    }

    pub fn transform_to_do(dish: Dish) -> crate::models::dish::ActiveModel {
        let image_value = serde_json::to_value(dish.images()).unwrap();

        let mut result = crate::models::dish::ActiveModel {
            id: ActiveValue::NotSet,
            train_id: ActiveValue::Set(dish.train_id().to_db_value()),
            r#type: ActiveValue::Set(dish.dish_type().to_string()),
            time: ActiveValue::Set(dish.dish_time().to_string()),
            name: ActiveValue::Set(dish.name().to_string()),
            price: ActiveValue::Set(dish.unit_price()),
            images: ActiveValue::Set(image_value),
        };

        if let Some(id) = dish.get_id() {
            result.id = ActiveValue::Set(id.to_db_value());
        }

        result
    }
}

impl_db_id_from_u64!(DishId, i32, "dish id");

pub struct DishRepositoryImpl {
    db: DatabaseConnection,
}

impl DishRepositoryImpl {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl Repository<Dish> for DishRepositoryImpl {
    async fn find(&self, id: DishId) -> Result<Option<Dish>, RepositoryError> {
        let model = crate::models::dish::Entity::find_by_id(id.to_db_value())
            .one(&self.db)
            .await
            .context(format!("failed to find dish {}", id))
            .map_err(RepositoryError::Db)?;

        let result = model.map(DishDataConverter::make_from_do).transpose()?;

        Ok(result)
    }

    async fn remove(&self, aggregate: Dish) -> Result<(), RepositoryError> {
        if let Some(id) = aggregate.get_id() {
            crate::models::dish::Entity::delete_by_id(id.to_db_value())
                .exec(&self.db)
                .await
                .context(format!("failed to delete dish: {}", id.to_db_value()))
                .map_err(RepositoryError::Db)?;
        }

        Ok(())
    }

    async fn save(&self, aggregate: &mut Dish) -> Result<DishId, RepositoryError> {
        let model = DishDataConverter::transform_to_do(aggregate.clone());

        if let Some(id) = aggregate.get_id() {
            crate::models::dish::Entity::update(model)
                .exec(&self.db)
                .await
                .context(format!("failed to update dish {}", id))
                .map_err(RepositoryError::Db)?;

            Ok(id)
        } else {
            let result = crate::models::dish::Entity::insert(model)
                .exec(&self.db)
                .await
                .context("failed to insert dish")
                .map_err(RepositoryError::Db)?;

            let new_id = DishId::from_db_value(result.last_insert_id)?;

            aggregate.set_id(new_id);

            Ok(new_id)
        }
    }
}

#[async_trait]
impl DishRepository for DishRepositoryImpl {
    async fn find_by_train_number(
        &self,
        train_number: TrainNumber<Verified>,
    ) -> Result<Vec<Dish>, RepositoryError> {
        let dish_list = crate::models::dish::Entity::find()
            .inner_join(crate::models::train::Entity)
            .filter(crate::models::train::Column::Number.eq(train_number.to_string()))
            .all(&self.db)
            .await
            .context(format!(
                "failed to find dishes by train number {}",
                train_number.deref()
            ))
            .map_err(RepositoryError::Db)?;

        let mut result = Vec::with_capacity(dish_list.len());

        for dish_do in dish_list {
            result.push(DishDataConverter::make_from_do(dish_do)?);
        }

        Ok(result)
    }

    #[instrument(skip_all)]
    async fn save_raw_dish<T: TrainRepository, OS: ObjectStorageService>(
        &self,
        data: DishData,
        data_path: &Path,
        train_repository: Arc<T>,
        object_storage_service: Arc<OS>,
    ) -> Result<(), RepositoryError> {
        let tx = self
            .db
            .begin()
            .await
            .inspect_err(|e| {
                error!("failed to begin transaction: {}", e);
            })
            .map_err(|e| RepositoryError::Db(e.into()))?;

        let mut image_path_to_uuid: HashMap<String, Uuid> = HashMap::new();

        let train_list = train_repository
            .get_trains()
            .await
            .inspect_err(|e| {
                error!("failed to get trains: {}", e);
            })
            .map_err(|e| RepositoryError::Db(e.into()))?;

        let train_number_str_to_id = train_list
            .iter()
            .map(|train| (train.number().to_string(), train.get_id().unwrap()))
            .collect::<HashMap<_, _>>();

        let mut dish_model_list = Vec::new();

        for (train_number_str, dish_list) in data {
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
                    let image_path = data_path.join(&dish.picture);

                    let image_data = fs::read(&image_path)
                        .context(format!("cannot read from: {:?}", &image_path))
                        .inspect_err(|e| {
                            error!("failed load dish image: {}", e);
                        })?;

                    let uuid = object_storage_service
                        .put_object(ObjectCategory::Dish, "image/jpeg", image_data)
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
                        train_id: ActiveValue::Set(train_id.to_db_value()),
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
        Ok(())
    }
}
