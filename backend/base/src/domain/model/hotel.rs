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
        if end_date < begin_date {
            return Err(HotelDateRangeError::InvalidEndDate {
                begin_date,
                end_date,
            });
        }
        let range = (end_date - begin_date).num_days() as u32;
        if range > HOTEL_MAX_BOOKING_DAYS {
            return Err(HotelDateRangeError::RangeTooLong {
                specified: range,
                max: HOTEL_MAX_BOOKING_DAYS,
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
    images: Vec<Uuid>,
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

    #[allow(clippy::too_many_arguments)]
    pub fn new_full_unchecked(
        id: Option<HotelId>,
        uuid: Uuid,
        name: String,
        city: City,
        station: Station,
        address: String,
        phone: Vec<String>,
        images: Vec<Uuid>,
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

    pub fn images(&self) -> &Vec<Uuid> {
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

    pub fn add_image(&mut self, image: Uuid) {
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

        for room_type in &mut self.room_type_list {
            room_type.hotel_id = Some(id);
        }
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
    hotel_id: Option<HotelId>,
    type_name: String,
    capacity: i32,
    price: Decimal,
}

impl HotelRoomType {
    pub fn new(
        id: Option<HotelRoomTypeId>,
        hotel_id: Option<HotelId>,
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

    pub fn hotel_id(&self) -> Option<HotelId> {
        self.hotel_id
    }

    pub fn set_hotel_id(&mut self, hotel_id: HotelId) {
        self.hotel_id = Some(hotel_id);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::city::{City, CityId, CityName, ProvinceName};
    use crate::domain::model::station::Station;
    use chrono::NaiveDate;
    use claims::{assert_err, assert_ok, assert_ok_eq};
    use rust_decimal::Decimal;
    use rust_decimal::prelude::FromPrimitive;

    #[test]
    fn hotel_date_range_valid_and_invalid() {
        let begin = NaiveDate::from_ymd_opt(2025, 8, 1).unwrap();
        let end_ok = NaiveDate::from_ymd_opt(2025, 8, 8).unwrap(); // 7天
        assert_ok!(HotelDateRange::new(begin, end_ok));

        let end_short = NaiveDate::from_ymd_opt(2025, 7, 31).unwrap();
        let err = HotelDateRange::new(begin, end_short).unwrap_err();
        assert!(matches!(err, HotelDateRangeError::InvalidEndDate { .. }));

        let end_long = NaiveDate::from_ymd_opt(2025, 8, 9).unwrap(); // 8天
        let err = HotelDateRange::new(begin, end_long).unwrap_err();
        assert!(matches!(err, HotelDateRangeError::RangeTooLong { .. }));
    }

    #[test]
    fn rating_bounds() {
        assert_ok_eq!(Rating::try_from(Decimal::ZERO), Rating(Decimal::ZERO));
        let five = Decimal::from_f64(5.0).unwrap();
        assert_ok_eq!(Rating::try_from(five), Rating(five));

        let over = Decimal::from_f64(5.1).unwrap();
        assert_err!(Rating::try_from(over));
        let neg = Decimal::from_f64(-0.1).unwrap();
        assert_err!(Rating::try_from(neg));
    }

    #[test]
    fn set_id_cascades_to_room_types() {
        // 准备一个带两个房型的酒店
        let city_id: CityId = 100u64.into();
        let city = City::new(
            Some(city_id),
            CityName::from("TestCity".to_string()),
            ProvinceName::from("TestProvince".to_string()),
        );
        let station = Station::new(None, "Station".to_string(), city_id);
        let mut hotel = Hotel::new("Hotel".into(), city, station, "addr".into(), "info".into());
        let price = Decimal::from_f64(100.0).unwrap();
        hotel.add_room_type(HotelRoomType::new(None, None, "A".into(), 2, price));
        hotel.add_room_type(HotelRoomType::new(None, None, "B".into(), 3, price));

        let new_id: HotelId = 1u64.into();
        assert!(hotel.get_id().is_none());
        hotel.set_id(new_id);
        assert_eq!(hotel.get_id(), Some(new_id));

        for rt in hotel.room_type_list() {
            assert_eq!(rt.hotel_id(), Some(new_id));
        }
    }
}
