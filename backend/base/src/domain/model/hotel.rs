use crate::HOTEL_MAX_BOOKING_DAYS;
use crate::domain::model::city::City;
use crate::domain::model::personal_info::PersonalInfoId;
use crate::domain::model::station::Station;
use crate::domain::model::user::UserId;
use crate::domain::{Aggregate, Entity, Identifiable, Identifier};
use chrono::NaiveDate;
use id_macro::define_id_type;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use sea_orm::prelude::DateTimeWithTimeZone;
use std::fmt::{Display, Formatter};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum HotelDateRangeError {
    #[error("Date range too long: {specified} > {max}")]
    RangeTooLong { specified: u32, max: u32 },

    #[error("end date should be after begin date: {begin_date} > {end_date}")]
    InvalidEndDate {
        begin_date: NaiveDate,
        end_date: NaiveDate,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HotelDateRange {
    begin_date: NaiveDate,
    end_date: NaiveDate,
}

impl HotelDateRange {
    pub fn new(
        begin_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<HotelDateRange, HotelDateRangeError> {
        let range = (end_date - begin_date).num_days() as u32;
        if range > HOTEL_MAX_BOOKING_DAYS {
            return Err(HotelDateRangeError::RangeTooLong {
                specified: range,
                max: HOTEL_MAX_BOOKING_DAYS,
            });
        }
        if end_date < begin_date {
            return Err(HotelDateRangeError::InvalidEndDate {
                begin_date,
                end_date,
            });
        }
        Ok(HotelDateRange {
            begin_date,
            end_date,
        })
    }

    pub fn begin_date(&self) -> NaiveDate {
        self.begin_date
    }

    pub fn end_date(&self) -> NaiveDate {
        self.end_date
    }
}

define_id_type!(Hotel);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Hotel {
    id: Option<HotelId>,
    name: String,
    uuid: Uuid,
    city: City,
    station: Station,
    address: String,
    phone: Vec<String>,
    images: Vec<String>,
    total_rating_count: i32,
    total_booking_count: i32,
    room_type_list: Vec<HotelRoomType>,
    info: String,
}

impl Hotel {
    pub fn new(name: String, city: City, station: Station, address: String, info: String) -> Self {
        Self {
            id: None,
            uuid: Uuid::new_v4(),
            name,
            city,
            station,
            address,
            phone: Vec::new(),
            images: Vec::new(),
            total_rating_count: 0,
            total_booking_count: 0,
            room_type_list: Vec::new(),
            info,
        }
    }
    pub fn new_full_unchecked(
        id: Option<HotelId>,
        uuid: Uuid,
        name: String,
        city: City,
        station: Station,
        address: String,
        phone: Vec<String>,
        images: Vec<String>,
        total_rating_count: i32,
        total_booking_count: i32,
        room_type_list: Vec<HotelRoomType>,
        info: String,
    ) -> Self {
        Self {
            id,
            uuid,
            name,
            city,
            station,
            address,
            phone,
            images,
            total_rating_count,
            total_booking_count,
            room_type_list,
            info,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn uuid(&self) -> Uuid {
        self.uuid
    }

    pub fn city(&self) -> &City {
        &self.city
    }

    pub fn station(&self) -> &Station {
        &self.station
    }

    pub fn address(&self) -> &String {
        &self.address
    }

    pub fn phone(&self) -> &Vec<String> {
        &self.phone
    }

    pub fn images(&self) -> &Vec<String> {
        &self.images
    }

    pub fn total_rating_count(&self) -> i32 {
        self.total_rating_count
    }

    pub fn total_booking_count(&self) -> i32 {
        self.total_booking_count
    }

    pub fn room_type_list(&self) -> &Vec<HotelRoomType> {
        &self.room_type_list
    }

    pub fn info(&self) -> &String {
        &self.info
    }

    pub fn add_phone(&mut self, phone: String) {
        self.phone.push(phone);
    }

    pub fn add_image(&mut self, image: String) {
        self.images.push(image);
    }

    pub fn add_room_type(&mut self, room_type: HotelRoomType) {
        self.room_type_list.push(room_type);
    }
}

impl Identifiable for Hotel {
    type ID = HotelId;

    fn get_id(&self) -> Option<Self::ID> {
        self.id
    }

    fn set_id(&mut self, id: Self::ID) {
        self.id = Some(id);
    }
}

impl Entity for Hotel {}
impl Aggregate for Hotel {}

define_id_type!(HotelRoomType);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rating(Decimal);

impl TryFrom<Decimal> for Rating {
    type Error = String;

    fn try_from(value: Decimal) -> Result<Self, Self::Error> {
        if value < Decimal::ZERO || value > Decimal::from_f64(5.0).unwrap() {
            Err(format!("Rating must be between 0.0 and 5.0, got {}", value))
        } else {
            Ok(Rating(value))
        }
    }
}

impl Default for Rating {
    fn default() -> Self {
        Rating(Decimal::ZERO)
    }
}

impl From<Rating> for Decimal {
    fn from(value: Rating) -> Self {
        value.0
    }
}

impl Display for Rating {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HotelRoomType {
    id: Option<HotelRoomTypeId>,
    hotel_id: HotelId,
    type_name: String,
    capacity: i32,
    price: Decimal,
}

impl HotelRoomType {
    pub fn new(
        id: Option<HotelRoomTypeId>,
        hotel_id: HotelId,
        type_name: String,
        capacity: i32,
        price: Decimal,
    ) -> Self {
        Self {
            id,
            hotel_id,
            type_name,
            capacity,
            price,
        }
    }

    pub fn hotel_id(&self) -> HotelId {
        self.hotel_id
    }

    pub fn type_name(&self) -> &String {
        &self.type_name
    }

    pub fn capacity(&self) -> i32 {
        self.capacity
    }

    pub fn price(&self) -> Decimal {
        self.price
    }
}

impl Identifiable for HotelRoomType {
    type ID = HotelRoomTypeId;

    fn get_id(&self) -> Option<Self::ID> {
        self.id
    }

    fn set_id(&mut self, id: Self::ID) {
        self.id = Some(id);
    }
}

impl Entity for HotelRoomType {}

define_id_type!(HotelRating);

#[derive(Debug, Clone, PartialEq)]
pub struct HotelRating {
    id: Option<HotelRatingId>,
    user_id: UserId,
    hotel_id: HotelId,
    time: DateTimeWithTimeZone,
    rating: Rating,
    text: String,
}

impl Identifiable for HotelRating {
    type ID = HotelRatingId;

    fn get_id(&self) -> Option<Self::ID> {
        self.id
    }

    fn set_id(&mut self, id: Self::ID) {
        self.id = Some(id);
    }
}

impl Entity for HotelRating {}
impl Aggregate for HotelRating {}

impl HotelRating {
    pub fn new(
        id: Option<HotelRatingId>,
        user_id: UserId,
        hotel_id: HotelId,
        time: DateTimeWithTimeZone,
        rating: Rating,
        text: String,
    ) -> Self {
        Self {
            id,
            user_id,
            hotel_id,
            time,
            rating,
            text,
        }
    }

    pub fn user_id(&self) -> UserId {
        self.user_id
    }

    pub fn hotel_id(&self) -> HotelId {
        self.hotel_id
    }

    pub fn time(&self) -> DateTimeWithTimeZone {
        self.time
    }

    pub fn rating(&self) -> Rating {
        self.rating
    }

    pub fn text(&self) -> &String {
        &self.text
    }
}

pub type HotelRoomTypeStr = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HotelRoomStatus {
    pub capacity: i32,
    pub remain_count: i32,
    pub price: Decimal,
}

define_id_type!(OccupiedRoom);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OccupiedRoom {
    id: Option<OccupiedRoomId>,
    hotel_id: HotelId,
    hotel_room_type_id: HotelRoomTypeId,
    booking_date_range: HotelDateRange,
    personal_info: PersonalInfoId,
}

impl OccupiedRoom {
    pub fn new(
        id: Option<OccupiedRoomId>,
        hotel_id: HotelId,
        hotel_room_type_id: HotelRoomTypeId,
        booking_date_range: HotelDateRange,
        personal_info: PersonalInfoId,
    ) -> Self {
        Self {
            id,
            hotel_id,
            hotel_room_type_id,
            booking_date_range,
            personal_info,
        }
    }

    pub fn hotel_id(&self) -> HotelId {
        self.hotel_id
    }

    pub fn hotel_room_type_id(&self) -> HotelRoomTypeId {
        self.hotel_room_type_id
    }

    pub fn booking_date_range(&self) -> &HotelDateRange {
        &self.booking_date_range
    }

    pub fn personal_info(&self) -> PersonalInfoId {
        self.personal_info
    }
}

impl Identifiable for OccupiedRoom {
    type ID = OccupiedRoomId;

    fn get_id(&self) -> Option<Self::ID> {
        self.id
    }

    fn set_id(&mut self, id: Self::ID) {
        self.id = Some(id);
    }
}

impl Entity for OccupiedRoom {}

impl Aggregate for OccupiedRoom {}
