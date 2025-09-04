use async_trait::async_trait;
use sea_orm::sea_query::OnConflict;
use sea_orm::{ActiveValue, DatabaseConnection, EntityTrait};
use shared::event::queue::{EventService, EventServiceError, get_channel};
use shared::event::{
    DishUpdatedEvent, EventRegistry, HotelRoomTypeUpdatedEvent, HotelUpdatedEvent,
    PersonalInfoUpdatedEvent, SeatTypeUpdatedEvent, StationUpdatedEvent, TakeawayDishUpdatedEvent,
    TrainScheduleUpdatedEvent, TrainUpdatedEvent, UserUpdatedEvent,
};
use shared::internal::dish::dto::{DbDishDTO, DbTakeawayDishDTO};
use shared::internal::geo::dto::DbStationDTO;
use shared::internal::hotel::dto::{DbHotelDTO, DbHotelRoomTypeDTO};
use shared::internal::train::dto::{DbSeatTypeDTO, DbTrainDTO, DbTrainScheduleDTO};
use shared::internal::user::dto::{DbPersonalInfo, DbUserDTO};
use shared::ports::dish::DishPort;
use shared::ports::geo::GeoPort;
use shared::ports::hotel::HotelPort;
use shared::ports::train::TrainPort;
use shared::ports::user::UserPort;
use shared::{DB_CHUNK_SIZE, MicroService};
use std::any::Any;
use std::sync::{Arc, Mutex};
use tracing::log::warn;
use tracing::{error, instrument};

pub struct OrderEventServiceImpl<UP, GP, TP, HP, DP>
where
    UP: UserPort,
    GP: GeoPort,
    TP: TrainPort,
    HP: HotelPort,
    DP: DishPort,
{
    channel: lapin::Channel,
    event_registry: Arc<Mutex<EventRegistry>>,
    db: DatabaseConnection,
    geo_port: Arc<GP>,
    user_port: Arc<UP>,
    train_port: Arc<TP>,
    hotel_port: Arc<HP>,
    dish_port: Arc<DP>,
}

