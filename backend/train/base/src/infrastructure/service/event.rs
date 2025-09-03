use async_trait::async_trait;
use log::warn;
use sea_orm::sea_query::OnConflict;
use sea_orm::{ActiveValue, DatabaseConnection, EntityTrait};
use shared::MicroService;
use shared::event::queue::{EventService, EventServiceError, get_channel};
use shared::event::{EventRegistry, PersonalInfoUpdatedEvent, StationUpdatedEvent};
use shared::internal::geo::dto::DbStationDTO;
use shared::internal::user::dto::DbPersonalInfo;
use shared::ports::geo::GeoPort;
use shared::ports::user::UserPort;
use std::any::Any;
use std::sync::{Arc, Mutex};
use tracing::{error, instrument};

pub struct TrainEventServiceImpl<GP, UP>
where
    GP: GeoPort,
    UP: UserPort,
{
    channel: lapin::Channel,
    event_registry: Arc<Mutex<EventRegistry>>,
    db: DatabaseConnection,
    geo_port: Arc<GP>,
    user_port: Arc<UP>,
}

impl<GP, UP> TrainEventServiceImpl<GP, UP>
where
    GP: GeoPort,
    UP: UserPort,
{
    pub async fn new(
        addr: &str,
        db: DatabaseConnection,
        geo_port: Arc<GP>,
        user_port: Arc<UP>,
    ) -> Result<Arc<Self>, EventServiceError> {
        let channel = get_channel(addr).await?;

        let mut event_registry = EventRegistry::new();

        event_registry.register::<StationUpdatedEvent>();
        event_registry.register::<PersonalInfoUpdatedEvent>();

        Ok(Arc::new(Self {
            channel,
            db,
            event_registry: Arc::new(Mutex::new(event_registry)),
            geo_port,
            user_port,
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
    async fn handle_personal_info_updated_event(&self) {
        match self.user_port.db_get_personal_info().await {
            Ok(personal_info_list) => {
                update_personal_info(&self.db, personal_info_list).await;
            }
            Err(e) => {
                error!("Failed to get db personal info list: {:?}", e);
            }
        }
    }
}

#[async_trait]
impl<GP, UP> EventService for TrainEventServiceImpl<GP, UP>
where
    GP: GeoPort,
    UP: UserPort,
{
    fn micro_service(&self) -> MicroService {
        MicroService::Train
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
        if let Some(_) = event.downcast_ref::<StationUpdatedEvent>() {
            self.handle_station_updated_event().await;
        } else if let Some(_) = event.downcast_ref::<PersonalInfoUpdatedEvent>() {
            self.handle_personal_info_updated_event().await;
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
async fn update_personal_info(db: &DatabaseConnection, personal_info_list: Vec<DbPersonalInfo>) {
    let active_model_list = personal_info_list
        .into_iter()
        .map(|item| crate::models::person_info::ActiveModel {
            id: ActiveValue::Set(item.id),
            uuid: ActiveValue::Set(item.uuid),
            name: ActiveValue::Set(item.name),
            identity_card: ActiveValue::Set(item.identity_card),
            preferred_seat_location: ActiveValue::Set(item.preferred_seat_location),
            user_id: ActiveValue::Set(item.user_id),
            is_default: ActiveValue::Set(item.is_default),
        })
        .collect::<Vec<_>>();

    let insert_result = crate::models::person_info::Entity::insert_many(active_model_list)
        .on_conflict(
            OnConflict::column(crate::models::person_info::Column::Id)
                .update_columns([
                    crate::models::person_info::Column::Uuid,
                    crate::models::person_info::Column::Name,
                    crate::models::person_info::Column::IdentityCard,
                    crate::models::person_info::Column::PreferredSeatLocation,
                    crate::models::person_info::Column::UserId,
                    crate::models::person_info::Column::IsDefault,
                ])
                .to_owned(),
        )
        .exec(db)
        .await;

    if let Err(err) = insert_result {
        error!("Error while inserting personal info: {:?}", err);
    }
}
