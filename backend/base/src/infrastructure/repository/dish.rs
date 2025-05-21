use crate::Verified;
use crate::domain::model::dish::{Dish, DishId, DishTime};
use crate::domain::model::train::{TrainId, TrainNumber};
use crate::domain::repository::dish::DishRepository;
use crate::domain::{DbId, Identifiable, Repository, RepositoryError};
use anyhow::{Context, anyhow};
use async_trait::async_trait;
use sea_orm::ColumnTrait;
use sea_orm::QueryFilter;
use sea_orm::{ActiveValue, DatabaseConnection, EntityTrait};
use std::ops::Deref;
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
            .filter(crate::models::dish::Column::TrainId.eq(train_number.deref()))
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
}