impl<UP, GP, TP, HP, DP> OrderEventServiceImpl<UP, GP, TP, HP, DP>
where
    UP: UserPort,
    GP: GeoPort,
    TP: TrainPort,
    HP: HotelPort,
    DP: DishPort,
{
    pub async fn new(
        addr: &str,
        db: DatabaseConnection,
        geo_port: Arc<GP>,
        user_port: Arc<UP>,
        train_port: Arc<TP>,
        hotel_port: Arc<HP>,
        dish_port: Arc<DP>,
    ) -> Result<Arc<Self>, EventServiceError> {
        let channel = get_channel(addr).await?;

        let mut event_registry = EventRegistry::new();

        event_registry.register::<UserUpdatedEvent>();
        event_registry.register::<PersonalInfoUpdatedEvent>();
        event_registry.register::<StationUpdatedEvent>();
        event_registry.register::<TrainUpdatedEvent>();
        event_registry.register::<TrainScheduleUpdatedEvent>();
        event_registry.register::<SeatTypeUpdatedEvent>();
        event_registry.register::<HotelUpdatedEvent>();
        event_registry.register::<HotelRoomTypeUpdatedEvent>();
        event_registry.register::<DishUpdatedEvent>();
        event_registry.register::<TakeawayDishUpdatedEvent>();

        Ok(Arc::new(Self {
            channel,
            db,
            event_registry: Arc::new(Mutex::new(event_registry)),
            geo_port,
            user_port,
            train_port,
            hotel_port,
            dish_port,
        }))
    }

    #[instrument(skip(self))]
    async fn handle_user_updated_event(&self) {
        match self.user_port.db_get_user_info().await {
            Ok(user_list) => {
                update_user_info(&self.db, user_list).await;
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
                error!("Failed to get db trains: {:?}", e);
            }
        }
    }

    #[instrument(skip(self))]
    async fn handle_train_schedule_updated_event(&self) {
        match self.train_port.db_get_train_schedule().await {
            Ok(schedule_list) => {
                update_train_schedule(&self.db, schedule_list).await;
            }
            Err(e) => {
                error!("Failed to get db train schedules: {:?}", e);
            }
        }
    }

    #[instrument(skip(self))]
    async fn handle_seat_type_updated_event(&self) {
        match self.train_port.db_get_seat_type().await {
            Ok(seat_type_list) => {
                update_seat_type(&self.db, seat_type_list).await;
            }
            Err(e) => {
                error!("Failed to get db seat types: {:?}", e);
            }
        }
    }

    #[instrument(skip(self))]
    async fn handle_hotel_updated_event(&self) {
        match self.hotel_port.db_get_hotels().await {
            Ok(hotel_list) => {
                update_hotel(&self.db, hotel_list).await;
            }
            Err(e) => {
                error!("Failed to get db hotels: {:?}", e);
            }
        }
    }

    #[instrument(skip(self))]
    async fn handle_hotel_room_type_updated_event(&self) {
        match self.hotel_port.db_get_hotel_room_types().await {
            Ok(room_type_list) => {
                update_hotel_room_type(&self.db, room_type_list).await;
            }
            Err(e) => {
                error!("Failed to get db hotel room types: {:?}", e);
            }
        }
    }

    #[instrument(skip(self))]
    async fn handle_dish_updated_event(&self) {
        match self.dish_port.db_get_dishes().await {
            Ok(dish_list) => {
                update_dish(&self.db, dish_list).await;
            }
            Err(e) => {
                error!("Failed to get db dishes: {:?}", e);
            }
        }
    }

    #[instrument(skip(self))]
    async fn handle_takeaway_dish_updated_event(&self) {
        match self.dish_port.db_get_takeaway_dishes().await {
            Ok(takeaway_dish_list) => {
                update_takeaway_dish(&self.db, takeaway_dish_list).await;
            }
            Err(e) => {
                error!("Failed to get db takeaway dishes: {:?}", e);
            }
        }
    }
}

