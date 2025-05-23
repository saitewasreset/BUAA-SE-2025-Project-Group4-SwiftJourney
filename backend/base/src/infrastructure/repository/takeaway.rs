use crate::domain::model::hotel::Hotel;
use crate::domain::model::route::{RouteId, Stop, StopId};
use crate::domain::model::station::StationId;
use crate::domain::model::takeaway::{TakeawayDish, TakeawayDishId, TakeawayShop, TakeawayShopId};
use crate::domain::repository::station::StationRepository;
use crate::domain::repository::takeaway::TakeawayShopRepository;
use crate::domain::repository::train::TrainRepository;
use crate::domain::service::object_storage::{ObjectCategory, ObjectStorageService};
use crate::domain::service::{AggregateManagerImpl, DiffInfo};
use crate::domain::{
    DbId, DbRepositorySupport, Diff, DiffType, Identifiable, MultiEntityDiff, Repository,
    RepositoryError, TypedDiff,
};
use anyhow::{Context, anyhow};
use async_trait::async_trait;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use sea_orm::{
    ActiveValue, DatabaseBackend, DatabaseConnection, EntityTrait, FromQueryResult, JsonValue,
    QueryFilter, Statement, TransactionTrait,
};
use shared::data::TakeawayData;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tracing::{error, instrument};
use uuid::Uuid;

impl_db_id_from_u64!(TakeawayDishId, i32, "takeaway dish id");
impl_db_id_from_u64!(TakeawayShopId, i32, "takeaway shop id");

pub struct TakeawayDishDataConverter;
pub struct TakeawayShopDataConverter;

impl TakeawayDishDataConverter {
    pub fn make_from_do(
        takeaway_dish_do: crate::models::takeaway_dish::Model,
    ) -> Result<TakeawayDish, anyhow::Error> {
        let images: Vec<Uuid> = serde_json::from_value(takeaway_dish_do.images)?;

        let id = TakeawayDishId::from_db_value(takeaway_dish_do.id)?;

        let takeaway_shop_id = TakeawayShopId::from_db_value(takeaway_dish_do.takeaway_shop_id)?;

        Ok(TakeawayDish::new(
            Some(id),
            Some(takeaway_shop_id),
            takeaway_dish_do.name,
            takeaway_dish_do.dish_type,
            takeaway_dish_do.price,
            images,
        ))
    }

    pub fn transform_to_do(
        takeaway_dish: &TakeawayDish,
    ) -> crate::models::takeaway_dish::ActiveModel {
        let images_json = serde_json::to_value(takeaway_dish.images()).unwrap();

        let mut model = crate::models::takeaway_dish::ActiveModel {
            id: ActiveValue::NotSet,
            name: ActiveValue::Set(takeaway_dish.name().to_string()),
            dish_type: ActiveValue::Set(takeaway_dish.dish_type().to_string()),
            price: ActiveValue::Set(takeaway_dish.unit_price()),
            takeaway_shop_id: ActiveValue::NotSet,
            images: ActiveValue::Set(images_json),
        };

        if let Some(id) = takeaway_dish.get_id() {
            model.id = ActiveValue::Set(id.to_db_value());
        }

        if let Some(shop_id) = takeaway_dish.shop_id() {
            model.takeaway_shop_id = ActiveValue::Set(shop_id.to_db_value());
        }

        model
    }
}

pub struct TakeawayShopDoPack {
    pub takeaway_shop_do: crate::models::takeaway_shop::Model,
    pub takeaway_shop_dish_do_list: Vec<crate::models::takeaway_dish::Model>,
}

pub struct TakeawayShopActiveModelPack {
    pub takeaway_shop_do: crate::models::takeaway_shop::ActiveModel,
    pub takeaway_shop_dish_do_list: Vec<crate::models::takeaway_dish::ActiveModel>,
}

impl TakeawayShopDataConverter {
    pub fn make_from_do(do_pack: TakeawayShopDoPack) -> Result<TakeawayShop, anyhow::Error> {
        let mut takeaway_dish_list = Vec::with_capacity(do_pack.takeaway_shop_dish_do_list.len());

        for takeaway_dish_do in do_pack.takeaway_shop_dish_do_list {
            let takeaway_dish = TakeawayDishDataConverter::make_from_do(takeaway_dish_do)?;
            takeaway_dish_list.push(takeaway_dish);
        }

        let images: Vec<Uuid> = serde_json::from_value(do_pack.takeaway_shop_do.images)?;

        let shop_id = TakeawayShopId::from_db_value(do_pack.takeaway_shop_do.id)?;
        let station_id = StationId::from_db_value(do_pack.takeaway_shop_do.station_id)?;

        Ok(TakeawayShop::new_full(
            Some(shop_id),
            do_pack.takeaway_shop_do.uuid,
            do_pack.takeaway_shop_do.name,
            station_id,
            images,
            takeaway_dish_list,
        ))
    }

