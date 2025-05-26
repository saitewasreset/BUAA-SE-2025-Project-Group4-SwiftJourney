use crate::application::service::message::TripNotifyDTO;
use crate::domain::model::message::{Notify, NotifyId, NotifyType, OrderNotify, TripNotify};
use crate::domain::model::user::UserId;
use crate::domain::repository::notify::NotifyRepository;
use crate::domain::{DbId, RepositoryError};
use anyhow::anyhow;
use async_trait::async_trait;
use sea_orm::ColumnTrait;
use sea_orm::{ActiveValue, DatabaseConnection, EntityTrait, QueryFilter};
use std::any::{Any, TypeId};
use tracing::{error, instrument};

impl_db_id_from_u64!(NotifyId, i32, "notify id");

pub struct NotifyRepositoryImpl {
    db: DatabaseConnection,
}

impl NotifyRepositoryImpl {
    pub fn new(db: DatabaseConnection) -> Self {
        NotifyRepositoryImpl { db }
    }
}

mod ser_order {
    use crate::Verified;
    use crate::domain::model::dish::DishId;
    use crate::domain::model::hotel::{HotelDateRange, HotelId, HotelRoomTypeId};
    use crate::domain::model::order::{
        BaseOrder, DishOrder, HotelOrder, Order, OrderId, OrderStatus, OrderTimeInfo, PaymentInfo,
        TakeawayOrder, TrainOrder,
    };
    use crate::domain::model::personal_info::PersonalInfoId;
    use crate::domain::model::station::StationId;
    use crate::domain::model::takeaway::TakeawayDishId;
    use crate::domain::model::train::{SeatType, SeatTypeId, SeatTypeName};
    use crate::domain::model::train_schedule::{
        Seat, SeatLocationInfo, SeatStatus, StationRange, TrainScheduleId,
    };
    use crate::domain::model::transaction::TransactionId;
    use crate::domain::{DbId, Identifiable};
    use anyhow::anyhow;
    use chrono::NaiveDate;
    use rust_decimal::Decimal;
    use sea_orm::prelude::DateTimeWithTimeZone;
    use serde::{Deserialize, Serialize};
    use std::any::Any;
    use std::str::FromStr;
    use uuid::Uuid;

    #[derive(Serialize, Deserialize)]
    pub struct StationRangeDTO {
        pub from_station_id: i32,
        pub to_station_id: i32,
    }

    // DTO 结构体定义
    #[derive(Serialize, Deserialize)]
    pub struct HotelDateRangeDTO {
        pub begin_date: NaiveDate,
        pub end_date: NaiveDate,
    }

    #[derive(Serialize, Deserialize)]
    pub struct SeatLocationInfoDTO {
        pub carriage: i32,  // 车厢号(如3)
        pub row: i32,       // 排数(如11)
        pub location: char, // 位置标记(如'A')
    }

    #[derive(Serialize, Deserialize)]
    pub struct SeatTypeDTO {
        pub seat_type_id: i32,
        pub type_name: String,
        pub capacity: u32,
        pub price: Decimal,
    }

    #[derive(Serialize, Deserialize)]
    pub struct SeatDTO {
        pub id: i64,
        pub seat_type: SeatTypeDTO,
        pub info: SeatLocationInfoDTO,
        pub status: String,
    }

    #[derive(Serialize, Deserialize)]
    pub struct OrderTimeInfoDto {
        pub create_time: String,
        pub active_time: String,
        pub complete_time: String,
    }

    #[derive(Serialize, Deserialize)]
    pub struct PaymentInfoDto {
        pub pay_transaction_id: Option<i32>,
        pub refund_transaction_id: Option<i32>,
    }

    #[derive(Serialize, Deserialize)]
    pub struct BaseOrderDto {
        pub order_id: i32,
        pub uuid: Uuid,
        pub order_status: String,
        pub order_time_info: OrderTimeInfoDto,
        pub unit_price: Decimal,
        pub amount: Decimal,
        pub payment_info: PaymentInfoDto,
        pub personal_info_id: i32,
    }

    #[derive(Serialize, Deserialize)]
    pub struct TrainOrderDto {
        pub base: BaseOrderDto,
        pub train_schedule_id: i32,
        pub seat: SeatDTO,
        pub station_range: StationRangeDTO,
    }

