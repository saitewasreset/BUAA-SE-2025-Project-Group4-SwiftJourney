use crate::HOTEL_MAX_BOOKING_DAYS;
use crate::application::commands::hotel::{HotelQuery, TargetType};
use crate::application::service::hotel::HotelGeneralInfoDTO;
use crate::domain::Identifiable;
use crate::domain::model::hotel::{
    Hotel, HotelDateRange, HotelId, HotelRoomStatus, HotelRoomTypeId,
};
use crate::domain::repository::city::CityRepository;
use crate::domain::repository::hotel::HotelRepository;
use crate::domain::repository::hotel_rating::HotelRatingRepository;
use crate::domain::repository::occupied_room::OccupiedRoomRepository;
use crate::domain::repository::station::StationRepository;
use crate::domain::service::hotel_query::{HotelQueryError, HotelQueryService};
use anyhow::anyhow;
use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::error;

pub struct HotelQueryServiceImpl<HR, HRR, CR, SR, ORR>
where
    HR: HotelRepository,
    HRR: HotelRatingRepository,
    CR: CityRepository,
    SR: StationRepository,
    ORR: OccupiedRoomRepository,
{
    hotel_repository: Arc<HR>,
    hotel_rating_repository: Arc<HRR>,
    city_repository: Arc<CR>,
    station_repository: Arc<SR>,
    occupied_room_repository: Arc<ORR>,
}

impl<HR, HRR, CR, SR, ORR> HotelQueryServiceImpl<HR, HRR, CR, SR, ORR>
where
    HR: HotelRepository,
    HRR: HotelRatingRepository,
    CR: CityRepository,
    SR: StationRepository,
    ORR: OccupiedRoomRepository,
{
    pub fn new(
        hotel_repository: Arc<HR>,
        hotel_rating_repository: Arc<HRR>,
        city_repository: Arc<CR>,
        station_repository: Arc<SR>,
        occupied_room_repository: Arc<ORR>,
    ) -> Self {
        Self {
            hotel_repository,
            hotel_rating_repository,
            city_repository,
            station_repository,
            occupied_room_repository,
        }
    }

    async fn get_all_room_status(
        &self,
        hotel_id: HotelId,
        date_range: &HotelDateRange,
    ) -> Result<HashMap<HotelRoomTypeId, HotelRoomStatus>, anyhow::Error> {
        let hotel = self
            .hotel_repository
            .find(hotel_id)
            .await?
            .ok_or_else(|| anyhow!("Hotel not found with id: {:?}", hotel_id))?;

        let room_type_id_to_capacity: HashMap<_, _> = hotel
            .room_type_list()
            .iter()
            .filter_map(|x| x.get_id().map(|id| (id, x.capacity())))
            .collect();

        let room_type_id_to_price: HashMap<_, _> = hotel
            .room_type_list()
            .iter()
            .filter_map(|x| x.get_id().map(|id| (id, x.price())))
            .collect();

        let mut room_type_id_to_date_to_occupied_count: HashMap<_, HashMap<NaiveDate, i32>> =
            HashMap::new();

        let possible_occupied_range = self
            .occupied_room_repository
            .find_possible_occupied_range(hotel_id, *date_range)
            .await?;

        for occupied_room in possible_occupied_range {
            let entry = room_type_id_to_date_to_occupied_count
                .entry(occupied_room.hotel_room_type_id())
                .or_default();

            let current_begin_date = occupied_room.booking_date_range().begin_date();
            let current_end_date = occupied_room.booking_date_range().end_date();

            for i in 0..(current_end_date - current_begin_date).num_days() {
                let date = current_begin_date + chrono::Duration::days(i);
                let count = entry.entry(date).or_insert(0);
                *count += 1;
            }
        }

        let mut result = HashMap::new();

        for (room_type_id, date_to_occupied_count) in room_type_id_to_date_to_occupied_count {
            if let Some(total_count) = room_type_id_to_capacity.get(&room_type_id) {
                let occupied_count = date_to_occupied_count
                    .iter()
                    .filter(|(date, _)| {
                        date >= &&date_range.begin_date() && date <= &&date_range.end_date()
                    })
                    .map(|(_, count)| *count)
                    .max()
                    .unwrap_or_default();

                if let Some(price) = room_type_id_to_price.get(&room_type_id) {
                    result.insert(
                        room_type_id,
                        HotelRoomStatus {
                            capacity: *total_count,
                            remain_count: total_count - occupied_count,
                            price: *price,
                        },
                    );
                }
            }
        }

        Ok(result)
    }
}