    pub fn transform_to_do_shop_only(
        takeaway_shop: &TakeawayShop,
    ) -> crate::models::takeaway_shop::ActiveModel {
        let mut model = crate::models::takeaway_shop::ActiveModel {
            id: ActiveValue::NotSet,
            uuid: ActiveValue::Set(takeaway_shop.uuid()),
            name: ActiveValue::Set(takeaway_shop.name().to_string()),
            station_id: ActiveValue::Set(takeaway_shop.station_id().to_db_value()),
            images: ActiveValue::Set(serde_json::to_value(takeaway_shop.images()).unwrap()),
        };

        if let Some(id) = takeaway_shop.get_id() {
            model.id = ActiveValue::Set(id.to_db_value());
        }

        model
    }

    pub fn transform_to_do(takeaway_shop: &TakeawayShop) -> TakeawayShopActiveModelPack {
        let mut model = Self::transform_to_do_shop_only(takeaway_shop);

        let mut takeaway_dish_do_list = Vec::with_capacity(takeaway_shop.dishes().len());

        for takeaway_dish in takeaway_shop.dishes() {
            let dish_do = TakeawayDishDataConverter::transform_to_do(takeaway_dish);
            takeaway_dish_do_list.push(dish_do);
        }

        TakeawayShopActiveModelPack {
            takeaway_shop_do: model,
            takeaway_shop_dish_do_list: takeaway_dish_do_list,
        }
    }
}

pub struct TakeawayShopRepositoryImpl {
    db: DatabaseConnection,
    aggregate_manager: Arc<Mutex<AggregateManagerImpl<TakeawayShop>>>,
}