    #[derive(Serialize, Deserialize)]
    pub struct HotelOrderDto {
        pub base: BaseOrderDto,
        pub hotel_id: i32,
        pub room_id: i32,
        pub booking_date_range: HotelDateRangeDTO,
    }

    #[derive(Serialize, Deserialize)]
    pub struct DishOrderDto {
        pub base: BaseOrderDto,
        pub train_order_id: i32,
        pub dish_id: i32,
        pub unit_price: String,
        pub amount: String,
    }

    #[derive(Serialize, Deserialize)]
    pub struct TakeawayOrderDto {
        pub base: BaseOrderDto,
        pub train_order_id: i32,
        pub takeaway_dish_id: i32,
        pub unit_price: String,
        pub amount: String,
    }

    #[derive(Serialize, Deserialize)]
    #[serde(tag = "type", content = "data")]
    pub enum OrderDto {
        Train(TrainOrderDto),
        Hotel(HotelOrderDto),
        Dish(DishOrderDto),
        Takeaway(TakeawayOrderDto),
    }

    impl From<HotelDateRange> for HotelDateRangeDTO {
        fn from(range: HotelDateRange) -> Self {
            Self {
                begin_date: range.begin_date(),
                end_date: range.end_date(),
            }
        }
    }

    impl TryFrom<HotelDateRangeDTO> for HotelDateRange {
        type Error = anyhow::Error;

        fn try_from(dto: HotelDateRangeDTO) -> Result<Self, Self::Error> {
            HotelDateRange::new(dto.begin_date, dto.end_date)
                .map_err(|e| anyhow!("Invalid hotel date range: {}", e))
        }
    }

    impl From<StationRange<Verified>> for StationRangeDTO {
        fn from(range: StationRange<Verified>) -> Self {
            Self {
                from_station_id: range.get_from_station_id().to_db_value(),
                to_station_id: range.get_to_station_id().to_db_value(),
            }
        }
    }

    impl TryFrom<StationRangeDTO> for StationRange<Verified> {
        type Error = anyhow::Error;
        fn try_from(dto: StationRangeDTO) -> Result<Self, anyhow::Error> {
            Ok(StationRange::from_unchecked(
                StationId::from_db_value(dto.from_station_id)?,
                StationId::from_db_value(dto.to_station_id)?,
            ))
        }
    }

    impl From<SeatType> for SeatTypeDTO {
        fn from(seat_type: SeatType) -> Self {
            Self {
                seat_type_id: seat_type
                    .get_id()
                    .expect("seat type should have id")
                    .to_db_value(),
                type_name: seat_type.name().to_string(),
                capacity: seat_type.capacity(),
                price: seat_type.unit_price(),
            }
        }
    }

    impl TryFrom<SeatTypeDTO> for SeatType {
        type Error = anyhow::Error;

        fn try_from(value: SeatTypeDTO) -> Result<Self, Self::Error> {
            Ok(Self::new(
                Some(SeatTypeId::from_db_value(value.seat_type_id)?),
                SeatTypeName::from_unchecked(value.type_name),
                value.capacity,
                value.price,
            ))
        }
    }

    impl From<SeatLocationInfo> for SeatLocationInfoDTO {
        fn from(info: SeatLocationInfo) -> Self {
            Self {
                carriage: info.carriage,
                row: info.row,
                location: info.location,
            }
        }
    }

    impl TryFrom<SeatLocationInfoDTO> for SeatLocationInfo {
        type Error = anyhow::Error;

        fn try_from(value: SeatLocationInfoDTO) -> Result<Self, Self::Error> {
            Ok(SeatLocationInfo {
                carriage: value.carriage,
                row: value.row,
                location: value.location,
            })
        }
    }

    impl From<Seat> for SeatDTO {
        fn from(value: Seat) -> Self {
            let seat_type = value.seat_type();
            let seat_location = value.location_info();

            Self {
                id: value.get_id().expect("seat should have id").to_db_value(),
                seat_type: seat_type.clone().into(),
                info: seat_location.clone().into(),
                status: value.status().to_string(),
            }
        }
    }

    impl TryFrom<SeatDTO> for Seat {
        type Error = anyhow::Error;

