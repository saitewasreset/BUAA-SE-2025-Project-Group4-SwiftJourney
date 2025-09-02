use crate::application::commands::hotel::TargetType;
use crate::application::service::hotel::HotelGeneralInfoDTO;
use crate::domain::model::hotel::{
    Hotel, HotelDateRange, HotelId, HotelRoomStatus, HotelRoomTypeId,
};
use crate::domain::repository::city::CityRepository;
use crate::domain::repository::hotel::HotelRepository;
use crate::domain::repository::hotel_rating::HotelRatingRepository;
use crate::domain::repository::occupied_room::OccupiedRoomRepository;
use crate::domain::repository::station::StationRepository;
use crate::domain::service::hotel_query::{HotelQueryError, HotelQueryService};
use crate::domain::Identifiable;
use anyhow::anyhow;
use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
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
        date_range: Option<&HotelDateRange>,
    ) -> Result<HashMap<HotelId, Decimal>, HotelQueryError> {
        let mut result = HashMap::with_capacity(hotels.len());

        if date_range.is_none() {
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

        let date_range = date_range.unwrap();

        for hotel in hotels {
            if let Some(hotel_id) = hotel.get_id() {
                let room_statuses = match self.get_all_room_status(hotel_id, date_range).await {
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
        target: &str,
        target_type: &TargetType,
        search_term: Option<&str>,
        date_range: Option<&HotelDateRange>,
    ) -> Result<Vec<HotelGeneralInfoDTO>, HotelQueryError> {
        let hotels = self
            .find_hotels_by_target(target, target_type, search_term)
            .await?;

        if hotels.is_empty() {
            return Ok(Vec::new());
        }

        let prices = self.calculate_minimum_prices(&hotels, date_range).await?;

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
                    Some(format!("/resource/hotel/images/{}", hotel.images()[0]))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::city::City;
    use crate::domain::model::hotel::{Hotel, HotelDateRange, HotelRoomType, OccupiedRoom, Rating};
    use crate::domain::model::station::Station;
    use crate::domain::repository::mock::{
        city::MockCityRepository, hotel::MockHotelRepository,
        hotel_rating::MockHotelRatingRepository, occupied_room::MockOccupiedRoomRepository,
        station::MockStationRepository,
    };
    use crate::domain::RepositoryError;
    use chrono::NaiveDate;
    use rust_decimal::Decimal;
    use std::sync::Arc;
    use uuid::Uuid;

    fn build_service(
        hotel_repo: MockHotelRepository,
        hotel_rating_repo: MockHotelRatingRepository,
        city_repo: MockCityRepository,
        station_repo: MockStationRepository,
        occupied_repo: MockOccupiedRoomRepository,
    ) -> HotelQueryServiceImpl<
        MockHotelRepository,
        MockHotelRatingRepository,
        MockCityRepository,
        MockStationRepository,
        MockOccupiedRoomRepository,
    > {
        HotelQueryServiceImpl::new(
            Arc::new(hotel_repo),
            Arc::new(hotel_rating_repo),
            Arc::new(city_repo),
            Arc::new(station_repo),
            Arc::new(occupied_repo),
        )
    }

    // ---------------- get_all_room_status 测试 ----------------
    #[tokio::test]
    async fn test_get_all_room_status_success() {
        let hotel_uuid = Uuid::new_v4();
        let hotel_id = 1u64.into();
        let room_type_id = 101u64.into();
        let hotel_repo = MockHotelRepository {
            hotel: Some(Hotel::new_full_unchecked(
                Some(1u64.into()),
                hotel_uuid,
                "日升大酒店".to_string(),
                City::new(
                    Some(1u64.into()),
                    "北京".to_string().into(),
                    "北京市".to_string().into(),
                ),
                Station::new(Some(1u64.into()), "北京南".to_string(), 1u64.into()),
                "日升路".to_string(),
                vec![],
                vec![],
                0,
                0,
                vec![HotelRoomType::new(
                    Some(room_type_id),
                    Some(hotel_id),
                    "大床房".to_string(),
                    2,
                    Decimal::new(100, 0),
                )],
                "日升全程为您服务".to_string(),
            )),
            hotel_id: Some(hotel_id),
        };

        let mut occupied_repo = MockOccupiedRoomRepository::new();

        occupied_repo
            .expect_find_possible_occupied_range()
            .returning(move |_, _| {
                Ok(vec![OccupiedRoom::new(
                    Some(1u64.into()),
                    hotel_id,
                    room_type_id,
                    HotelDateRange::new(
                        NaiveDate::from_ymd_opt(2025, 9, 1).unwrap(),
                        NaiveDate::from_ymd_opt(2025, 9, 3).unwrap(),
                    )
                    .unwrap(),
                    1u64.into(),
                )])
            });

        let service = build_service(
            hotel_repo,
            MockHotelRatingRepository::new(),
            MockCityRepository::new(),
            MockStationRepository::new(),
            occupied_repo,
        );

        let date_range = HotelDateRange::new(
            NaiveDate::from_ymd_opt(2025, 9, 1).unwrap(),
            NaiveDate::from_ymd_opt(2025, 9, 3).unwrap(),
        );

        let result = service
            .get_all_room_status(hotel_id, &date_range.unwrap())
            .await
            .unwrap();

        assert!(result.contains_key(&room_type_id));
        let status = result.get(&room_type_id).unwrap();
        assert_eq!(status.capacity, 2);
        assert_eq!(status.remain_count, 1);
        assert_eq!(status.price, Decimal::new(100, 0));
    }

    #[tokio::test]
    async fn test_get_all_room_status_hotel_not_found() {
        let hotel_id = 1u64.into();

        // 不返回任何酒店
        let hotel_repo = MockHotelRepository {
            hotel: None,
            hotel_id: None,
        };

        let occupied_repo = MockOccupiedRoomRepository::new();

        let service = build_service(
            hotel_repo,
            MockHotelRatingRepository::new(),
            MockCityRepository::new(),
            MockStationRepository::new(),
            occupied_repo,
        );

        let date_range = HotelDateRange::new(
            NaiveDate::from_ymd_opt(2025, 9, 1).unwrap(),
            NaiveDate::from_ymd_opt(2025, 9, 2).unwrap(),
        )
        .unwrap();

        let result = service.get_all_room_status(hotel_id, &date_range).await;

        assert!(result.is_err());
    }

    // ---------------- find_hotels_by_target 测试 ----------------
    #[tokio::test]
    async fn test_find_hotels_by_target_city_success() {
        let hotel_uuid = Uuid::new_v4();
        let hotel_id = 1u64.into();
        let room_type_id = 101u64.into();
        let hotel_repo = MockHotelRepository {
            hotel: Some(Hotel::new_full_unchecked(
                Some(1u64.into()),
                hotel_uuid,
                "日升大酒店".to_string(),
                City::new(
                    Some(1u64.into()),
                    "北京".to_string().into(),
                    "北京市".to_string().into(),
                ),
                Station::new(Some(1u64.into()), "北京南".to_string(), 1u64.into()),
                "日升路".to_string(),
                vec![],
                vec![],
                0,
                0,
                vec![HotelRoomType::new(
                    Some(room_type_id),
                    Some(hotel_id),
                    "大床房".to_string(),
                    2,
                    Decimal::new(1000, 2),
                )],
                "日升全程为您服务".to_string(),
            )),
            hotel_id: Some(hotel_id),
        };

        let mut city_repo = MockCityRepository::new();

        let city = City::new(
            Some(1u64.into()),
            "北京".to_string().into(),
            "北京市".to_string().into(),
        );

        city_repo
            .expect_find_by_name()
            .returning(move |_| Ok(vec![city.clone()]));

        let service = build_service(
            hotel_repo,
            MockHotelRatingRepository::new(),
            city_repo,
            MockStationRepository::new(),
            MockOccupiedRoomRepository::new(),
        );

        let result = service
            .find_hotels_by_target("TestCity", &TargetType::City, None)
            .await
            .unwrap();
        assert_eq!(result.len(), 1);
    }

    #[tokio::test]
    async fn test_find_hotels_by_target_city_not_found() {
        let hotel_uuid = Uuid::new_v4();
        let hotel_id = 1u64.into();
        let room_type_id = 101u64.into();
        let hotel_repo = MockHotelRepository {
            hotel: Some(Hotel::new_full_unchecked(
                Some(1u64.into()),
                hotel_uuid,
                "日升大酒店".to_string(),
                City::new(
                    Some(1u64.into()),
                    "北京".to_string().into(),
                    "北京市".to_string().into(),
                ),
                Station::new(Some(1u64.into()), "北京南".to_string(), 1u64.into()),
                "日升路".to_string(),
                vec![],
                vec![],
                0,
                0,
                vec![HotelRoomType::new(
                    Some(room_type_id),
                    Some(hotel_id),
                    "大床房".to_string(),
                    2,
                    Decimal::new(1000, 2),
                )],
                "日升全程为您服务".to_string(),
            )),
            hotel_id: Some(hotel_id),
        };
        let mut city_repo = MockCityRepository::new();
        city_repo.expect_find_by_name().returning(|_| Ok(vec![]));

        let service = build_service(
            hotel_repo,
            MockHotelRatingRepository::new(),
            city_repo,
            MockStationRepository::new(),
            MockOccupiedRoomRepository::new(),
        );

        let result = service
            .find_hotels_by_target("Unknown", &TargetType::City, None)
            .await;
        assert!(matches!(result, Err(HotelQueryError::TargetNotFound(_))));
    }

    // ---------------- calculate_minimum_prices 测试 ----------------
    #[tokio::test]
    async fn test_calculate_minimum_prices_no_date_range() {
        let hotel_uuid = Uuid::new_v4();
        let hotel_id = 1u64.into();
        let room_type_id = 101u64.into();
        let hotel = Hotel::new_full_unchecked(
            Some(1u64.into()),
            hotel_uuid,
            "日升大酒店".to_string(),
            City::new(
                Some(1u64.into()),
                "北京".to_string().into(),
                "北京市".to_string().into(),
            ),
            Station::new(Some(1u64.into()), "北京南".to_string(), 1u64.into()),
            "日升路".to_string(),
            vec![],
            vec![],
            0,
            0,
            vec![HotelRoomType::new(
                Some(room_type_id),
                Some(hotel_id),
                "大床房".to_string(),
                2,
                Decimal::new(200, 0),
            )],
            "日升全程为您服务".to_string(),
        );
        let hotel_repo = MockHotelRepository {
            hotel: Some(hotel.clone()),
            hotel_id: Some(hotel_id),
        };

        let service = build_service(
            hotel_repo,
            MockHotelRatingRepository::new(),
            MockCityRepository::new(),
            MockStationRepository::new(),
            MockOccupiedRoomRepository::new(),
        );

        let result = service
            .calculate_minimum_prices(&[hotel], None)
            .await
            .unwrap();
        assert_eq!(result[&hotel_id], Decimal::new(200, 0));
    }

    #[tokio::test]
    async fn test_calculate_minimum_prices_with_date_range_error() {
        let hotel_uuid = Uuid::new_v4();
        let hotel_id = 1u64.into();
        let room_type_id = 101u64.into();
        let hotel = Hotel::new_full_unchecked(
            Some(1u64.into()),
            hotel_uuid,
            "日升大酒店".to_string(),
            City::new(
                Some(1u64.into()),
                "北京".to_string().into(),
                "北京市".to_string().into(),
            ),
            Station::new(Some(1u64.into()), "北京南".to_string(), 1u64.into()),
            "日升路".to_string(),
            vec![],
            vec![],
            0,
            0,
            vec![HotelRoomType::new(
                Some(room_type_id),
                Some(hotel_id),
                "大床房".to_string(),
                2,
                Decimal::new(1000, 2),
            )],
            "日升全程为您服务".to_string(),
        );
        let hotel_repo = MockHotelRepository {
            hotel: Some(hotel.clone()),
            hotel_id: Some(hotel_id),
        };

        let mut occupied_repo = MockOccupiedRoomRepository::new();

        occupied_repo
            .expect_find_possible_occupied_range()
            .returning(|_, _| Err(RepositoryError::Db(anyhow!("DB error"))));

        let service = build_service(
            hotel_repo,
            MockHotelRatingRepository::new(),
            MockCityRepository::new(),
            MockStationRepository::new(),
            occupied_repo,
        );

        let date_range = HotelDateRange::new(
            NaiveDate::from_ymd_opt(2025, 9, 1).unwrap(),
            NaiveDate::from_ymd_opt(2025, 9, 3).unwrap(),
        );

        let result = service
            .calculate_minimum_prices(&[hotel], Some(&date_range.unwrap()))
            .await
            .unwrap();
        assert!(result.is_empty());
    }

    // ---------------- query_hotels 测试 ----------------
    #[tokio::test]
    async fn test_query_hotels_success() {
        let hotel_uuid = Uuid::new_v4();
        let hotel_id = 1u64.into();
        let room_type_id = 101u64.into();
        let hotel = Hotel::new_full_unchecked(
            Some(1u64.into()),
            hotel_uuid,
            "日升大酒店".to_string(),
            City::new(
                Some(1u64.into()),
                "北京".to_string().into(),
                "北京市".to_string().into(),
            ),
            Station::new(Some(1u64.into()), "北京南".to_string(), 1u64.into()),
            "日升路".to_string(),
            vec![],
            vec![],
            0,
            0,
            vec![HotelRoomType::new(
                Some(room_type_id),
                Some(hotel_id),
                "大床房".to_string(),
                2,
                Decimal::new(1000, 2),
            )],
            "日升全程为您服务".to_string(),
        );
        let hotel_repo = MockHotelRepository {
            hotel: Some(hotel.clone()),
            hotel_id: Some(hotel_id),
        };
        let mut rating_repo = MockHotelRatingRepository::new();
        let mut city_repo = MockCityRepository::new();
        let city = City::new(
            Some(1u64.into()),
            "北京".to_string().into(),
            "北京市".to_string().into(),
        );
        city_repo
            .expect_find_by_name()
            .returning(move |_| Ok(vec![city.clone()]));

        rating_repo
            .expect_get_hotel_rating()
            .returning(|_| Ok(Some(Rating::try_from(Decimal::new(45, 1)).unwrap()))); // 4.5 rating

        let service = build_service(
            hotel_repo,
            rating_repo,
            city_repo,
            MockStationRepository::new(),
            MockOccupiedRoomRepository::new(),
        );

        let result = service
            .query_hotels("TestCity", &TargetType::City, None, None)
            .await
            .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].rating, 4.5);
    }

    #[tokio::test]
    async fn test_query_hotels_not_found() {
        let hotel_uuid = Uuid::new_v4();
        let hotel_id = 1u64.into();
        let room_type_id = 101u64.into();
        let hotel = Hotel::new_full_unchecked(
            Some(1u64.into()),
            hotel_uuid,
            "日升大酒店".to_string(),
            City::new(
                Some(1u64.into()),
                "北京".to_string().into(),
                "北京市".to_string().into(),
            ),
            Station::new(Some(1u64.into()), "北京南".to_string(), 1u64.into()),
            "日升路".to_string(),
            vec![],
            vec![],
            0,
            0,
            vec![HotelRoomType::new(
                Some(room_type_id),
                Some(hotel_id),
                "大床房".to_string(),
                2,
                Decimal::new(1000, 2),
            )],
            "日升全程为您服务".to_string(),
        );
        let hotel_repo = MockHotelRepository {
            hotel: Some(hotel.clone()),
            hotel_id: Some(hotel_id),
        };

        let mut city_repo = MockCityRepository::new();
        city_repo
            .expect_find_by_name()
            .returning(|_target| Ok(vec![])); // 忽略参数，返回空

        let service = build_service(
            hotel_repo,
            MockHotelRatingRepository::new(),
            city_repo,
            MockStationRepository::new(),
            MockOccupiedRoomRepository::new(),
        );

        let result = service
            .query_hotels("Unknown", &TargetType::City, None, None)
            .await;

        assert!(matches!(result, Err(HotelQueryError::TargetNotFound(_))));
    }
}