impl TakeawayShopRepositoryImpl {
    pub fn new(db: DatabaseConnection) -> Self {
        let detect_changes_fn = |diff: DiffInfo<TakeawayShop>| {
            let mut result = MultiEntityDiff::new();

            let old = diff.old;
            let new = diff.new;

            if let (Some(old), Some(new)) = (old, new) {
                if !(old.name() == new.name()
                    && old.uuid() == new.uuid()
                    && old.station_id() == new.station_id()
                    && old.images() == new.images())
                {
                    result.add_change(TypedDiff::new(
                        DiffType::Modified,
                        Some(old.clone()),
                        Some(new.clone()),
                    ));
                }

                let old_dish_id_to_dish = old
                    .dishes()
                    .iter()
                    .map(|v| (v.get_id().unwrap(), v.clone()))
                    .collect::<HashMap<_, _>>();

                let new_dish_id_to_dish = new
                    .dishes()
                    .iter()
                    .map(|v| (v.get_id().unwrap(), v.clone()))
                    .collect::<HashMap<_, _>>();

                for (dish_id, old_data) in &old_dish_id_to_dish {
                    if let Some(new_data) = new_dish_id_to_dish.get(dish_id) {
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

                for (dish_id, new_data) in &new_dish_id_to_dish {
                    if !old_dish_id_to_dish.contains_key(dish_id) {
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
        }
    }
}

#[async_trait]
impl DbRepositorySupport<TakeawayShop> for TakeawayShopRepositoryImpl {
    type Manager = AggregateManagerImpl<TakeawayShop>;
    fn get_aggregate_manager(&self) -> Arc<Mutex<Self::Manager>> {
        Arc::clone(&self.aggregate_manager)
    }

    #[instrument(skip(self))]
    async fn on_insert(&self, aggregate: TakeawayShop) -> Result<TakeawayShopId, RepositoryError> {
        let mut do_pack = TakeawayShopDataConverter::transform_to_do(&aggregate);

        let tx = self
            .db
            .begin()
            .await
            .inspect_err(|e| {
                error!("Failed to begin transaction: {}", e);
            })
            .context("Failed to begin transaction")?;

        let result = crate::models::takeaway_shop::Entity::insert(do_pack.takeaway_shop_do)
            .exec(&tx)
            .await
            .inspect_err(|e| {
                error!("Failed to insert takeaway shop: {}", e);
            })
            .map_err(|e| RepositoryError::Db(e.into()))?;

        let takeaway_shop_id = result.last_insert_id;

        do_pack
            .takeaway_shop_dish_do_list
            .iter_mut()
            .for_each(|dish| dish.takeaway_shop_id = ActiveValue::Set(takeaway_shop_id));

        let takeaway_shop_id = TakeawayShopId::from_db_value(takeaway_shop_id)?;

        crate::models::takeaway_dish::Entity::insert_many(do_pack.takeaway_shop_dish_do_list)
            .exec(&tx)
            .await
            .inspect_err(|e| {
                error!("Failed to insert takeaway dish: {}", e);
            })
            .map_err(|e| RepositoryError::Db(e.into()))?;

        tx.commit()
            .await
            .inspect_err(|e| {
                error!("Failed to commit transaction: {}", e);
            })
            .context("Failed to commit transaction")?;

        Ok(takeaway_shop_id)
    }

    #[instrument(skip(self))]
    async fn on_select(&self, id: TakeawayShopId) -> Result<Option<TakeawayShop>, RepositoryError> {
        let r = crate::models::takeaway_shop::Entity::find_by_id(id.to_db_value())
            .find_with_related(crate::models::takeaway_dish::Entity)
            .all(&self.db)
            .await
            .inspect_err(|e| {
                error!("Failed to select takeaway shop: {}", e);
            })
            .map_err(|e| RepositoryError::Db(e.into()))?;

        let r = r.into_iter().next();

        if let Some((takeaway_shop_do, takeaway_shop_dish_do_list)) = r {
            let do_pack = TakeawayShopDoPack {
                takeaway_shop_do,
                takeaway_shop_dish_do_list,
            };

            Ok(Some(TakeawayShopDataConverter::make_from_do(do_pack)?))
        } else {
            Ok(None)
        }
    }

    #[instrument(skip_all)]
    async fn on_update(&self, diff: MultiEntityDiff) -> Result<(), RepositoryError> {
        let tx = self
            .db
            .begin()
            .await
            .inspect_err(|e| {
                error!("Failed to begin transaction: {}", e);
            })
            .context("Failed to begin transaction")?;

        for change in diff.get_changes::<TakeawayShop>() {
            match change.diff_type() {
                DiffType::Unchanged => {}
                DiffType::Added => {
                    panic!("Added TakeawayShop entity should go through on_insert")
                }
                DiffType::Modified => {
                    let new_do = TakeawayShopDataConverter::transform_to_do_shop_only(
                        &change.new_value.unwrap(),
                    );

                    crate::models::takeaway_shop::Entity::update(new_do)
                        .exec(&tx)
                        .await
                        .inspect_err(|e| {
                            error!("Failed to update takeaway shop: {}", e);
                        })
                        .map_err(|e| RepositoryError::Db(e.into()))?;
                }
                DiffType::Removed => {
                    panic!("Removed TakeawayShop entity should go through on_delete")
                }
            }
        }

        for change in diff.get_changes::<TakeawayDish>() {
            match change.diff_type() {
                DiffType::Unchanged => {}
                DiffType::Added => {
                    let new_do = TakeawayDishDataConverter::transform_to_do(
                        change.new_value.as_ref().unwrap(),
                    );

                    crate::models::takeaway_dish::Entity::insert(new_do)
                        .exec(&tx)
                        .await
                        .inspect_err(|e| {
                            error!("Failed to insert takeaway dish: {}", e);
                        })
                        .map_err(|e| RepositoryError::Db(e.into()))?;
                }
                DiffType::Modified => {
                    let new_do = TakeawayDishDataConverter::transform_to_do(
                        change.new_value.as_ref().unwrap(),
                    );

                    crate::models::takeaway_dish::Entity::update(new_do)
                        .exec(&tx)
                        .await
                        .inspect_err(|e| {
                            error!("Failed to update takeaway dish: {}", e);
                        })
                        .map_err(|e| RepositoryError::Db(e.into()))?;
                }
                DiffType::Removed => {
                    let id = change.old_value.as_ref().unwrap().get_id().unwrap();

                    crate::models::takeaway_dish::Entity::delete_by_id(id.to_db_value())
                        .exec(&tx)
                        .await
                        .inspect_err(|e| {
                            error!("Failed to delete takeaway dish: {}", e);
                        })
                        .map_err(|e| RepositoryError::Db(e.into()))?;
                }
            }
        }

        tx.commit()
            .await
            .inspect_err(|e| {
                error!("Failed to commit transaction: {}", e);
            })
            .context("Failed to commit transaction")?;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn on_delete(&self, aggregate: TakeawayShop) -> Result<(), RepositoryError> {
        if let Some(id) = aggregate.get_id() {
            let tx = self
                .db
                .begin()
                .await
                .context("Failed to begin transaction")?;

            crate::models::takeaway_shop::Entity::delete_by_id(id.to_db_value())
                .exec(&tx)
                .await
                .inspect_err(|e| {
                    error!("Failed to delete takeaway shop: {}", e);
                })
                .map_err(|e| RepositoryError::Db(e.into()))?;

            tx.commit().await.context("Failed to commit transaction")?;
        }

        Ok(())
    }
}

#[async_trait]
impl TakeawayShopRepository for TakeawayShopRepositoryImpl {
    #[instrument(skip(self))]
    async fn find_by_train_route(
        &self,
        route_id: RouteId,
    ) -> Result<HashMap<Stop, Vec<TakeawayShop>>, RepositoryError> {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, FromQueryResult)]
        struct QueryResult {
            shop_id: i32,
            shop_uuid: Uuid,
            shop_name: String,
            shop_station_id: i32,
            shop_images: JsonValue,
            dish_id: i32,
            dish_name: String,
            dish_type: String,
            dish_price: Decimal,
            dish_takeaway_shop_id: i32,
            dish_images: JsonValue,
            stop_id: i32,
            stop_route_id: i64,
            route_station_id: i32,
            route_arrival_time: i32,
            route_departure_time: i32,
            route_order: i32,
        }

        let r = QueryResult::find_by_statement(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            r#"SELECT
    "takeaway_shop"."id" AS "shop_id",
    "takeaway_shop"."uuid" AS "shop_uuid",
    "takeaway_shop"."name" AS "shop_name",
    "takeaway_shop"."station_id" AS "shop_station_id",
    "takeaway_shop"."images" AS "shop_images",
    "takeaway_dish"."id" AS "dish_id",
    "takeaway_dish"."name" AS "dish_name",
    "takeaway_dish"."dish_type" AS "dish_type",
    "takeaway_dish"."price" AS "dish_price",
    "takeaway_dish"."takeaway_shop_id" AS "dish_takeaway_shop_id",
    "takeaway_dish"."images" AS "dish_images",
    "route"."id" AS "stop_id",
    "route"."line_id" AS "stop_route_id",
    "route"."station_id" AS "route_station_id",
    "route"."arrival_time" AS "route_arrival_time",
    "route"."departure_time" AS "route_departure_time",
    "route"."order" AS "route_order"
FROM "takeaway_shop"
    INNER JOIN "takeaway_dish"
        ON "takeaway_shop"."id" = "takeaway_dish"."takeaway_shop_id"
    JOIN "route"
        ON "takeaway_shop"."station_id" = "route"."station_id"
WHERE "route"."line_id" = $1;"#,
            [route_id.to_db_value().into()],
        ))
        .all(&self.db)
        .await
        .inspect_err(|e| {
            error!("Failed to find takeaway shop by train route: {}", e);
        })
        .map_err(|e| RepositoryError::Db(e.into()))?;

        let mut result: HashMap<Stop, Vec<TakeawayShop>> = HashMap::new();

        let mut takeaway_shop_id_to_do_pack: HashMap<
            i32,
            (
                crate::models::takeaway_shop::Model,
                Vec<crate::models::takeaway_dish::Model>,
            ),
        > = HashMap::new();

        let mut station_id_to_stop: HashMap<i32, Stop> = HashMap::new();

        for data in &r {
            let takeaway_shop_do = crate::models::takeaway_shop::Model {
                id: data.shop_id,
                uuid: data.shop_uuid,
                name: data.shop_name.clone(),
                station_id: data.shop_station_id,
                images: data.shop_images.clone(),
            };

            let takeaway_dish_do = crate::models::takeaway_dish::Model {
                id: data.dish_id,
                name: data.dish_name.clone(),
                dish_type: data.dish_type.clone(),
                price: data.dish_price,
                takeaway_shop_id: data.dish_takeaway_shop_id,
                images: data.dish_images.clone(),
            };

            let entry = takeaway_shop_id_to_do_pack
                .entry(takeaway_shop_do.id)
                .or_insert((takeaway_shop_do, Vec::new()));

            entry.1.push(takeaway_dish_do);
        }

        for data in &r {
            let stop = Stop::new(
                Some(StopId::from_db_value(data.stop_id)?),
                Some(RouteId::from_db_value(data.stop_route_id)?),
                StationId::from_db_value(data.route_station_id)?,
                data.route_arrival_time as u32,
                data.route_departure_time as u32,
                data.route_order as u32,
            );

            station_id_to_stop.insert(data.route_station_id, stop);
        }

        for (_, (takeaway_shop_do, takeaway_dish_do_list)) in &takeaway_shop_id_to_do_pack {
            let takeaway_shop_do_pack = TakeawayShopDoPack {
                takeaway_shop_do: takeaway_shop_do.clone(),
                takeaway_shop_dish_do_list: takeaway_dish_do_list.clone(),
            };

            let takeaway_shop = TakeawayShopDataConverter::make_from_do(takeaway_shop_do_pack)?;

            let stop = station_id_to_stop
                .get(&takeaway_shop.station_id().to_db_value())
                .ok_or_else(|| {
                    RepositoryError::InconsistentState(anyhow!(
                        "Station ID {} not found in the result set",
                        takeaway_shop.station_id()
                    ))
                })?;

            result.entry(stop.clone()).or_default().push(takeaway_shop);
        }

        Ok(result)
    }

    async fn save_many_atomic(&self, entities: Vec<TakeawayShop>) -> Result<(), RepositoryError> {
        let tx = self
            .db
            .begin()
            .await
            .inspect_err(|e| {
                error!("Failed to begin transaction: {}", e);
            })
            .context("Failed to begin transaction")?;

        for entity in entities {
            let mut do_pack = TakeawayShopDataConverter::transform_to_do(&entity);

            let result = crate::models::takeaway_shop::Entity::insert(do_pack.takeaway_shop_do)
                .exec(&tx)
                .await
                .inspect_err(|e| {
                    error!("Failed to insert takeaway shop: {}", e);
                })
                .map_err(|e| RepositoryError::Db(e.into()))?;

            let takeaway_shop_id = result.last_insert_id;

            do_pack
                .takeaway_shop_dish_do_list
                .iter_mut()
                .for_each(|dish| dish.takeaway_shop_id = ActiveValue::Set(takeaway_shop_id));

            crate::models::takeaway_dish::Entity::insert_many(do_pack.takeaway_shop_dish_do_list)
                .exec(&tx)
                .await
                .inspect_err(|e| {
                    error!("Failed to insert takeaway dish: {}", e);
                })
                .map_err(|e| RepositoryError::Db(e.into()))?;
        }

        tx.commit()
            .await
            .inspect_err(|e| {
                error!("Failed to commit transaction: {}", e);
            })
            .context("Failed to commit transaction")?;

        Ok(())
    }

    async fn save_raw_takeaway<S: StationRepository, OS: ObjectStorageService>(
        &self,
        data: TakeawayData,
        data_path: &Path,
        station_repository: Arc<S>,
        object_storage_service: Arc<OS>,
    ) -> Result<(), RepositoryError> {
        let mut image_path_to_uuid: HashMap<String, Uuid> = HashMap::new();

        let station_list = station_repository
            .load()
            .await
            .inspect_err(|e| {
                error!("failed to get stations: {}", e);
            })
            .map_err(|e| RepositoryError::Db(e.into()))?;

        let station_name_to_id = station_list
            .iter()
            .map(|station| (station.name().to_string(), station.get_id().unwrap()))
            .collect::<HashMap<_, _>>();

        let mut entity_list = Vec::new();

        for (station_name, inner_map) in data {
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
                        let image_path = data_path.join(&takeaway.picture);

                        let image_data = fs::read(&image_path)
                            .context(format!("cannot read from: {:?}", &image_path))
                            .inspect_err(|e| {
                                error!("failed load takeaway image: {}", e);
                            })?;

                        let uuid = object_storage_service
                            .put_object(ObjectCategory::Takeaway, "image/jpeg", image_data)
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

        self.save_many_atomic(entity_list).await.inspect_err(|e| {
            error!("failed to save takeaway: {}", e);
        })?;

        Ok(())
    }
}
