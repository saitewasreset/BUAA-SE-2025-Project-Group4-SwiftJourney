use crate::Verified;
use crate::domain::model::route::RouteId;
use crate::domain::model::train::{
    SeatType, SeatTypeId, SeatTypeName, Train, TrainId, TrainNumber, TrainType,
};
use crate::domain::model::train_schedule::SeatId;
use crate::domain::repository::route::RouteRepository;
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
use shared::data::{TrainNumberData, TrainTypeData};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::ops::Deref;
use std::sync::Arc;
use tracing::{debug, error, instrument, trace};

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
    #[instrument(skip_all)]
    fn make_from_db(pack: TrainDoPack) -> anyhow::Result<Train> {
        let train_id = TrainId::from_db_value(pack.train.id)?;

        let train_number = TrainNumber::from_unchecked(pack.train.number);

        let train_type = TrainType::from_unchecked(pack.train_type.type_name);

        let default_route_id = RouteId::from_db_value(pack.train.default_line_id)?;

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
        )
        .map_err(|e| {
            error!("failed to transform seat type list: {}", e);

            e
        })?;

        let seats: HashMap<_, _> = seat_type_list
            .into_iter()
            .map(|x| (x.name().to_string(), x))
            .collect();

        Ok(Train::new(
            Some(train_id),
            train_number,
            train_type,
            seats,
            default_route_id,
        ))
    }

    #[instrument]
    fn transform_to_do(train: &Train, train_type_id: i32) -> TrainActiveModelPack {
        let mut train_model = crate::models::train::ActiveModel {
            id: ActiveValue::NotSet,
            number: ActiveValue::Set(train.number().to_string()),
            type_id: ActiveValue::Set(train_type_id),
            default_line_id: ActiveValue::Set(train.default_route_id().to_db_value()),
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
    #[instrument(skip(self))]
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
            ))
            .map_err(|e| {
                error!(
                    "failed to find related train type for train id: {}: {}",
                    train.id, e
                );
                e
            })?
            .ok_or(RepositoryError::InconsistentState(anyhow!(
                "no train type for train id: {}",
                train.id
            )))
            .map_err(|e| {
                error!("no train type for train id: {}: {}", train.id, e);
                e
            })?;

        let seat_type = train_type
            .find_related(crate::models::seat_type::Entity)
            .all(&self.db)
            .await
            .context(format!(
                "failed to find related seat type for train id: {}",
                train.id
            ))
            .map_err(|e| {
                error!(
                    "failed to find related seat type for train id: {}: {}",
                    train.id, e
                );
                e
            })?;

        let pack = TrainDoPack {
            train,
            train_type,
            seat_type,
        };

        TrainDataConverter::make_from_db(pack).map_err(|e| {
            error!("failed to transform train data: {}", e);
            RepositoryError::ValidationError(e)
        })
    }
}

#[async_trait]
impl Repository<Train> for TrainRepositoryImpl {
    #[instrument(skip(self))]
    async fn find(&self, id: TrainId) -> Result<Option<Train>, RepositoryError> {
        let train = crate::models::train::Entity::find_by_id(id.to_db_value())
            .one(&self.db)
            .await
            .context(format!(
                "failed to find train with id: {}",
                id.to_db_value()
            ))
            .map_err(|e| {
                error!("failed to find train with id: {}: {}", id.to_db_value(), e);
                e
            })?;

        if let Some(train) = train {
            Ok(Some(self.find_aggregate(train).await?))
        } else {
            Ok(None)
        }
    }

    #[instrument(skip(self))]
    async fn remove(&self, aggregate: Train) -> Result<(), RepositoryError> {
        if let Some(id) = aggregate.get_id() {
            crate::models::train::Entity::delete_by_id(id.to_db_value())
                .exec(&self.db)
                .await
                .context(format!(
                    "failed to remove train with id: {}",
                    id.to_db_value()
                ))
                .map_err(|e| {
                    error!(
                        "failed to remove train with id: {}: {}",
                        id.to_db_value(),
                        e
                    );
                    e
                })?;
        }

        Ok(())
    }

