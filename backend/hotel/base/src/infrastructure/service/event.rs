use async_trait::async_trait;
use sea_orm::sea_query::OnConflict;
use sea_orm::{ActiveValue, DatabaseConnection, EntityTrait};
use shared::MicroService;
use shared::event::queue::{EventService, EventServiceError, get_channel};
use shared::event::{
    CityUpdatedEvent, EventRegistry, PersonalInfoUpdatedEvent, StationUpdatedEvent,
    UserUpdatedEvent,
};
use shared::internal::geo::dto::{DbCityDTO, DbStationDTO};
use shared::internal::user::dto::{DbPersonalInfo, DbUserDTO};
use shared::ports::geo::GeoPort;
use shared::ports::user::UserPort;
use std::any::Any;
use std::sync::{Arc, Mutex};
use tracing::log::warn;
use tracing::{error, instrument};

pub struct HotelEventServiceImpl<GP, UP>
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

impl<GP, UP> HotelEventServiceImpl<GP, UP>
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

        event_registry.register::<CityUpdatedEvent>();
        event_registry.register::<StationUpdatedEvent>();
        event_registry.register::<UserUpdatedEvent>();
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
    async fn handle_city_updated_event(&self) {
        match self.geo_port.db_get_cities().await {
            Ok(station_list) => {
                update_city(&self.db, station_list).await;
            }
            Err(e) => {
                error!("Failed to get db cities: {:?}", e);
            }
        }
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
    async fn handle_user_updated_event(&self) {
        match self.user_port.db_get_user_info().await {
            Ok(personal_info_list) => {
                update_user_info(&self.db, personal_info_list).await;
            }
            Err(e) => {
                error!("Failed to get db user info list: {:?}", e);
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
impl<GP, UP> EventService for HotelEventServiceImpl<GP, UP>
where
    GP: GeoPort,
    UP: UserPort,
{
    fn micro_service(&self) -> MicroService {
        MicroService::Hotel
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
        } else if event.downcast_ref::<CityUpdatedEvent>().is_some() {
            self.handle_city_updated_event().await;
        } else if event.downcast_ref::<UserUpdatedEvent>().is_some() {
            self.handle_user_updated_event().await;
        } else if event.downcast_ref::<PersonalInfoUpdatedEvent>().is_some() {
            self.handle_personal_info_updated_event().await;
        } else {
            warn!("Unknown event type");
        }

        Ok(())
    }
}

#[instrument(skip_all)]
async fn update_city(db: &DatabaseConnection, city_list: Vec<DbCityDTO>) {
    let active_model_list = city_list
        .into_iter()
        .map(|item| crate::models::city::ActiveModel {
            id: ActiveValue::Set(item.id),
            name: ActiveValue::Set(item.name),
            province: ActiveValue::Set(item.province),
        })
        .collect::<Vec<_>>();

    let insert_result = crate::models::city::Entity::insert_many(active_model_list)
        .on_conflict(
            OnConflict::column(crate::models::city::Column::Id)
                .update_columns([
                    crate::models::city::Column::Name,
                    crate::models::city::Column::Province,
                ])
                .to_owned(),
        )
        .exec(db)
        .await;

    if let Err(err) = insert_result {
        error!("Error while inserting city: {:?}", err);
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

#[instrument(skip_all)]
async fn update_user_info(db: &DatabaseConnection, user_info_list: Vec<DbUserDTO>) {
    let active_model_list = user_info_list
        .into_iter()
        .map(|item| crate::models::user::ActiveModel {
            id: ActiveValue::Set(item.id),
            username: ActiveValue::Set(item.username),
            hashed_password: ActiveValue::Set(item.hashed_password),
            hashed_payment_password: ActiveValue::Set(item.hashed_payment_password),
            salt: ActiveValue::Set(item.salt),
            wrong_payment_password_tried: ActiveValue::Set(item.wrong_payment_password_tried),
            gender: ActiveValue::Set(item.gender),
            age: ActiveValue::Set(item.age),
            phone: ActiveValue::Set(item.phone),
            email: ActiveValue::Set(item.email),
            name: ActiveValue::Set(item.name),
            identity_card_id: ActiveValue::Set(item.identity_card_id),
        })
        .collect::<Vec<_>>();

    let insert_result = crate::models::user::Entity::insert_many(active_model_list)
        .on_conflict(
            OnConflict::column(crate::models::user::Column::Id)
                .update_columns([
                    crate::models::user::Column::Username,
                    crate::models::user::Column::HashedPassword,
                    crate::models::user::Column::HashedPaymentPassword,
                    crate::models::user::Column::Salt,
                    crate::models::user::Column::WrongPaymentPasswordTried,
                    crate::models::user::Column::Gender,
                    crate::models::user::Column::Age,
                    crate::models::user::Column::Phone,
                    crate::models::user::Column::Email,
                    crate::models::user::Column::Name,
                    crate::models::user::Column::IdentityCardId,
                ])
                .to_owned(),
        )
        .exec(db)
        .await;

    if let Err(err) = insert_result {
        error!("Error while inserting user info: {:?}", err);
    }
}