#[async_trait]
impl<HR, HRR, CR, SR, ORR> HotelQueryService for HotelQueryServiceImpl<HR, HRR, CR, SR, ORR>
where
    HR: HotelRepository,
    HRR: HotelRatingRepository,
    CR: CityRepository,
    SR: StationRepository,
    ORR: OccupiedRoomRepository,
{
    async fn validate_date_range(
        &self,
        begin_date: Option<NaiveDate>,
        end_date: Option<NaiveDate>,
    ) -> Result<(), HotelQueryError> {
        match (begin_date, end_date) {
            (None, None) => Ok(()),

            (Some(_), None) | (None, Some(_)) => Err(HotelQueryError::InvalidDateRange(
                "Both dates must be specified or none".into(),
            )),

            (Some(begin), Some(end)) => {
                if end <= begin {
                    return Err(HotelQueryError::InvalidDateRange(
                        "End date must be after begin date".into(),
                    ));
                }

                let duration = end.signed_duration_since(begin).num_days();
                if duration > HOTEL_MAX_BOOKING_DAYS as i64 {
                    return Err(HotelQueryError::InvalidDateRange(format!(
                        "Stay cannot exceed {} days",
                        HOTEL_MAX_BOOKING_DAYS
                    )));
                }

                Ok(())
            }
        }
    }

    async fn validate_target(
        &self,
        target: &str,
        target_type: &TargetType,
    ) -> Result<(), HotelQueryError> {
        match target_type {
            TargetType::City => {
                let cities = self
                    .city_repository
                    .find_by_name(target)
                    .await
                    .map_err(|e| HotelQueryError::RepositoryError(e.to_string()))?;

                if cities.is_empty() {
                    return Err(HotelQueryError::TargetNotFound(target.to_string()));
                }
            }
            TargetType::Station => {
                let stations = self
                    .station_repository
                    .find_by_name(target)
                    .await
                    .map_err(|e| HotelQueryError::RepositoryError(e.to_string()))?;

                if stations.is_none() {
                    return Err(HotelQueryError::TargetNotFound(target.to_string()));
                }
            }
        }

        Ok(())
    }

    async fn find_hotels_by_target(
        &self,
        target: &str,
        target_type: &TargetType,
        search_term: Option<&str>,
    ) -> Result<Vec<Hotel>, HotelQueryError> {
        let hotels = match target_type {
            TargetType::City => {
                let cities = self
                    .city_repository
                    .find_by_name(target)
                    .await
                    .map_err(|e| HotelQueryError::RepositoryError(e.to_string()))?;

                if cities.is_empty() {
                    return Err(HotelQueryError::TargetNotFound(target.to_string()));
                }

                // 匹配城市应该只有一个
                let city = &cities[0];

                let city_id = city.get_id().expect("City should have an ID");
                self.hotel_repository
                    .find_by_city(city_id, search_term)
                    .await
                    .map_err(|e| HotelQueryError::RepositoryError(e.to_string()))?
            }
            TargetType::Station => {
                let station = self
                    .station_repository
                    .find_by_name(target)
                    .await
                    .map_err(|e| HotelQueryError::RepositoryError(e.to_string()))?
                    .ok_or_else(|| HotelQueryError::TargetNotFound(target.to_string()))?;

                let station_id = station.get_id().expect("Station should have an ID");
                self.hotel_repository
                    .find_by_station(station_id, search_term)
                    .await
                    .map_err(|e| HotelQueryError::RepositoryError(e.to_string()))?
            }
        };

        Ok(hotels)
    }

    async fn calculate_minimum_prices(
        &self,
        hotels: &[Hotel],
        begin_date: Option<NaiveDate>,
        end_date: Option<NaiveDate>,
    ) -> Result<HashMap<HotelId, Decimal>, HotelQueryError> {
        let mut result = HashMap::with_capacity(hotels.len());

        if begin_date.is_none() || end_date.is_none() {
            for hotel in hotels {
                if let Some(hotel_id) = hotel.get_id() {
                    let min_price = hotel
                        .room_type_list()
                        .iter()
                        .map(|room| room.price())
                        .min()
                        .unwrap_or_else(|| Decimal::new(0, 0));

                    result.insert(hotel_id, min_price);
                }
            }
            return Ok(result);
        }

        let date_range = match HotelDateRange::new(begin_date.unwrap(), end_date.unwrap()) {
            Ok(range) => range,
            Err(e) => return Err(HotelQueryError::InvalidDateRange(e.to_string())),
        };

        for hotel in hotels {
            if let Some(hotel_id) = hotel.get_id() {
                let room_statuses = match self.get_all_room_status(hotel_id, &date_range).await {
                    Ok(statuses) => statuses,
                    Err(e) => {
                        error!("Failed to get room status for hotel {}: {}", hotel_id, e);
                        continue;
                    }
                };

                let min_price = room_statuses
                    .values()
                    .filter(|status| status.remain_count > 0)
                    .map(|status| status.price)
                    .min()
                    .unwrap_or_else(|| Decimal::new(0, 0));

                result.insert(hotel_id, min_price);
            }
        }

        Ok(result)
    }

    async fn query_hotels(
        &self,
        query: &HotelQuery,
    ) -> Result<Vec<HotelGeneralInfoDTO>, HotelQueryError> {
        self.validate_date_range(query.begin_date, query.end_date)
            .await?;

        self.validate_target(&query.target, &query.target_type)
            .await?;

        let hotels = self
            .find_hotels_by_target(&query.target, &query.target_type, query.search.as_deref())
            .await?;

        if hotels.is_empty() {
            return Ok(Vec::new());
        }

        let prices = self
            .calculate_minimum_prices(&hotels, query.begin_date, query.end_date)
            .await?;

        let mut hotel_infos = Vec::with_capacity(hotels.len());
        for hotel in hotels {
            if let Some(hotel_id) = hotel.get_id() {
                let rating_info = self
                    .hotel_rating_repository
                    .get_hotel_rating(hotel_id)
                    .await
                    .map_err(|e| HotelQueryError::RepositoryError(e.to_string()))?
                    .unwrap_or_default();

                let min_price = prices.get(&hotel_id).cloned().unwrap_or_default();

                let picture_url = if !hotel.images().is_empty() {
                    Some(format!("/api/image/{}", hotel.images()[0]))
                } else {
                    None
                };

                let decimal_rating: Decimal = rating_info.into();
                let rating_value = decimal_rating.to_f64().unwrap_or(0.0);

                hotel_infos.push(HotelGeneralInfoDTO {
                    hotel_id: hotel.uuid(),
                    name: hotel.name().to_string(),
                    picture: picture_url,
                    rating: rating_value,
                    rating_count: hotel.total_rating_count(),
                    total_bookings: hotel.total_booking_count(),
                    price: min_price.to_f64().unwrap_or(0.0),
                    info: hotel.info().to_string(),
                });
            }
        }

        Ok(hotel_infos)
    }
}
