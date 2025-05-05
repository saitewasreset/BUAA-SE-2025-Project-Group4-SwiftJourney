use crate::HOTEL_MAX_BOOKING_DAYS;
use crate::domain::Identifier;
use chrono::NaiveDate;
use id_macro::define_id_type;
use thiserror::Error;

define_id_type!(Hotel);
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

define_id_type!(HotelRoom);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HotelRoom {
    id: Option<HotelRoomId>,
}
