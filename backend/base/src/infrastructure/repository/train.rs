use crate::Verified;
use crate::domain::model::train::{
    SeatType, SeatTypeId, SeatTypeName, Train, TrainId, TrainNumber, TrainType,
};
use crate::domain::repository::train::TrainRepository;
use crate::domain::{DbId, Identifiable, Repository, RepositoryError};
use crate::infrastructure::repository::transform_list;
use anyhow::{Context, anyhow};
use async_trait::async_trait;
use sea_orm::sea_query::OnConflict;
use sea_orm::{ActiveValue, DatabaseConnection, DbErr};
use sea_orm::{ColumnTrait, ModelTrait};
use sea_orm::{EntityTrait, TransactionTrait};
use sea_orm::{QueryFilter, Select};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

impl_db_id_from_u64!(TrainId, i32, "train");
impl_db_id_from_u64!(SeatTypeId, i32, "seat type");

struct TrainDoPack {
    pub train: crate::models::train::Model,
    pub train_type: crate::models::train_type::Model,
    pub seat_type: Vec<crate::models::seat_type::Model>,
}

struct TrainActiveModelPack {
    pub train: crate::models::train::ActiveModel,
    pub seat_type: Vec<crate::models::seat_type::ActiveModel>,
    pub seat_type_in_train_type: Vec<crate::models::seat_type_in_train_type::ActiveModel>,
}

pub struct TrainDataConverter;

impl TrainDataConverter {
    fn make_from_db(pack: TrainDoPack) -> anyhow::Result<Train> {
        let train_id = TrainId::from_db_value(pack.train.id)?;

        let train_number = TrainNumber::from_unchecked(pack.train.number);

        let train_type = TrainType::from_unchecked(pack.train_type.type_name);

        let seat_type_list = transform_list(
            pack.seat_type,
            |model| {
                let seat_type_id = SeatTypeId::from_db_value(model.id)?;
                let seat_type_name = SeatTypeName::from_unchecked(model.type_name);
                let capacity = model.capacity as u32;
                let price = model.price;

                Ok(SeatType::new(
                    Some(seat_type_id),
                    seat_type_name,
                    capacity,
                    price,
                ))
            },
            |model| model.id,
        )?;

        let seats: HashMap<_, _> = seat_type_list
            .into_iter()
            .map(|x| (x.name().to_string(), x))
            .collect();

        Ok(Train::new(Some(train_id), train_number, train_type, seats))
    }

    fn transform_to_do(train: &Train, train_type_id: i32) -> TrainActiveModelPack {
        let mut train_model = crate::models::train::ActiveModel {
            id: ActiveValue::NotSet,
            number: ActiveValue::Set(train.number().to_string()),
            type_id: ActiveValue::Set(train_type_id),
        };

        if let Some(id) = train.get_id() {
            train_model.id = ActiveValue::Set(id.to_db_value());
        }

        let mut seat_type_models = Vec::with_capacity(train.seats().len());
        let mut seat_type_in_train_type_models = Vec::with_capacity(train.seats().len());

        for seat in train.seats().values() {
            let mut seat_type_model = crate::models::seat_type::ActiveModel {
                id: ActiveValue::NotSet,
                type_name: ActiveValue::Set(seat.name().to_string()),
                capacity: ActiveValue::Set(seat.capacity() as i32),
                price: ActiveValue::Set(seat.unit_price()),
            };

            let mut seat_type_in_train_type_model =
                crate::models::seat_type_in_train_type::ActiveModel {
                    train_type_id: ActiveValue::Set(train_type_id),
                    seat_type_id: ActiveValue::NotSet,
                };

            if let Some(id) = seat.get_id() {
                seat_type_model.id = ActiveValue::Set(id.to_db_value());
                seat_type_in_train_type_model.seat_type_id = ActiveValue::Set(id.to_db_value());

                seat_type_models.push(seat_type_model);
                seat_type_in_train_type_models.push(seat_type_in_train_type_model);
            }
        }

        TrainActiveModelPack {
            train: train_model,
            seat_type: seat_type_models,
            seat_type_in_train_type: seat_type_in_train_type_models,
        }
    }
}

pub struct TrainRepositoryImpl {
    db: DatabaseConnection,
}

impl TrainRepositoryImpl {
    async fn find_aggregate(
        &self,
        train: crate::models::train::Model,
    ) -> Result<Train, RepositoryError> {
        let train_type = train
            .find_related(crate::models::train_type::Entity)
            .one(&self.db)
            .await
            .context(format!(
                "failed to find related train type for train id: {}",
                train.id
            ))?
            .ok_or(RepositoryError::InconsistentState(anyhow!(
                "no train type for train id: {}",
                train.id
            )))?;

        let seat_type = train_type
            .find_related(crate::models::seat_type::Entity)
            .all(&self.db)
            .await
            .context(format!(
                "failed to find related seat type for train id: {}",
                train.id
            ))?;

        let pack = TrainDoPack {
            train,
            train_type,
            seat_type,
        };

        TrainDataConverter::make_from_db(pack).map_err(RepositoryError::ValidationError)
    }
}