    #[instrument(skip(self))]
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
            ))
            .map_err(|e| {
                error!(
                    "failed to find related train type {} for train id: {:?}: {}",
                    train_type,
                    aggregate.get_id(),
                    e
                );
                e
            })?
            .ok_or(RepositoryError::InconsistentState(anyhow!(
                "no train type {} for train id: {:?}",
                train_type,
                aggregate.get_id()
            )))
            .map_err(|e| {
                error!(
                    "no train type {} for train id: {:?}: {}",
                    train_type,
                    aggregate.get_id(),
                    e
                );
                e
            })?;

        let do_pack = TrainDataConverter::transform_to_do(aggregate, train_type_model.id);

        trace!("Begin transaction");

        let txn = self
            .db
            .begin()
            .await
            .context("cannot start database transaction")
            .map_err(|e| {
                error!("cannot start database transaction: {}", e);
                e
            })?;

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
            ))
            .map_err(|e| {
                error!(
                    "failed to save seat for train id: {:?}: {}",
                    aggregate.get_id(),
                    e
                );
                e
            })?;

        crate::models::seat_type_in_train_type::Entity::insert_many(
            do_pack.seat_type_in_train_type,
        )
        .on_conflict_do_nothing()
        .exec(&txn)
        .await
        .context(format!(
            "failed to save seat type in train type for train id: {:?}",
            aggregate.get_id()
        ))
        .map_err(|e| {
            error!(
                "failed to save seat type in train type for train id: {:?}: {}",
                aggregate.get_id(),
                e
            );
            e
        })?;

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
            ))
            .map_err(|e| {
                error!(
                    "failed to save train with id: {:?}: {}",
                    aggregate.get_id(),
                    e
                );
                e
            })?;

        let train_id = TrainId::from_db_value(result.last_insert_id)?;

        trace!("Commit transaction");
        txn.commit()
            .await
            .context("cannot commit database transaction")?;

        debug!("Train saved with id: {:?}", train_id);

        Ok(train_id)
    }
}

impl TrainRepositoryImpl {
    pub fn new(db: DatabaseConnection) -> Self {
        TrainRepositoryImpl { db }
    }

    #[instrument(skip_all)]
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
            .await
            .map_err(|e| {
                error!("failed to query table: {}", e);
                e
            })?
            .into_iter()
            .map(|model| {
                let key = key_func(&model);
                Ok((key, model))
            })
            .collect::<Result<HashMap<K, E::Model>, DbErr>>()
    }

    #[instrument(skip_all)]
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

        let models = builder(E::find()).all(&self.db).await.map_err(|e| {
            error!("failed to query table: {}", e);
            e
        })?;

        for model in models {
            let key = key_func(&model);

            result.entry(key).or_default().push(model);
        }

        Ok(result)
    }

    #[instrument(skip_all)]
    async fn query_trains(
        &self,
        builder: impl FnOnce(
            Select<crate::models::train::Entity>,
        ) -> Select<crate::models::train::Entity>,
    ) -> Result<Vec<Train>, RepositoryError> {
        let train_model_list = builder(crate::models::train::Entity::find())
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::Db(e.into()))
            .map_err(|e| {
                error!("failed to query train: {}", e);
                e
            })?;

        let mut train_list = Vec::with_capacity(train_model_list.len());

        for train_model in train_model_list {
            let train = self.find_aggregate(train_model).await.map_err(|e| {
                error!("failed to find aggregate train: {}", e);
                e
            })?;

            train_list.push(train);
        }

        Ok(train_list)
    }

    #[instrument(skip_all)]
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
            .map_err(|e| RepositoryError::Db(e.into()))
            .map_err(|e| {
                error!("failed to query train: {}", e);
                e
            })?;

        let mut train_list = Vec::with_capacity(train_model_list.len());

        for train in train_model_list {
            let train_type = train_type_map
                .get(&train.id)
                .ok_or(RepositoryError::InconsistentState(anyhow!(
                    "no train type for train id: {}",
                    train.id
                )))
                .map_err(|e| {
                    error!("no train type for train id: {}: {}", train.id, e);
                    e
                })?
                .clone();

            let seat_type = seat_type_map
                .get(&train.id)
                .ok_or(RepositoryError::InconsistentState(anyhow!(
                    "no seat type for train id: {}",
                    train.id
                )))
                .map_err(|e| {
                    error!("no seat type for train id: {}: {}", train.id, e);
                    e
                })?
                .clone();

            let pack = TrainDoPack {
                train,
                train_type,
                seat_type,
            };

            train_list.push(
                TrainDataConverter::make_from_db(pack)
                    .map_err(RepositoryError::ValidationError)
                    .map_err(|e| {
                        error!("failed to transform train data: {}", e);
                        e
                    })?,
            );
        }

        Ok(train_list)
    }
}