        fn try_from(value: SeatDTO) -> Result<Self, Self::Error> {
            let seat_type = SeatType::try_from(value.seat_type)?;
            let seat_location = SeatLocationInfo::try_from(value.info)?;
            Ok(Seat::new(
                DbId::from_db_value(value.id)?,
                seat_type,
                seat_location,
                SeatStatus::try_from(value.status.as_str()).map_err(|e| anyhow!(e))?,
            ))
        }
    }

    // OrderTimeInfo 转换
    impl From<OrderTimeInfo> for OrderTimeInfoDto {
        fn from(info: OrderTimeInfo) -> Self {
            Self {
                create_time: info.crate_time().to_rfc3339(),
                active_time: info.active_time().to_rfc3339(),
                complete_time: info.complete_time().to_rfc3339(),
            }
        }
    }

    impl TryFrom<OrderTimeInfoDto> for OrderTimeInfo {
        type Error = chrono::ParseError;

        fn try_from(dto: OrderTimeInfoDto) -> Result<Self, Self::Error> {
            let create_time = DateTimeWithTimeZone::from_str(&dto.create_time)?;
            let active_time = DateTimeWithTimeZone::from_str(&dto.active_time)?;
            let complete_time = DateTimeWithTimeZone::from_str(&dto.complete_time)?;
            Ok(OrderTimeInfo::new(create_time, active_time, complete_time))
        }
    }

    // PaymentInfo 转换
    impl From<PaymentInfo> for PaymentInfoDto {
        fn from(info: PaymentInfo) -> Self {
            Self {
                pay_transaction_id: info.pay_transaction_id().map(|id| id.to_db_value()),
                refund_transaction_id: info.refund_transaction_id().map(|id| id.to_db_value()),
            }
        }
    }

    impl TryFrom<PaymentInfoDto> for PaymentInfo {
        type Error = anyhow::Error;

        fn try_from(dto: PaymentInfoDto) -> Result<Self, Self::Error> {
            let pay_transaction_id = dto
                .pay_transaction_id
                .map(|s| TransactionId::from_db_value(s))
                .transpose()?;
            let refund_transaction_id = dto
                .refund_transaction_id
                .map(|s| TransactionId::from_db_value(s))
                .transpose()?;
            Ok(PaymentInfo::new(pay_transaction_id, refund_transaction_id))
        }
    }

    // BaseOrder 转换
    impl From<BaseOrder> for BaseOrderDto {
        fn from(base: BaseOrder) -> Self {
            Self {
                order_id: base.order_id.to_db_value(),
                uuid: base.uuid,
                order_status: base.order_status.to_string(),
                order_time_info: base.order_time_info.into(),
                unit_price: base.unit_price,
                amount: base.amount,
                payment_info: base.payment_info.into(),
                personal_info_id: base.personal_info_id.to_db_value(),
            }
        }
    }

    impl TryFrom<BaseOrderDto> for BaseOrder {
        type Error = anyhow::Error;

        fn try_from(dto: BaseOrderDto) -> Result<Self, Self::Error> {
            Ok(BaseOrder::new(
                OrderId::from_db_value(dto.order_id)?,
                dto.uuid,
                OrderStatus::try_from(dto.order_status.as_str())
                    .map_err(|e| anyhow!("Invalid order status: {}", e))?,
                dto.order_time_info.try_into()?,
                dto.unit_price,
                dto.amount,
                dto.payment_info.try_into()?,
                PersonalInfoId::from_db_value(dto.personal_info_id)?,
            ))
        }
    }

    // TrainOrder 转换
    impl From<TrainOrder> for TrainOrderDto {
        fn from(order: TrainOrder) -> Self {
            Self {
                base: order.base().clone().into(),
                train_schedule_id: order.train_schedule_id().to_db_value(),
                seat: order.seat().clone().into(),
                station_range: order.station_range().clone().into(),
            }
        }
    }

    impl TryFrom<TrainOrderDto> for TrainOrder {
        type Error = anyhow::Error;

        fn try_from(dto: TrainOrderDto) -> Result<Self, Self::Error> {
            let base = dto.base.try_into()?;
            Ok(TrainOrder::new(
                base,
                TrainScheduleId::from_db_value(dto.train_schedule_id)?,
                Seat::try_from(dto.seat)?,
                dto.station_range.try_into()?,
            ))
        }
    }