#[async_trait]
impl Repository<Train> for TrainRepositoryImpl {
    async fn find(&self, id: TrainId) -> Result<Option<Train>, RepositoryError> {
        let train = crate::models::train::Entity::find_by_id(id.to_db_value())
            .one(&self.db)
            .await
            .context(format!(
                "failed to find train with id: {}",
                id.to_db_value()
            ))?;

        if let Some(train) = train {
            Ok(Some(self.find_aggregate(train).await?))
        } else {
            Ok(None)
        }
    }

    async fn remove(&self, aggregate: Train) -> Result<(), RepositoryError> {
        if let Some(id) = aggregate.get_id() {
            crate::models::train::Entity::delete_by_id(id.to_db_value())
                .exec(&self.db)
                .await
                .context(format!(
                    "failed to remove train with id: {}",
                    id.to_db_value()
                ))?;
        }

        Ok(())
    }

    async fn save(&self, aggregate: &mut Train) -> Result<TrainId, RepositoryError> {
        let train_type = aggregate.train_type();

        let train_type_model = crate::models::train_type::Entity::find()
            .filter(crate::models::train_type::Column::TypeName.eq(train_type))
            .one(&self.db)
            .await
            .context(format!(
                "failed to find related train type {} for train id: {:?}",
                train_type,
                aggregate.get_id()
            ))?
            .ok_or(RepositoryError::InconsistentState(anyhow!(
                "no train type {} for train id: {:?}",
                train_type,
                aggregate.get_id()
            )))?;

        let do_pack = TrainDataConverter::transform_to_do(aggregate, train_type_model.id);

        let txn = self
            .db
            .begin()
            .await
            .context("cannot start database transaction")?;

        crate::models::seat_type::Entity::insert_many(do_pack.seat_type)
            .on_conflict(
                OnConflict::column(crate::models::seat_type::Column::Id)
                    .update_columns([
                        crate::models::seat_type::Column::TypeName,
                        crate::models::seat_type::Column::Capacity,
                        crate::models::seat_type::Column::Price,
                    ])
                    .to_owned(),
            )
            .exec(&txn)
            .await
            .context(format!(
                "failed to save seat for train id: {:?}",
                aggregate.get_id()
            ))?;

        crate::models::seat_type_in_train_type::Entity::insert_many(
            do_pack.seat_type_in_train_type,
        )
        .on_conflict_do_nothing()
        .exec(&txn)
        .await
        .context(format!(
            "failed to save seat type in train type for train id: {:?}",
            aggregate.get_id()
        ))?;

        let result = crate::models::train::Entity::insert(do_pack.train)
            .on_conflict(
                OnConflict::column(crate::models::train::Column::Id)
                    .update_columns([
                        crate::models::train::Column::TypeId,
                        crate::models::train::Column::Number,
                    ])
                    .to_owned(),
            )
            .exec(&txn)
            .await
            .context(format!(
                "failed to save train with id: {:?}",
                aggregate.get_id()
            ))?;

        let train_id = TrainId::from_db_value(result.last_insert_id)?;

        txn.commit()
            .await
            .context("cannot commit database transaction")?;

        Ok(train_id)
    }
}

impl TrainRepositoryImpl {
    pub fn new(db: DatabaseConnection) -> Self {
        TrainRepositoryImpl { db }
    }

    async fn cache_table<E, K, F, B>(
        &self,
        builder: B,
        key_func: F,
    ) -> Result<HashMap<K, E::Model>, DbErr>
    where
        E: EntityTrait,
        K: Hash + Eq + Clone,
        F: Fn(&E::Model) -> K,
        B: FnOnce(Select<E>) -> Select<E>,
    {
        builder(E::find())
            .all(&self.db)
            .await?
            .into_iter()
            .map(|model| {
                let key = key_func(&model);
                Ok((key, model))
            })
            .collect::<Result<HashMap<K, E::Model>, DbErr>>()
    }

    async fn cache_table_vec<E, K, F, B>(
        &self,
        builder: B,
        key_func: F,
    ) -> Result<HashMap<K, Vec<E::Model>>, DbErr>
    where
        E: EntityTrait,
        K: Hash + Eq + Clone,
        F: Fn(&E::Model) -> K,
        B: FnOnce(Select<E>) -> Select<E>,
    {
        let mut result: HashMap<K, Vec<E::Model>> = HashMap::new();

        let models = builder(E::find()).all(&self.db).await?;

        for model in models {
            let key = key_func(&model);

            result.entry(key).or_default().push(model);
        }

        Ok(result)
    }

    async fn query_trains(
        &self,
        builder: impl FnOnce(
            Select<crate::models::train::Entity>,
        ) -> Select<crate::models::train::Entity>,
    ) -> Result<Vec<Train>, RepositoryError> {
        let train_model_list = builder(crate::models::train::Entity::find())
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::Db(e.into()))?;