#[async_trait]
impl<UP, GP, TP, HP, DP> EventService for OrderEventServiceImpl<UP, GP, TP, HP, DP>
where
    UP: UserPort,
    GP: GeoPort,
    TP: TrainPort,
    HP: HotelPort,
    DP: DishPort,
{
    fn micro_service(&self) -> MicroService {
        MicroService::Order
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
        if event.downcast_ref::<UserUpdatedEvent>().is_some() {
            self.handle_user_updated_event().await;
        } else if event.downcast_ref::<PersonalInfoUpdatedEvent>().is_some() {
            self.handle_personal_info_updated_event().await;
        } else if event.downcast_ref::<StationUpdatedEvent>().is_some() {
            self.handle_station_updated_event().await;
        } else if event.downcast_ref::<TrainUpdatedEvent>().is_some() {
            self.handle_train_updated_event().await;
        } else if event.downcast_ref::<TrainScheduleUpdatedEvent>().is_some() {
            self.handle_train_schedule_updated_event().await;
        } else if event.downcast_ref::<SeatTypeUpdatedEvent>().is_some() {
            self.handle_seat_type_updated_event().await;
        } else if event.downcast_ref::<HotelUpdatedEvent>().is_some() {
            self.handle_hotel_updated_event().await;
        } else if event.downcast_ref::<HotelRoomTypeUpdatedEvent>().is_some() {
            self.handle_hotel_room_type_updated_event().await;
        } else if event.downcast_ref::<DishUpdatedEvent>().is_some() {
            self.handle_dish_updated_event().await;
        } else if event.downcast_ref::<TakeawayDishUpdatedEvent>().is_some() {
            self.handle_takeaway_dish_updated_event().await;
        } else {
            warn!("Unknown event type");
        }

        Ok(())
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
        let insert_result = crate::models::train::Entity::insert_many(chunk)
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
async fn update_train_schedule(db: &DatabaseConnection, schedule_list: Vec<DbTrainScheduleDTO>) {
    let active_model_list = schedule_list
        .into_iter()
        .map(|item| crate::models::train_schedule::ActiveModel {
            id: ActiveValue::Set(item.id),
            train_id: ActiveValue::Set(item.train_id),
            departure_date: ActiveValue::Set(item.departure_date),
            origin_departure_time: ActiveValue::Set(item.origin_departure_time),
            line_id: ActiveValue::Set(item.line_id),
        })
        .collect::<Vec<_>>();

    for chunk in active_model_list.chunks(DB_CHUNK_SIZE) {
        let insert_result = crate::models::train_schedule::Entity::insert_many(chunk)
            .on_conflict(
                OnConflict::column(crate::models::train_schedule::Column::Id)
                    .update_columns([
                        crate::models::train_schedule::Column::TrainId,
                        crate::models::train_schedule::Column::DepartureDate,
                        crate::models::train_schedule::Column::OriginDepartureTime,
                        crate::models::train_schedule::Column::LineId,
                    ])
                    .to_owned(),
            )
            .exec(db)
            .await;

        if let Err(err) = insert_result {
            error!("Error while inserting train schedule: {:?}", err);
        }
    }
}

#[instrument(skip_all)]
async fn update_seat_type(db: &DatabaseConnection, seat_type_list: Vec<DbSeatTypeDTO>) {
    let active_model_list = seat_type_list
        .into_iter()
        .map(|item| crate::models::seat_type::ActiveModel {
            id: ActiveValue::Set(item.id),
            type_name: ActiveValue::Set(item.type_name),
            capacity: ActiveValue::Set(item.capacity),
            price: ActiveValue::Set(item.price),
        })
        .collect::<Vec<_>>();

    let insert_result = crate::models::seat_type::Entity::insert_many(active_model_list)
        .on_conflict(
            OnConflict::column(crate::models::seat_type::Column::Id)
                .update_columns([
                    crate::models::seat_type::Column::TypeName,
                    crate::models::seat_type::Column::Capacity,
                    crate::models::seat_type::Column::Price,
                ])
                .to_owned(),
        )
        .exec(db)
        .await;

    if let Err(err) = insert_result {
        error!("Error while inserting seat type: {:?}", err);
    }
}

#[instrument(skip_all)]
async fn update_hotel(db: &DatabaseConnection, hotel_list: Vec<DbHotelDTO>) {
    let active_model_list = hotel_list
        .into_iter()
        .map(|item| crate::models::hotel::ActiveModel {
            id: ActiveValue::Set(item.id),
            uuid: ActiveValue::Set(item.uuid),
            name: ActiveValue::Set(item.name),
            city_id: ActiveValue::Set(item.city_id),
            station_id: ActiveValue::Set(item.station_id),
            address: ActiveValue::Set(item.address),
            phone: ActiveValue::Set(item.phone),
            images: ActiveValue::Set(item.images),
            total_rating_count: ActiveValue::Set(item.total_rating_count),
            total_booking_count: ActiveValue::Set(item.total_booking_count),
            info: ActiveValue::Set(item.info),
        })
        .collect::<Vec<_>>();

    let insert_result = crate::models::hotel::Entity::insert_many(active_model_list)
        .on_conflict(
            OnConflict::column(crate::models::hotel::Column::Id)
                .update_columns([
                    crate::models::hotel::Column::Uuid,
                    crate::models::hotel::Column::Name,
                    crate::models::hotel::Column::CityId,
                    crate::models::hotel::Column::StationId,
                    crate::models::hotel::Column::Address,
                    crate::models::hotel::Column::Phone,
                    crate::models::hotel::Column::Images,
                    crate::models::hotel::Column::TotalRatingCount,
                    crate::models::hotel::Column::TotalBookingCount,
                    crate::models::hotel::Column::Info,
                ])
                .to_owned(),
        )
        .exec(db)
        .await;

    if let Err(err) = insert_result {
        error!("Error while inserting hotel: {:?}", err);
    }
}

#[instrument(skip_all)]
async fn update_hotel_room_type(db: &DatabaseConnection, room_type_list: Vec<DbHotelRoomTypeDTO>) {
    let active_model_list = room_type_list
        .into_iter()
        .map(|item| crate::models::hotel_room_type::ActiveModel {
            id: ActiveValue::Set(item.id),
            type_name: ActiveValue::Set(item.type_name),
            capacity: ActiveValue::Set(item.capacity),
            price: ActiveValue::Set(item.price),
            hotel_id: ActiveValue::Set(item.hotel_id),
        })
        .collect::<Vec<_>>();

    let insert_result = crate::models::hotel_room_type::Entity::insert_many(active_model_list)
        .on_conflict(
            OnConflict::column(crate::models::hotel_room_type::Column::Id)
                .update_columns([
                    crate::models::hotel_room_type::Column::TypeName,
                    crate::models::hotel_room_type::Column::Capacity,
                    crate::models::hotel_room_type::Column::Price,
                    crate::models::hotel_room_type::Column::HotelId,
                ])
                .to_owned(),
        )
        .exec(db)
        .await;

    if let Err(err) = insert_result {
        error!("Error while inserting hotel room type: {:?}", err);
    }
}

#[instrument(skip_all)]
async fn update_dish(db: &DatabaseConnection, dish_list: Vec<DbDishDTO>) {
    let active_model_list = dish_list
        .into_iter()
        .map(|item| crate::models::dish::ActiveModel {
            id: ActiveValue::Set(item.id),
            train_id: ActiveValue::Set(item.train_id),
            r#type: ActiveValue::Set(item.r#type),
            time: ActiveValue::Set(item.time),
            name: ActiveValue::Set(item.name),
            price: ActiveValue::Set(item.price),
            images: ActiveValue::Set(item.images),
        })
        .collect::<Vec<_>>();

    let insert_result = crate::models::dish::Entity::insert_many(active_model_list)
        .on_conflict(
            OnConflict::column(crate::models::dish::Column::Id)
                .update_columns([
                    crate::models::dish::Column::TrainId,
                    crate::models::dish::Column::Type,
                    crate::models::dish::Column::Time,
                    crate::models::dish::Column::Name,
                    crate::models::dish::Column::Price,
                    crate::models::dish::Column::Images,
                ])
                .to_owned(),
        )
        .exec(db)
        .await;

    if let Err(err) = insert_result {
        error!("Error while inserting dish: {:?}", err);
    }
}

#[instrument(skip_all)]
async fn update_takeaway_dish(db: &DatabaseConnection, takeaway_dish_list: Vec<DbTakeawayDishDTO>) {
    let active_model_list = takeaway_dish_list
        .into_iter()
        .map(|item| crate::models::takeaway_dish::ActiveModel {
            id: ActiveValue::Set(item.id),
            name: ActiveValue::Set(item.name),
            dish_type: ActiveValue::Set(item.dish_type),
            price: ActiveValue::Set(item.price),
            takeaway_shop_id: ActiveValue::Set(item.takeaway_shop_id),
            images: ActiveValue::Set(item.images),
        })
        .collect::<Vec<_>>();

    let insert_result = crate::models::takeaway_dish::Entity::insert_many(active_model_list)
        .on_conflict(
            OnConflict::column(crate::models::takeaway_dish::Column::Id)
                .update_columns([
                    crate::models::takeaway_dish::Column::Name,
                    crate::models::takeaway_dish::Column::DishType,
                    crate::models::takeaway_dish::Column::Price,
                    crate::models::takeaway_dish::Column::TakeawayShopId,
                    crate::models::takeaway_dish::Column::Images,
                ])
                .to_owned(),
        )
        .exec(db)
        .await;

    if let Err(err) = insert_result {
        error!("Error while inserting takeaway dish: {:?}", err);
    }
}