#[async_trait]
impl TrainRepository for TrainRepositoryImpl {
    #[instrument(skip(self))]
    async fn get_verified_train_number(&self) -> Result<HashSet<String>, RepositoryError> {
        let train_models = crate::models::train::Entity::find()
            .all(&self.db)
            .await
            .map_err(anyhow::Error::from)
            .map_err(|e| {
                error!("failed to query train: {}", e);
                e
            })?;

        Ok(train_models.into_iter().map(|e| e.number).collect())
    }

    #[instrument(skip(self))]
    async fn get_verified_train_type(&self) -> Result<HashSet<String>, RepositoryError> {
        let train_type_models = crate::models::train_type::Entity::find()
            .all(&self.db)
            .await
            .map_err(anyhow::Error::from)
            .map_err(|e| {
                error!("failed to query train type: {}", e);
                e
            })?;

        Ok(train_type_models.into_iter().map(|e| e.type_name).collect())
    }

    #[instrument(skip(self))]
    async fn get_verified_seat_type(
        &self,
        train_id: TrainId,
    ) -> Result<HashSet<String>, RepositoryError> {
        if let Some(train) = self.find(train_id).await.map_err(|e| {
            error!("failed to find train: {}", e);
            e
        })? {
            let r = crate::models::seat_type::Entity::find()
                .inner_join(crate::models::seat_type_in_train_type::Entity)
                .filter(
                    crate::models::seat_type_in_train_type::Column::TrainTypeId
                        .eq(train.train_type().to_string()),
                )
                .all(&self.db)
                .await
                .map_err(|e| RepositoryError::Db(e.into()))
                .map_err(|e| {
                    error!("failed to query seat type: {}", e);
                    e
                })
                .map(|seat_types| {
                    seat_types
                        .into_iter()
                        .map(|e| e.type_name)
                        .collect::<Vec<_>>()
                })?;

            Ok(r.into_iter().collect())
        } else {
            Ok(HashSet::default())
        }
    }

    #[instrument(skip(self))]
    async fn get_trains(&self) -> Result<Vec<Train>, RepositoryError> {
        let train_type_map = self
            .cache_table::<crate::models::train_type::Entity, _, _, _>(|q| q, |m| m.id)
            .await
            .context("failed to query train type")
            .map_err(|e| {
                error!("failed to query train type: {}", e);
                e
            })?;
        let seat_type_map = self
            .cache_table_vec::<crate::models::seat_type::Entity, _, _, _>(|q| q, |m| m.id)
            .await
            .context("failed to query train route")
            .map_err(|e| {
                error!("failed to query train route: {}", e);
                e
            })?;

        self.query_trains_cached(|q| q, &train_type_map, &seat_type_map)
            .await
    }

    #[instrument(skip(self))]
    async fn get_seat_id_map(
        &self,
        train_id: TrainId,
    ) -> Result<HashMap<SeatTypeName<Verified>, Vec<SeatId>>, RepositoryError> {
        if let Some(train) = crate::models::train::Entity::find_by_id(train_id.to_db_value())
            .one(&self.db)
            .await
            .context(format!(
                "failed to query train for train id: {}",
                train_id.to_db_value()
            ))
            .map_err(|e| {
                error!(
                    "failed to query train for train id: {}: {}",
                    train_id.to_db_value(),
                    e
                );
                e
            })?
        {
            let seat_type_mapping = crate::models::seat_type_mapping::Entity::find()
                .filter(crate::models::seat_type_mapping::Column::TrainTypeId.eq(train.type_id))
                .all(&self.db)
                .await
                .context(format!(
                    "failed to query seat type mapping for train type id: {}",
                    train.type_id
                ))
                .map_err(|e| {
                    error!(
                        "failed to query seat type mapping for train type id: {}: {}",
                        train.type_id, e
                    );
                    e
                })?;

            let seat_type = crate::models::seat_type::Entity::find()
                .all(&self.db)
                .await
                .context("failed to query seat type")?;

            let seat_type_to_name = seat_type
                .into_iter()
                .map(|x| (x.id, x.type_name))
                .collect::<HashMap<_, _>>();

            let mut result: HashMap<SeatTypeName<Verified>, Vec<SeatId>> = HashMap::new();

            for seat_type in seat_type_mapping {
                let seat_type_name = SeatTypeName::from_unchecked(
                    seat_type_to_name
                        .get(&seat_type.seat_type_id)
                        .ok_or(RepositoryError::InconsistentState(anyhow!(
                            "no seat type for seat type id: {}",
                            seat_type.seat_type_id
                        )))
                        .cloned()
                        .map_err(|e| {
                            error!(
                                "no seat type for seat type id: {}: {}",
                                seat_type.seat_type_id, e
                            );
                            e
                        })?,
                );

                result.entry(seat_type_name).or_default().push(
                    SeatId::try_from(seat_type.seat_id)
                        .map_err(|e| RepositoryError::ValidationError(e.into()))
                        .map_err(|e| {
                            error!("failed to convert seat id: {}: {}", seat_type.seat_id, e);
                            e
                        })?,
                );
            }

            Ok(result)
        } else {
            Ok(HashMap::default())
        }
    }