        let mut train_list = Vec::with_capacity(train_model_list.len());

        for train_model in train_model_list {
            let train = self.find_aggregate(train_model).await?;

            train_list.push(train);
        }

        Ok(train_list)
    }

    async fn query_trains_cached(
        &self,
        builder: impl FnOnce(
            Select<crate::models::train::Entity>,
        ) -> Select<crate::models::train::Entity>,
        train_type_map: &HashMap<i32, crate::models::train_type::Model>,
        seat_type_map: &HashMap<i32, Vec<crate::models::seat_type::Model>>,
    ) -> Result<Vec<Train>, RepositoryError> {
        let train_model_list = builder(crate::models::train::Entity::find())
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::Db(e.into()))?;

        let mut train_list = Vec::with_capacity(train_model_list.len());

        for train in train_model_list {
            let train_type = train_type_map
                .get(&train.id)
                .ok_or(RepositoryError::InconsistentState(anyhow!(
                    "no train type for train id: {}",
                    train.id
                )))?
                .clone();

            let seat_type = seat_type_map
                .get(&train.id)
                .ok_or(RepositoryError::InconsistentState(anyhow!(
                    "no seat type for train id: {}",
                    train.id
                )))?
                .clone();

            let pack = TrainDoPack {
                train,
                train_type,
                seat_type,
            };

            train_list.push(
                TrainDataConverter::make_from_db(pack).map_err(RepositoryError::ValidationError)?,
            );
        }

        Ok(train_list)
    }
}

#[async_trait]
impl TrainRepository for TrainRepositoryImpl {
    async fn get_verified_train_number(
        &self,
    ) -> Result<HashSet<TrainNumber<Verified>>, RepositoryError> {
        let train_models = crate::models::train::Entity::find()
            .all(&self.db)
            .await
            .map_err(anyhow::Error::from)?;

        Ok(train_models
            .into_iter()
            .map(|e| TrainNumber::from_unchecked(e.number))
            .collect())
    }

    async fn get_verified_train_type(
        &self,
    ) -> Result<HashSet<TrainType<Verified>>, RepositoryError> {
        let train_type_models = crate::models::train_type::Entity::find()
            .all(&self.db)
            .await
            .map_err(anyhow::Error::from)?;

        Ok(train_type_models
            .into_iter()
            .map(|e| TrainType::from_unchecked(e.type_name))
            .collect())
    }

    async fn get_verified_seat_type(
        &self,
        train_id: TrainId,
    ) -> Result<HashSet<SeatType>, RepositoryError> {
        if let Some(train) = self.find(train_id).await? {
            let r = crate::models::seat_type::Entity::find()
                .inner_join(crate::models::seat_type_in_train_type::Entity)
                .filter(
                    crate::models::seat_type_in_train_type::Column::TrainTypeId
                        .eq(train.train_type().to_string()),
                )
                .all(&self.db)
                .await
                .map_err(|e| RepositoryError::Db(e.into()))
                .and_then(|seat_types| {
                    transform_list(
                        seat_types,
                        |model| {
                            let seat_type_id = SeatTypeId::from_db_value(model.id)?;
                            let seat_type_name = SeatTypeName::from_unchecked(model.type_name);
                            let capacity = model.capacity as u32;
                            let price = model.price;

                            Ok(SeatType::new(
                                Some(seat_type_id),
                                seat_type_name,
                                capacity,
                                price,
                            ))
                        },
                        |model| model.id,
                    )
                    .map_err(RepositoryError::ValidationError)
                })?;

            Ok(r.into_iter().collect())
        } else {
            Ok(HashSet::default())
        }
    }

    async fn find_by_train_number(
        &self,
        train_number: TrainNumber<Verified>,
    ) -> Result<Option<Train>, RepositoryError> {
        let query_results = self
            .query_trains(|q| {
                q.filter(crate::models::train::Column::Number.eq(train_number.to_string()))
            })
            .await?;

        Ok(query_results.into_iter().next())
    }

    async fn find_by_train_type(
        &self,
        train_type: TrainType<Verified>,
    ) -> Result<Vec<Train>, RepositoryError> {
        let train_type_map = self
            .cache_table::<crate::models::train_type::Entity, _, _, _>(|q| q, |m| m.id)
            .await
            .context("failed to query train type")?;
        let seat_type_map = self
            .cache_table_vec::<crate::models::seat_type::Entity, _, _, _>(|q| q, |m| m.id)
            .await
            .context("failed to query train route")?;

        let train_type_id = train_type_map
            .values()
            .filter(|m| m.type_name == &train_type as &str)
            .map(|m| m.id)
            .next();

        if let Some(train_type_id) = train_type_id {
            self.query_trains_cached(
                |q| q.filter(crate::models::train::Column::TypeId.eq(train_type_id)),
                &train_type_map,
                &seat_type_map,
            )
            .await
        } else {
            Ok(Vec::new())
        }
    }
}
