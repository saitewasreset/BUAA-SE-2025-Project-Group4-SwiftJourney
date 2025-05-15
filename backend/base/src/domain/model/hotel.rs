use crate::HOTEL_MAX_BOOKING_DAYS;
use crate::domain::Identifier;
use crate::domain::model::city::City;
use crate::domain::model::station::Station;
use crate::domain::model::user::UserId;
use chrono::NaiveDate;
use id_macro::define_id_type;
use rust_decimal::Decimal;
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

pub struct Hotel {
    id: Option<HotelId>,
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
    pub fn new(city: City, station: Station, address: String, info: String) -> Self {
        Self {
            id: None,
            uuid: Uuid::new_v4(),
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

define_id_type!(HotelRoomType);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rating(f64);

impl TryFrom<f64> for Rating {
    type Error = String;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if value < 0.0 || value > 5.0 {
            Err(format!("Rating must be between 0.0 and 5.0, got {}", value))
        } else {
            Ok(Rating(value))
        }
    }
}

impl From<Rating> for f64 {
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

#[derive(Debug, Clone, PartialEq)]
pub struct HotelRating {
    user_id: UserId,
    hotel_id: HotelId,
    time: DateTimeWithTimeZone,
    rating: Rating,
    text: String,
}