    #[instrument(skip(self))]
    async fn find_by_train_number(
        &self,
        train_number: TrainNumber<Verified>,
    ) -> Result<Train, RepositoryError> {
        let query_results = self
            .query_trains(|q| {
                q.filter(crate::models::train::Column::Number.eq(train_number.to_string()))
            })
            .await
            .map_err(|e| {
                error!("failed to query train by number: {}", e);
                e
            })?;

        Ok(query_results
            .into_iter()
            .next()
            .ok_or(RepositoryError::InconsistentState(anyhow!(
                "no train for verified train number: {}",
                train_number.deref()
            )))?)
    }

    #[instrument(skip(self))]
    async fn find_by_train_type(
        &self,
        train_type: TrainType<Verified>,
    ) -> Result<Vec<Train>, RepositoryError> {
        let train_type_map = self
            .cache_table::<crate::models::train_type::Entity, _, _, _>(|q| q, |m| m.id)
            .await
            .context("failed to query train type")
            .map_err(|e| {
                error!("failed to query train type: {}", e);
                e
            })?;
        let seat_type_map = self
            .cache_table_vec::<crate::models::seat_type::Entity, _, _, _>(|q| q, |m| m.id)
            .await
            .context("failed to query train route")
            .map_err(|e| {
                error!("failed to query train route: {}", e);
                e
            })?;

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

    #[instrument(skip_all)]
    async fn save_raw_train_number<T: RouteRepository>(
        &self,
        train_number_data: TrainNumberData,
        route_repository: Arc<T>,
    ) -> Result<(), RepositoryError> {
        let txn = self
            .db
            .begin()
            .await
            .context("failed to start transaction")
            .map_err(|e| {
                error!("failed to start transaction: {}", e);
                e
            })?;

        let to_insert_train_number_set = train_number_data
            .iter()
            .map(|item| item.train_number.clone())
            .collect::<HashSet<_>>();

        let trains = crate::models::train::Entity::find()
            .all(&txn)
            .await
            .context("failed to load trains")
            .map_err(|e| {
                error!("failed to load trains: {}", e);
                e
            })?;

        let to_delete_train_number = trains
            .iter()
            .filter(|item| to_insert_train_number_set.contains(&item.number))
            .map(|item| (item.number.clone(), item.clone()))
            .collect::<Vec<_>>();

        for (train_number, train_data) in to_delete_train_number {
            crate::models::route::Entity::delete_many()
                .filter(crate::models::route::Column::LineId.eq(train_data.default_line_id))
                .exec(&txn)
                .await
                .context(format!(
                    "failed to delete route with id: {}",
                    train_data.default_line_id
                ))
                .map_err(|e| {
                    error!(
                        "failed to delete route with id: {}: {}",
                        train_data.default_line_id, e
                    );
                    e
                })?;
            crate::models::train::Entity::delete_many()
                .filter(crate::models::train::Column::Number.eq(&train_number))
                .exec(&txn)
                .await
                .context(format!(
                    "failed to delete train with number: {}",
                    train_number
                ))
                .map_err(|e| {
                    error!(
                        "failed to delete train with number: {}: {}",
                        train_number, e
                    );
                    e
                })?;
        }

        let train_types = crate::models::train_type::Entity::find()
            .all(&txn)
            .await
            .context("failed to load train types")
            .map_err(|e| {
                error!("failed to load train types: {}", e);
                e
            })?;

        let train_type_to_id = train_types
            .into_iter()
            .map(|item| (item.type_name, item.id))
            .collect::<HashMap<_, _>>();

        let mut model_list = Vec::with_capacity(train_number_data.len());

        for data in train_number_data {
            let route_id = route_repository
                .save_raw(data.route)
                .await
                .context(format!(
                    "failed to save route for train number: {}",
                    &data.train_number
                ))
                .map_err(|e| {
                    error!(
                        "failed to save route for train number: {}: {}",
                        &data.train_number, e
                    );
                    e
                })?;

            let train_type_id = train_type_to_id
                .get(&data.train_type)
                .copied()
                .ok_or(RepositoryError::InconsistentState(anyhow!(
                    "no train type {} (train number: {})",
                    &data.train_type,
                    &data.train_number
                )))
                .map_err(|e| {
                    error!(
                        "no train type {} (train number: {}): {}",
                        &data.train_type, &data.train_number, e
                    );
                    e
                })?;

            let model = crate::models::train::ActiveModel {
                id: ActiveValue::NotSet,
                number: ActiveValue::Set(data.train_number),
                type_id: ActiveValue::Set(train_type_id),
                default_line_id: ActiveValue::Set(route_id.to_db_value()),
            };

            model_list.push(model);
        }

        crate::models::train::Entity::insert_many(model_list)
            .exec(&txn)
            .await
            .context("failed to save train data")
            .map_err(|e| {
                error!("failed to save train data: {}", e);
                e
            })?;

        trace!("Commit transaction");
        txn.commit().await.context("failed to commit transaction")?;

        Ok(())
    }

    #[instrument(skip_all)]
    async fn save_raw_train_type(
        &self,
        train_type_data: TrainTypeData,
    ) -> Result<(), RepositoryError> {
        let txn = self
            .db
            .begin()
            .await
            .context("failed to start transaction")
            .map_err(|e| {
                error!("failed to start transaction: {}", e);
                e
            })?;

        let train_type_list = train_type_data
            .iter()
            .map(|item| crate::models::train_type::ActiveModel {
                id: ActiveValue::NotSet,
                type_name: ActiveValue::Set(item.name.to_string()),
            })
            .collect::<Vec<_>>();

        crate::models::train_type::Entity::insert_many(train_type_list)
            .on_conflict(
                OnConflict::column(crate::models::train_type::Column::TypeName)
                    .do_nothing()
                    .to_owned(),
            )
            .exec(&txn)
            .await
            .context("failed to save raw train type data")
            .map_err(|e| {
                error!("failed to save raw train type data: {}", e);
                e
            })?;

        let to_insert_train_type_name_set = train_type_data
            .iter()
            .map(|item| item.name.to_string())
            .collect::<HashSet<_>>();

        let db_train_type_list = crate::models::train_type::Entity::find()
            .all(&txn)
            .await
            .context("failed to load train type")
            .map_err(|e| {
                error!("failed to load train type: {}", e);
                e
            })?;

        let train_type_name_to_id = db_train_type_list
            .iter()
            .map(|item| (item.type_name.clone(), item.id))
            .collect::<HashMap<_, _>>();

        let inserted_train_type_id_set = db_train_type_list
            .iter()
            .filter(|item| to_insert_train_type_name_set.contains(&item.type_name))
            .map(|item| item.id)
            .collect::<HashSet<_>>();

        let seat_type_in_train_type_list = crate::models::seat_type_in_train_type::Entity::find()
            .all(&txn)
            .await
            .context("failed to load seat type in train type")
            .map_err(|e| {
                error!("failed to load seat type in train type: {}", e);
                e
            })?;

        let to_delete_seat_type_id = seat_type_in_train_type_list
            .iter()
            .filter(|item| inserted_train_type_id_set.contains(&item.train_type_id))
            .map(|item| item.seat_type_id)
            .collect::<Vec<_>>();

        // 注意：不同车次，相同座位类型（例如，都是一等座），其seat_type_id也不同
        for seat_type_id in to_delete_seat_type_id {
            crate::models::seat_type_mapping::Entity::delete_many()
                .filter(crate::models::seat_type_mapping::Column::SeatTypeId.eq(seat_type_id))
                .exec(&txn)
                .await
                .context(format!(
                    "failed to delete seat type mapping for seat type id: {}",
                    seat_type_id
                ))
                .map_err(|e| {
                    error!(
                        "failed to delete seat type mapping for seat type id: {}: {}",
                        seat_type_id, e
                    );
                    e
                })?;

            crate::models::seat_type_in_train_type::Entity::delete_many()
                .filter(crate::models::seat_type_in_train_type::Column::SeatTypeId.eq(seat_type_id))
                .exec(&txn)
                .await
                .context(format!(
                    "failed to delete seat type in train type for seat type id: {}",
                    seat_type_id
                ))
                .map_err(|e| {
                    error!(
                        "failed to delete seat type in train type for seat type id: {}: {}",
                        seat_type_id, e
                    );
                    e
                })?;

            crate::models::seat_type::Entity::delete_many()
                .filter(crate::models::seat_type::Column::Id.eq(seat_type_id))
                .exec(&txn)
                .await
                .context(format!(
                    "failed to delete seat type for seat type id: {}",
                    seat_type_id
                ))
                .map_err(|e| {
                    error!(
                        "failed to delete seat type for seat type id: {}: {}",
                        seat_type_id, e
                    );
                    e
                })?;
        }

        let mut train_type_id_to_seat_type_name_to_id: HashMap<i32, HashMap<String, i32>> =
            HashMap::new();

        let mut seat_type_in_train_type_model_list = Vec::new();

        for train_type_info in &train_type_data {
            let train_type_id = train_type_name_to_id[&train_type_info.name];
            for (seat_type, m) in &train_type_info.seat {
                let capacity = m.values().map(|v| v.len()).sum::<usize>() as i32;

                let model = crate::models::seat_type::ActiveModel {
                    id: ActiveValue::NotSet,
                    type_name: ActiveValue::Set(seat_type.to_string()),
                    capacity: ActiveValue::Set(capacity),
                    price: ActiveValue::Set(rust_decimal::Decimal::from(
                        m.values().next().unwrap().iter().next().unwrap().price,
                    )),
                };

                let result = crate::models::seat_type::Entity::insert(model)
                    .exec(&txn)
                    .await
                    .context("failed to insert seat type")
                    .map_err(|e| {
                        error!("failed to insert seat type: {}", e);
                        e
                    })?;

                let seat_type_id = result.last_insert_id;

                seat_type_in_train_type_model_list.push(
                    crate::models::seat_type_in_train_type::ActiveModel {
                        seat_type_id: ActiveValue::Set(seat_type_id),
                        train_type_id: ActiveValue::Set(train_type_id),
                    },
                );

                train_type_id_to_seat_type_name_to_id
                    .entry(train_type_id)
                    .or_default()
                    .insert(seat_type.clone(), seat_type_id);
            }
        }

        crate::models::seat_type_in_train_type::Entity::insert_many(
            seat_type_in_train_type_model_list,
        )
        .exec(&txn)
        .await
        .context("failed to insert seat type in train type")
        .map_err(|e| {
            error!("failed to insert seat type in train type: {}", e);
            e
        })?;

        let mut seat_type_mapping_model_list = Vec::new();

        for train_type_info in train_type_data {
            let train_type_id = train_type_name_to_id[&train_type_info.name];
            for (seat_type, m) in train_type_info.seat {
                let mut current_seat_id = 0;
                let seat_type_id =
                    train_type_id_to_seat_type_name_to_id[&train_type_id][&seat_type];

                for (seat_location, seat_info_list) in m {
                    for seat_info in seat_info_list {
                        let model = crate::models::seat_type_mapping::ActiveModel {
                            train_type_id: ActiveValue::Set(train_type_id),
                            seat_type_id: ActiveValue::Set(seat_type_id),
                            seat_id: ActiveValue::Set(current_seat_id),
                            carriage: ActiveValue::Set(seat_info.description.carriage),
                            row: ActiveValue::Set(seat_info.description.row),
                            location: ActiveValue::Set(String::from(seat_location)),
                        };

                        seat_type_mapping_model_list.push(model);
                        current_seat_id += 1;
                    }
                }
            }
        }

        crate::models::seat_type_mapping::Entity::insert_many(seat_type_mapping_model_list)
            .exec(&txn)
            .await
            .context("failed to insert seat type mapping")
            .map_err(|e| {
                error!("failed to insert seat type mapping: {}", e);
                e
            })?;

        trace!("Commit transaction");

        txn.commit()
            .await
            .context("failed to commit transaction")
            .map_err(|e| {
                error!("failed to commit transaction: {}", e);
                e
            })?;
        Ok(())
    }
}