    impl From<HotelOrder> for HotelOrderDto {
        fn from(order: HotelOrder) -> Self {
            Self {
                base: order.base().clone().into(),
                hotel_id: order.hotel_id().to_db_value(),
                room_id: order.room_id().to_db_value(),
                booking_date_range: order.booking_date_range().into(),
            }
        }
    }

    impl TryFrom<HotelOrderDto> for HotelOrder {
        type Error = anyhow::Error;

        fn try_from(dto: HotelOrderDto) -> Result<Self, Self::Error> {
            Ok(Self::new(
                dto.base.try_into()?,
                HotelId::from_db_value(dto.hotel_id)?,
                HotelRoomTypeId::from_db_value(dto.room_id)?,
                dto.booking_date_range.try_into()?,
            ))
        }
    }

    impl From<DishOrder> for DishOrderDto {
        fn from(order: DishOrder) -> Self {
            Self {
                base: order.base().clone().into(),
                train_order_id: order.train_order_id().to_db_value(),
                dish_id: order.dish_id().to_db_value(),
                unit_price: order.unit_price().to_string(),
                amount: order.amount().to_string(),
            }
        }
    }

    impl TryFrom<DishOrderDto> for DishOrder {
        type Error = anyhow::Error;

        fn try_from(dto: DishOrderDto) -> Result<Self, Self::Error> {
            Ok(Self::new(
                dto.base.try_into()?,
                OrderId::from_db_value(dto.train_order_id)?,
                DishId::from_db_value(dto.dish_id)?,
                Decimal::from_str(&dto.unit_price)?,
                Decimal::from_str(&dto.amount)?,
            ))
        }
    }

    impl From<TakeawayOrder> for TakeawayOrderDto {
        fn from(order: TakeawayOrder) -> Self {
            Self {
                base: order.base().clone().into(),
                train_order_id: order.train_order_id().to_db_value(),
                takeaway_dish_id: order.takeaway_dish_id().to_db_value(),
                unit_price: order.unit_price().to_string(),
                amount: order.amount().to_string(),
            }
        }
    }

    impl TryFrom<TakeawayOrderDto> for TakeawayOrder {
        type Error = anyhow::Error;

        fn try_from(dto: TakeawayOrderDto) -> Result<Self, Self::Error> {
            Ok(Self::new(
                dto.base.try_into()?,
                OrderId::from_db_value(dto.train_order_id)?,
                TakeawayDishId::from_db_value(dto.takeaway_dish_id)?,
                Decimal::from_str(&dto.unit_price)?,
                Decimal::from_str(&dto.amount)?,
            ))
        }
    }

    impl OrderDto {
        /// 将动态Order类型转换为DTO枚举
        pub fn from_dyn_order(order: &dyn Order) -> Result<Self, anyhow::Error> {
            let order = order as &dyn Any;
            if let Some(train) = order.downcast_ref::<TrainOrder>() {
                Ok(OrderDto::Train(train.clone().into()))
            } else if let Some(hotel) = order.downcast_ref::<HotelOrder>() {
                Ok(OrderDto::Hotel(hotel.clone().into()))
            } else if let Some(dish) = order.downcast_ref::<DishOrder>() {
                Ok(OrderDto::Dish(dish.clone().into()))
            } else if let Some(takeaway) = order.downcast_ref::<TakeawayOrder>() {
                Ok(OrderDto::Takeaway(takeaway.clone().into()))
            } else {
                Err(anyhow!("Unsupported order type"))
            }
        }

        /// 将DTO转换回具体Order类型
        pub fn into_dyn_order(self) -> Result<Box<dyn Order>, anyhow::Error> {
            match self {
                OrderDto::Train(dto) => Ok(Box::new(TrainOrder::try_from(dto)?)),
                OrderDto::Hotel(dto) => Ok(Box::new(HotelOrder::try_from(dto)?)),
                OrderDto::Dish(dto) => Ok(Box::new(DishOrder::try_from(dto)?)),
                OrderDto::Takeaway(dto) => Ok(Box::new(TakeawayOrder::try_from(dto)?)),
            }
        }
    }
}

pub struct NotifyDataConverter;

