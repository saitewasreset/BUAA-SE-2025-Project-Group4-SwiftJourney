use crate::domain::model::route::{Route, RouteId, StopId};
use crate::domain::model::station::StationId;
use crate::domain::repository::route::RouteRepository;
use crate::domain::{DbId, Identifiable, Repository, RepositoryError};
use anyhow::{Context, anyhow};
use async_trait::async_trait;
use sea_orm::{ActiveValue, DatabaseConnection};
use sea_orm::{ColumnTrait, TransactionTrait};
use sea_orm::{DatabaseTransaction, QueryFilter};
use sea_orm::{EntityTrait, QueryOrder};
use shared::data::RouteStationInfo;
use std::collections::HashMap;
use tracing::{error, instrument, trace};

impl_db_id_from_u64!(RouteId, i64, "route");
impl_db_id_from_u64!(StopId, i32, "stop");

pub struct RouteDataConverter;

impl RouteDataConverter {
    #[instrument(skip_all)]
    pub fn make_from_do(
        route_model_list: Vec<crate::models::route::Model>,
    ) -> Result<Route, anyhow::Error> {
        if route_model_list.is_empty() {
            error!("Route model list is empty");
            return Err(anyhow::anyhow!("Route model list is empty"));
        }

        let route_id = route_model_list[0].line_id;

        for stop in &route_model_list {
            if stop.line_id != route_id {
                error!(
                    "Inconsistent: Route ID mismatch, current: {}, first: {}",
                    stop.line_id, route_id
                );
                return Err(anyhow::anyhow!("Inconsistent: Route ID mismatch"));
            }
        }

        let route_id = RouteId::from_db_value(route_id)?;

        let mut route = Route::new(Some(route_id));

        for stop in route_model_list {
            route.add_stop(
                Some(StopId::from_db_value(stop.id)?),
                StationId::from_db_value(stop.station_id)?,
                stop.arrival_time as u32,
                stop.departure_time as u32,
                stop.order as u32,
            );
        }

        Ok(route)
    }

    #[instrument]
    pub fn transform_to_do(route: Route) -> Vec<crate::models::route::ActiveModel> {
        let route_id = route.get_id();

        let mut result = Vec::new();

        for stop in route.into_stops() {
            let stop_id = stop.get_id();
            let station_id = stop.station_id();
            let arrival_time = stop.arrival_time();
            let departure_time = stop.departure_time();
            let order = stop.order();

            // Convert to ActiveModel
            let mut model = crate::models::route::ActiveModel {
                id: ActiveValue::NotSet,
                line_id: ActiveValue::NotSet,
                station_id: ActiveValue::Set(station_id.to_db_value()),
                arrival_time: ActiveValue::Set(arrival_time as i32),
                departure_time: ActiveValue::Set(departure_time as i32),
                order: ActiveValue::Set(order as i32),
            };

            if let Some(stop_id) = stop_id {
                model.id = ActiveValue::Set(stop_id.to_db_value());
            }

            result.push(model);
        }

        if let Some(route_id) = route_id {
            for model in &mut result {
                model.line_id = ActiveValue::Set(route_id.to_db_value());
            }
        }

        result
    }
}

pub struct RouteRepositoryImpl {
    db: DatabaseConnection,
}

impl RouteRepositoryImpl {
    pub fn new(db: DatabaseConnection) -> Self {
        RouteRepositoryImpl { db }
    }

    #[instrument(skip_all)]
    pub async fn get_new_line_id(&self, txn: &DatabaseTransaction) -> Result<i64, RepositoryError> {
        let last_line = crate::models::route::Entity::find()
            .order_by_desc(crate::models::route::Column::LineId)
            .one(txn)
            .await
            .context("failed to get last line id")
            .map_err(|e| {
                error!("Failed to get last line id: {}", e);
                e
            })?;

        Ok(last_line.map(|model| model.line_id + 1).unwrap_or(1))
    }
}

#[async_trait]
impl Repository<Route> for RouteRepositoryImpl {
    #[instrument(skip(self))]
    async fn find(&self, id: RouteId) -> Result<Option<Route>, RepositoryError> {
        let model = crate::models::route::Entity::find()
            .filter(crate::models::route::Column::LineId.eq(id.to_db_value()))
            .all(&self.db)
            .await
            .context(format!(
                "failed to find route with id: {}",
                id.to_db_value()
            ))
            .map_err(|e| {
                error!("Failed to find route with id: {}: {}", id.to_db_value(), e);
                e
            })?;

        if !model.is_empty() {
            let route = RouteDataConverter::make_from_do(model)
                .context(format!(
                    "failed to convert route with id: {}",
                    id.to_db_value()
                ))
                .map_err(|e| {
                    error!(
                        "Failed to convert route with id: {}: {}",
                        id.to_db_value(),
                        e
                    );
                    e
                })
                .map_err(RepositoryError::ValidationError)?;
            Ok(Some(route))
        } else {
            Ok(None)
        }
    }

