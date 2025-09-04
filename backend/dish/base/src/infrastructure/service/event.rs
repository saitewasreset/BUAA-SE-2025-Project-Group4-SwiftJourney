use async_trait::async_trait;
use sea_orm::sea_query::OnConflict;
use sea_orm::{ActiveValue, DatabaseConnection, EntityTrait};
use shared::event::queue::{EventService, EventServiceError, get_channel};
use shared::event::{EventRegistry, RouteUpdatedEvent, StationUpdatedEvent, TrainUpdatedEvent};
use shared::internal::geo::dto::DbStationDTO;
use shared::internal::train::dto::{DbRouteDTO, DbTrainDTO};
use shared::ports::geo::GeoPort;
use shared::ports::train::TrainPort;
use shared::{DB_CHUNK_SIZE, MicroService};
use std::any::Any;
use std::sync::{Arc, Mutex};
use tracing::log::warn;
use tracing::{error, instrument};

pub struct DishEventServiceImpl<GP, TP>
where
    GP: GeoPort,
    TP: TrainPort,
{
    channel: lapin::Channel,
    event_registry: Arc<Mutex<EventRegistry>>,
    db: DatabaseConnection,
    geo_port: Arc<GP>,
    train_port: Arc<TP>,
}

impl<GP, TP> DishEventServiceImpl<GP, TP>
where
    GP: GeoPort,
    TP: TrainPort,
{
    pub async fn new(
        addr: &str,
        db: DatabaseConnection,
        geo_port: Arc<GP>,
        train_port: Arc<TP>,
    ) -> Result<Arc<Self>, EventServiceError> {
        let channel = get_channel(addr).await?;

        let mut event_registry = EventRegistry::new();

        event_registry.register::<TrainUpdatedEvent>();
        event_registry.register::<StationUpdatedEvent>();
        event_registry.register::<RouteUpdatedEvent>();

        Ok(Arc::new(Self {
            channel,
            db,
            event_registry: Arc::new(Mutex::new(event_registry)),
            geo_port,
            train_port,
        }))
    }

    #[instrument(skip(self))]
    async fn handle_station_updated_event(&self) {
        match self.geo_port.db_get_stations().await {
            Ok(station_list) => {
                update_station(&self.db, station_list).await;
            }
            Err(e) => {
                error!("Failed to get db stations: {:?}", e);
            }
        }
    }

    #[instrument(skip(self))]
    async fn handle_train_updated_event(&self) {
        match self.train_port.db_get_trains().await {
            Ok(train_list) => {
                update_train(&self.db, train_list).await;
            }
            Err(e) => {
                error!("Failed to get train list: {:?}", e);
            }
        }
    }

    #[instrument(skip(self))]
    async fn handle_route_updated_event(&self) {
        match self.train_port.db_get_routes().await {
            Ok(route_list) => {
                update_route(&self.db, route_list).await;
            }
            Err(e) => {
                error!("Failed to get db route list: {:?}", e);
            }
        }
    }
}

#[async_trait]
impl<GP, TP> EventService for DishEventServiceImpl<GP, TP>
where
    GP: GeoPort,
    TP: TrainPort,
{
    fn micro_service(&self) -> MicroService {
        MicroService::Dish
    }

    fn lapin_channel(&self) -> lapin::Channel {
        self.channel.clone()
    }

    fn event_registry(&self) -> Arc<Mutex<EventRegistry>> {
        Arc::clone(&self.event_registry)
    }

    #[instrument(skip_all)]
    async fn handle_event(
        &self,
        event: Box<dyn Any + Send + Sync>,
    ) -> Result<(), EventServiceError> {
        if event.downcast_ref::<StationUpdatedEvent>().is_some() {
            self.handle_station_updated_event().await;
        } else if event.downcast_ref::<TrainUpdatedEvent>().is_some() {
            self.handle_train_updated_event().await;
        } else if event.downcast_ref::<RouteUpdatedEvent>().is_some() {
            self.handle_route_updated_event().await;
        } else {
            warn!("Unknown event type");
        }

        Ok(())
    }
}

#[instrument(skip_all)]
async fn update_station(db: &DatabaseConnection, station_list: Vec<DbStationDTO>) {
    let active_model_list = station_list
        .into_iter()
        .map(|item| crate::models::station::ActiveModel {
            id: ActiveValue::Set(item.id),
            name: ActiveValue::Set(item.name),
            city_id: ActiveValue::Set(item.city_id),
        })
        .collect::<Vec<_>>();

    let insert_result = crate::models::station::Entity::insert_many(active_model_list)
        .on_conflict(
            OnConflict::column(crate::models::station::Column::Id)
                .update_columns([
                    crate::models::station::Column::Name,
                    crate::models::station::Column::CityId,
                ])
                .to_owned(),
        )
        .exec(db)
        .await;

    if let Err(err) = insert_result {
        error!("Error while inserting station: {:?}", err);
    }
}

#[instrument(skip_all)]
async fn update_train(db: &DatabaseConnection, train_list: Vec<DbTrainDTO>) {
    let active_model_list = train_list
        .into_iter()
        .map(|item| crate::models::train::ActiveModel {
            id: ActiveValue::Set(item.id),
            number: ActiveValue::Set(item.number),
            type_id: ActiveValue::Set(item.type_id),
            default_origin_departure_time: ActiveValue::Set(item.default_origin_departure_time),
            default_line_id: ActiveValue::Set(item.default_line_id),
        })
        .collect::<Vec<_>>();

    for chunk in active_model_list.chunks(DB_CHUNK_SIZE) {
        let insert_result = crate::models::train::Entity::insert_many(chunk.to_vec())
            .on_conflict(
                OnConflict::column(crate::models::train::Column::Id)
                    .update_columns([
                        crate::models::train::Column::Number,
                        crate::models::train::Column::TypeId,
                        crate::models::train::Column::DefaultOriginDepartureTime,
                        crate::models::train::Column::DefaultLineId,
                    ])
                    .to_owned(),
            )
            .exec(db)
            .await;

        if let Err(err) = insert_result {
            error!("Error while inserting trains: {:?}", err);
        }
    }
}

#[instrument(skip_all)]
async fn update_route(db: &DatabaseConnection, route_list: Vec<DbRouteDTO>) {
    let active_model_list = route_list
        .into_iter()
        .map(|item| crate::models::route::ActiveModel {
            id: ActiveValue::Set(item.id),
            line_id: ActiveValue::Set(item.line_id),
            station_id: ActiveValue::Set(item.station_id),
            arrival_time: ActiveValue::Set(item.arrival_time),
            departure_time: ActiveValue::Set(item.departure_time),
            order: ActiveValue::Set(item.order),
        })
        .collect::<Vec<_>>();

    for chunk in active_model_list.chunks(DB_CHUNK_SIZE) {
        let insert_result = crate::models::route::Entity::insert_many(chunk.to_vec())
            .on_conflict(
                OnConflict::column(crate::models::route::Column::Id)
                    .update_columns([
                        crate::models::route::Column::Id,
                        crate::models::route::Column::LineId,
                        crate::models::route::Column::StationId,
                        crate::models::route::Column::ArrivalTime,
                        crate::models::route::Column::DepartureTime,
                        crate::models::route::Column::Order,
                    ])
                    .to_owned(),
            )
            .exec(db)
            .await;

        if let Err(err) = insert_result {
            error!("Error while inserting route: {:?}", err);
        }
    }
}