impl NotifyDataConverter {
    pub fn transform_order_notify_to_do(
        order_notify: &OrderNotify,
    ) -> crate::models::message::ActiveModel {
        let dto = ser_order::OrderDto::from_dyn_order(order_notify.order())
            .expect("Failed to convert order notify to DTO");

        let content =
            serde_json::to_value(dto).expect("Failed to serialize order notify DTO to JSON");

        let mut model = crate::models::message::ActiveModel {
            id: ActiveValue::NotSet,
            user_id: ActiveValue::Set(order_notify.user_id().to_db_value()),
            message_type: ActiveValue::Set(order_notify.notify_type().to_string()),
            time: ActiveValue::Set(order_notify.message_time()),
            title: ActiveValue::Set(order_notify.title().to_string()),
            content: ActiveValue::Set(content),
        };

        if let Some(id) = order_notify.notify_id() {
            model.id = ActiveValue::Set(id.to_db_value());
        }

        model
    }

    pub fn make_from_order_notify_do(
        model_do: crate::models::message::Model,
    ) -> Result<OrderNotify, anyhow::Error> {
        let content_dto: ser_order::OrderDto = serde_json::from_value(model_do.content)?;

        let order = content_dto.into_dyn_order()?;

        Ok(OrderNotify::new(
            Some(NotifyId::from_db_value(model_do.id)?),
            UserId::from_db_value(model_do.user_id)?,
            model_do.title,
            model_do.time,
            order,
        ))
    }

    pub fn transform_trip_notify_to_do(
        trip_notify: &TripNotify,
    ) -> crate::models::message::ActiveModel {
        let dto = TripNotifyDTO {
            title: trip_notify.title().to_string(),
            message_time: trip_notify.message_time(),
            train_number: trip_notify.train_number().to_string(),
            departure_time: trip_notify.departure_time(),
            departure_station: trip_notify.departure_station().to_string(),
            arrival_station: trip_notify.arrival_station().to_string(),
        };

        let content =
            serde_json::to_value(dto).expect("Failed to serialize trip notify DTO to JSON");

        let mut model = crate::models::message::ActiveModel {
            id: ActiveValue::NotSet,
            user_id: ActiveValue::Set(trip_notify.user_id().to_db_value()),
            message_type: ActiveValue::Set(trip_notify.notify_type().to_string()),
            time: ActiveValue::Set(trip_notify.message_time()),
            title: ActiveValue::Set(trip_notify.title().to_string()),
            content: ActiveValue::Set(content),
        };

        if let Some(id) = trip_notify.notify_id() {
            model.id = ActiveValue::Set(id.to_db_value());
        }

        model
    }

    pub fn make_from_trip_notify_do(
        model_do: crate::models::message::Model,
    ) -> Result<TripNotify, anyhow::Error> {
        let content_dto: TripNotifyDTO = serde_json::from_value(model_do.content)?;

        Ok(TripNotify::new(
            Some(NotifyId::from_db_value(model_do.id)?),
            UserId::from_db_value(model_do.user_id)?,
            model_do.title,
            model_do.time,
            content_dto.train_number,
            content_dto.departure_time,
            content_dto.departure_station,
            content_dto.arrival_station,
        ))
    }

    pub fn transform_notify_to_do(notify: &dyn Notify) -> crate::models::message::ActiveModel {
        let notify = notify as &dyn Any;

        match (*notify).type_id() {
            id if id == TypeId::of::<OrderNotify>() => {
                Self::transform_order_notify_to_do(notify.downcast_ref::<OrderNotify>().unwrap())
            }
            id if id == TypeId::of::<TripNotify>() => {
                Self::transform_trip_notify_to_do(notify.downcast_ref::<TripNotify>().unwrap())
            }
            _ => panic!("Unsupported notify type"),
        }
    }

    #[instrument]
    pub fn make_from_notify_do(
        model_do: crate::models::message::Model,
    ) -> Result<Box<dyn Notify>, anyhow::Error> {
        let notify_type = NotifyType::try_from(model_do.message_type.as_str()).map_err(|e| {
            error!("Failed to convert notify type: {}", e);
            anyhow!("Invalid notify type: {}", model_do.message_type)
        })?;

        match notify_type {
            NotifyType::Order => {
                let order_notify = Self::make_from_order_notify_do(model_do)?;
                Ok(Box::new(order_notify) as Box<dyn Notify>)
            }
            NotifyType::Trip => {
                let trip_notify = Self::make_from_trip_notify_do(model_do)?;
                Ok(Box::new(trip_notify) as Box<dyn Notify>)
            }
        }
    }
}