    #[instrument(skip(self))]
    async fn remove(&self, aggregate: Route) -> Result<(), RepositoryError> {
        if let Some(route_id) = aggregate.get_id() {
            trace!("Begin Transaction");

            let txn = self
                .db
                .begin()
                .await
                .context("Failed to start transaction")
                .map_err(|e| {
                    error!("Failed to start transaction: {}", e);
                    e
                })?;

            crate::models::route::Entity::delete_many()
                .filter(crate::models::route::Column::LineId.eq(route_id.to_db_value()))
                .exec(&txn)
                .await
                .context(format!(
                    "failed to delete route with id: {}",
                    route_id.to_db_value()
                ))
                .map_err(|e| {
                    error!(
                        "Failed to delete route with id: {}: {}",
                        route_id.to_db_value(),
                        e
                    );
                    e
                })?;

            trace!("Commit Transaction");
            txn.commit().await.context("Failed to commit transaction")?;
        }

        Ok(())
    }

    #[instrument(skip(self))]
    async fn save(&self, aggregate: &mut Route) -> Result<RouteId, RepositoryError> {
        trace!("Begin Transaction");
        let txn = self
            .db
            .begin()
            .await
            .context("Failed to start transaction")
            .map_err(|e| {
                error!("Failed to start transaction: {}", e);
                e
            })?;

        let models = RouteDataConverter::transform_to_do(aggregate.clone());

        let last_line_id = self.get_new_line_id(&txn).await?;

        crate::models::route::Entity::insert_many(models)
            .exec(&txn)
            .await
            .context(format!("failed to insert route with id: {}", last_line_id))
            .map_err(|e| {
                error!("Failed to insert route with id: {}: {}", last_line_id, e);
                e
            })?;

        trace!("Commit Transaction");
        txn.commit().await.context("Failed to commit transaction")?;

        *aggregate = self
            .find(RouteId::from_db_value(last_line_id)?)
            .await?
            .expect("should found already inserted route");

        Ok(aggregate
            .get_id()
            .expect("already inserted route should have id"))
    }
}

#[async_trait]
impl RouteRepository for RouteRepositoryImpl {
    #[instrument(skip_all)]
    async fn load(&self) -> Result<Vec<Route>, RepositoryError> {
        let route_models = crate::models::route::Entity::find()
            .all(&self.db)
            .await
            .context("failed to load routes")
            .map_err(|e| {
                error!("Failed to load routes: {}", e);
                e
            })?;

        let mut route_id_to_stops: HashMap<i64, Vec<crate::models::route::Model>> = HashMap::new();

        for model in route_models {
            route_id_to_stops
                .entry(model.line_id)
                .or_default()
                .push(model);
        }

        let mut result = Vec::with_capacity(route_id_to_stops.len());

        for v in route_id_to_stops.into_values() {
            result.push(
                RouteDataConverter::make_from_do(v)
                    .map_err(RepositoryError::ValidationError)
                    .map_err(|e| {
                        error!("Failed to convert route: {}", e);
                        e
                    })?,
            );
        }

        Ok(result)
    }

    #[instrument(skip_all)]
    async fn save_raw(
        &self,
        raw_routes: Vec<RouteStationInfo>,
    ) -> Result<RouteId, RepositoryError> {
        trace!("Begin Transaction");
        let txn = self
            .db
            .begin()
            .await
            .context("Failed to start transaction")
            .map_err(|e| {
                error!("Failed to start transaction: {}", e);
                e
            })?;

        let new_line_id = self.get_new_line_id(&txn).await?;

        let stations = crate::models::station::Entity::find()
            .all(&txn)
            .await
            .context("Failed to load stations")
            .map_err(|e| {
                error!("Failed to load stations: {}", e);
                e
            })?;

        let station_name_to_id = stations
            .into_iter()
            .map(|s| (s.name, s.id))
            .collect::<HashMap<_, _>>();

        let mut model_list = Vec::new();

        for stop in raw_routes {
            let station_id = station_name_to_id
                .get(&stop.station)
                .copied()
                .ok_or(RepositoryError::InconsistentState(anyhow!(
                    "Station not found: {}",
                    stop.station
                )))
                .map_err(|e| {
                    error!("Failed to find station: {}: {}", stop.station, e);
                    e
                })?;

            let model = crate::models::route::ActiveModel {
                id: ActiveValue::NotSet,
                line_id: ActiveValue::Set(new_line_id),
                station_id: ActiveValue::Set(station_id),
                arrival_time: ActiveValue::Set(stop.arrival_time as i32),
                departure_time: ActiveValue::Set(stop.departure_time as i32),
                order: ActiveValue::Set(stop.order as i32),
            };

            model_list.push(model);
        }

        crate::models::route::Entity::insert_many(model_list)
            .exec(&txn)
            .await
            .context("Failed to insert routes")
            .map_err(|e| {
                error!("Failed to insert routes: {}", e);
                e
            })?;

        trace!("Commit Transaction");
        txn.commit().await.context("Failed to commit transaction")?;

        Ok(RouteId::from_db_value(new_line_id)?)
    }
}