#[async_trait]
impl NotifyRepository for NotifyRepositoryImpl {
    #[instrument(skip(self))]
    async fn find(&self, notify_id: NotifyId) -> Result<Option<Box<dyn Notify>>, RepositoryError> {
        let message_do = crate::models::message::Entity::find_by_id(notify_id.to_db_value())
            .one(&self.db)
            .await
            .inspect_err(|e| {
                error!("failed to find notify by id {}: {}", notify_id, e);
            })
            .map_err(|e| RepositoryError::Db(e.into()))?;

        message_do
            .map(NotifyDataConverter::make_from_notify_do)
            .transpose()
            .inspect_err(|e| {
                error!("failed to convert notify from db: {}", e);
            })
            .map_err(RepositoryError::ValidationError)
    }

    #[instrument(skip(self))]
    async fn find_order(
        &self,
        notify_id: NotifyId,
    ) -> Result<Option<OrderNotify>, RepositoryError> {
        let notify = self.find(notify_id).await?;

        let r = notify
            .map(|notify| {
                (notify as Box<dyn Any>)
                    .downcast::<OrderNotify>()
                    .map(|boxed| *boxed)
                    .map_err(|_| {
                        RepositoryError::ValidationError(anyhow!(
                            "Notify {} is not an OrderNotify",
                            notify_id
                        ))
                    })
            })
            .transpose()?;

        Ok(r)
    }

    async fn find_trip(&self, notify_id: NotifyId) -> Result<Option<TripNotify>, RepositoryError> {
        let notify = self.find(notify_id).await?;

        let r = notify
            .map(|notify| {
                (notify as Box<dyn Any>)
                    .downcast::<TripNotify>()
                    .map(|boxed| *boxed)
                    .map_err(|_| {
                        RepositoryError::ValidationError(anyhow!(
                            "Notify {} is not an OrderNotify",
                            notify_id
                        ))
                    })
            })
            .transpose()?;

        Ok(r)
    }

    async fn remove(&self, notify_id: NotifyId) -> Result<(), RepositoryError> {
        crate::models::message::Entity::delete_by_id(notify_id.to_db_value())
            .exec(&self.db)
            .await
            .inspect_err(|e| {
                error!("failed to remove notify by id {}: {}", notify_id, e);
            })
            .map_err(|e| RepositoryError::Db(e.into()))?;

        Ok(())
    }

    async fn save(&self, notify: &mut dyn Notify) -> Result<NotifyId, RepositoryError> {
        let model_do = NotifyDataConverter::transform_notify_to_do(notify);

        if let Some(id) = notify.notify_id() {
            crate::models::message::Entity::update(model_do)
                .exec(&self.db)
                .await
                .inspect_err(|e| {
                    error!("failed to update notify {}: {}", id, e);
                })
                .map_err(|e| RepositoryError::Db(e.into()))?;

            Ok(id)
        } else {
            let result = crate::models::message::Entity::insert(model_do)
                .exec(&self.db)
                .await
                .inspect_err(|e| {
                    error!("failed to insert notify: {}", e);
                })
                .map_err(|e| RepositoryError::Db(e.into()))?;

            let notify_id = NotifyId::from_db_value(result.last_insert_id).map_err(|e| {
                RepositoryError::ValidationError(anyhow!("Invalid notify id: {}", e))
            })?;

            notify.set_notify_id(notify_id);

            Ok(notify_id)
        }
    }

    async fn load_by_user_id(
        &self,
        user_id: UserId,
    ) -> Result<Vec<Box<dyn Notify>>, RepositoryError> {
        let model_do_list = crate::models::message::Entity::find()
            .filter(crate::models::message::Column::UserId.eq(user_id.to_db_value()))
            .all(&self.db)
            .await
            .inspect_err(|e| {
                error!("failed to load notify by user id {}: {}", user_id, e);
            })
            .map_err(|e| RepositoryError::Db(e.into()))?;

        let mut result = Vec::with_capacity(model_do_list.len());

        for model in model_do_list {
            result.push(
                NotifyDataConverter::make_from_notify_do(model)
                    .inspect_err(|e| {
                        error!("failed to convert notify from db: {}", e);
                    })
                    .map_err(RepositoryError::ValidationError)?,
            );
        }

        Ok(result)
    }
}
