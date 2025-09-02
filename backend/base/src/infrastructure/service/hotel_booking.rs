use crate::domain::model::hotel::{
    HotelDateRange, HotelId, HotelRoomStatus, HotelRoomTypeId, OccupiedRoom,
};
use crate::domain::model::order::{HotelOrder, Order, OrderStatus};
use crate::domain::repository::hotel::HotelRepository;
use crate::domain::repository::occupied_room::OccupiedRoomRepository;
use crate::domain::repository::order::OrderRepository;
use crate::domain::service::hotel_booking::{HotelBookingService, HotelBookingServiceError};
use crate::domain::{Identifiable, RepositoryError};
use anyhow::{anyhow, Context};
use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::prelude::ToPrimitive;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, instrument, warn};
use uuid::Uuid;

pub struct HotelBookingServiceImpl<HR, OR, ORR>
where
    HR: HotelRepository,
    OR: OrderRepository,
    ORR: OccupiedRoomRepository + ?Sized,
{
    hotel_repository: Arc<HR>,
    order_repository: Arc<OR>,
    occupied_room_repository: Arc<ORR>,
}

impl<HR, OR, ORR> HotelBookingServiceImpl<HR, OR, ORR>
where
    HR: HotelRepository,
    OR: OrderRepository,
    ORR: OccupiedRoomRepository,
{
    pub fn new(
        hotel_repository: Arc<HR>,
        order_repository: Arc<OR>,
        occupied_room_repository: Arc<ORR>,
    ) -> Self {
        Self {
            hotel_repository,
            order_repository,
            occupied_room_repository,
        }
    }
}

#[async_trait]
impl<HR, OR, ORR> HotelBookingService for HotelBookingServiceImpl<HR, OR, ORR>
where
    HR: HotelRepository,
    OR: OrderRepository,
    ORR: OccupiedRoomRepository,
{
    #[instrument(skip(self))]
    async fn get_available_room(
        &self,
        hotel_id: HotelId,
        booking_date_range: HotelDateRange,
    ) -> Result<HashMap<HotelRoomTypeId, HotelRoomStatus>, HotelBookingServiceError> {
        let hotel = self
            .hotel_repository
            .find(hotel_id)
            .await?
            .ok_or(HotelBookingServiceError::InvalidHotelId(hotel_id))?;

        let room_type_id_to_capacity = hotel
            .room_type_list()
            .iter()
            .map(|x| {
                (
                    x.get_id().expect("hotel room type should have id"),
                    x.capacity(),
                )
            })
            .collect::<HashMap<_, _>>();

        debug!(
            "room type id to capacity for hotel id: {:?}",
            room_type_id_to_capacity
        );

        let room_type_id_to_price = hotel
            .room_type_list()
            .iter()
            .map(|x| {
                (
                    x.get_id().expect("hotel room type should have id"),
                    x.price(),
                )
            })
            .collect::<HashMap<_, _>>();

        debug!(
            "room type id to price for hotel id: {:?}",
            room_type_id_to_price
        );

        let mut room_type_id_to_date_to_occupied_count: HashMap<
            HotelRoomTypeId,
            HashMap<NaiveDate, i32>,
        > = HashMap::new();

        let possible_occupied_range = self
            .occupied_room_repository
            .find_possible_occupied_range(hotel_id, booking_date_range)
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

        let mut result: HashMap<HotelRoomTypeId, HotelRoomStatus> = HashMap::new();

        for (&room_type_id, &total_count) in &room_type_id_to_capacity {
            let date_to_occupied_count = room_type_id_to_date_to_occupied_count.get(&room_type_id);

            let occupied_count = if let Some(date_to_occupied_count) = date_to_occupied_count {
                date_to_occupied_count
                    .iter()
                    .filter(|(date, _)| {
                        date >= &&booking_date_range.begin_date()
                            && date <= &&booking_date_range.end_date()
                    })
                    .map(|(_, count)| *count)
                    .max()
                    .unwrap_or_default()
            } else {
                0
            };

            debug!(
                "room type id = {} total = {} occupied = {}",
                room_type_id, total_count, occupied_count
            );

            if occupied_count > total_count {
                panic!(
                    "Inconsistent: occupied count {} > total count {} for hotel id {} room type id {}",
                    occupied_count, total_count, hotel_id, room_type_id
                );
            }

            result.insert(
                room_type_id,
                HotelRoomStatus {
                    capacity: total_count,
                    remain_count: total_count - occupied_count,
                    price: *room_type_id_to_price
                        .get(&room_type_id)
                        .expect("room type id should exist"),
                },
            );
        }

        debug!("result: {:?}", result);

        Ok(result)
    }

    #[instrument(skip(self))]
    async fn booking_hotel(&self, order_uuid: Uuid) -> Result<(), HotelBookingServiceError> {
        let mut order = self
            .order_repository
            .find_hotel_order_by_uuid(order_uuid)
            .await
            .inspect_err(|e| error!("Failed to load hotel order: {}", e))?
            .ok_or(HotelBookingServiceError::InvalidOrder(order_uuid))?;

        if order.order_status() != OrderStatus::Paid {
            return Err(HotelBookingServiceError::InvalidOrderStatus(
                order_uuid,
                order.order_status(),
            ));
        }

        let available_room = self
            .get_available_room(order.hotel_id(), order.booking_date_range())
            .await
            .inspect_err(|e| error!("Failed to get available room: {}", e))?;

        let available_count = available_room
            .get(&order.room_id())
            .expect("room id should exist")
            .remain_count;

        let to_order_count = order.amount().to_i32().unwrap();

        if available_count < to_order_count {
            return Err(HotelBookingServiceError::NoAvailableRoom(order_uuid));
        }

        for _ in 0..to_order_count {
            let mut occupied_room = OccupiedRoom::new(
                None,
                order.hotel_id(),
                order.room_id(),
                order.booking_date_range(),
                order.personal_info_id(),
            );

            self.occupied_room_repository
                .save(&mut occupied_room)
                .await
                .inspect_err(|e| error!("Failed to save occupied room: {}", e))?;
        }

        order.set_status(OrderStatus::Ongoing);

        self.order_repository
            .update(Box::new(order))
            .await
            .inspect_err(|e| error!("Failed to update order status: {}", e))?;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn cancel_hotel(&self, order_uuid: Uuid) -> Result<(), HotelBookingServiceError> {
        let mut order = self
            .order_repository
            .find_hotel_order_by_uuid(order_uuid)
            .await?
            .ok_or(HotelBookingServiceError::InvalidOrder(order_uuid))?;

        if order.order_status() != OrderStatus::Ongoing {
            return Err(HotelBookingServiceError::InvalidOrderStatus(
                order_uuid,
                order.order_status(),
            ));
        }

        let to_cancel_occupied_rooms = self
            .occupied_room_repository
            .find_by_order_uuid(order_uuid)
            .await?;

        self.occupied_room_repository
            .remove_many(to_cancel_occupied_rooms)
            .await?;

        order.set_status(OrderStatus::Cancelled);

        self.order_repository
            .update(Box::new(order))
            .await
            .inspect_err(|e| error!("Failed to update order status: {}", e))?;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn booking_group(
        &self,
        order_uuid_list: Vec<Uuid>,
        atomic: bool,
    ) -> Result<Vec<HotelOrder>, HotelBookingServiceError> {
        let mut success_booking_order_list = Vec::new();
        let mut failed_booking_order_list = Vec::new();
        for order_uuid in order_uuid_list {
            if let Err(e) = self.booking_hotel(order_uuid).await {
                warn!("error while booking hotel: {}", e);
                failed_booking_order_list.push(order_uuid);
                match e {
                    HotelBookingServiceError::NoAvailableRoom(_) => continue,
                    x => {
                        error!("Failed to book hotel: {:?}", x);
                        break;
                    }
                }
            } else {
                success_booking_order_list.push(order_uuid);
            }
        }

        if atomic && !failed_booking_order_list.is_empty() {
            for order_uuid in &success_booking_order_list {
                if let Err(e) = self.cancel_hotel(*order_uuid).await {
                    error!("Failed to cancel hotel: {:?}", e);

                    return Err(e);
                }
            }
        }

        let mut result = Vec::with_capacity(failed_booking_order_list.len());

        for order_uuid in failed_booking_order_list {
            result.push(
                self.order_repository
                    .find_hotel_order_by_uuid(order_uuid)
                    .await
                    .context(format!(
                        "Failed to find hotel order by uuid: {}",
                        order_uuid
                    ))
                    .map_err(RepositoryError::Db)?
                    .ok_or(RepositoryError::InconsistentState(anyhow!(
                        "no hotel order record for uuid: {}",
                        order_uuid
                    )))?,
            );
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::mock::{
        hotel::MockHotelRepository, occupied_room::MockOccupiedRoomRepository,
        order::MockOrderRepository,
    };

    use crate::domain::model::hotel::{Hotel, HotelRoomType};
    use crate::domain::model::order::{
        BaseOrder, HotelOrder, OrderStatus, OrderTimeInfo, PaymentInfo,
    };

    use crate::infrastructure::service::hotel_booking::HotelBookingServiceImpl;

    use crate::domain::model::city::City;
    use crate::domain::model::hotel::OccupiedRoom;
    use crate::domain::model::station::Station;
    use crate::domain::model::transaction::Transaction;
    use crate::domain::repository::mock::order::mock_order_repository;
    use chrono::NaiveDate;
    use rust_decimal::Decimal;
    use std::sync::Arc;

    fn make_service(
        hotel_repo: Arc<MockHotelRepository>,
        order_repo: Arc<MockOrderRepository>,
        occupied_repo: Arc<MockOccupiedRoomRepository>,
    ) -> HotelBookingServiceImpl<MockHotelRepository, MockOrderRepository, MockOccupiedRoomRepository>
    {
        HotelBookingServiceImpl::new(hotel_repo, order_repo, occupied_repo)
    }

    // ========== TESTS ==========

    #[tokio::test]
    async fn test_get_available_room_success() {
        // Arrange
        let hotel_uuid = Uuid::new_v4();
        let hotel_repo = Arc::new(MockHotelRepository {
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
                    Some(101u64.into()),
                    Some(1u64.into()),
                    "大床房".to_string(),
                    2,
                    Decimal::new(1000, 2),
                )],
                "日升全程为您服务".to_string(),
            )),
            hotel_id: Some(1u64.into()),
        });
        let order_repo = Arc::new(MockOrderRepository::new());
        let mut occupied_repo = MockOccupiedRoomRepository::new();

        occupied_repo
            .expect_find_possible_occupied_range()
            .returning(|_, _| {
                Ok(vec![OccupiedRoom::new(
                    Some(1u64.into()),
                    1u64.into(),
                    1u64.into(),
                    HotelDateRange::new(
                        NaiveDate::from_ymd_opt(2025, 9, 1).unwrap(),
                        NaiveDate::from_ymd_opt(2025, 9, 2).unwrap(),
                    )
                    .unwrap(),
                    1u64.into(),
                )])
            });

        let service = make_service(hotel_repo, order_repo, Arc::new(occupied_repo));
        let hotel_id = 1u64.into();
        let date_range = HotelDateRange::new(
            NaiveDate::from_ymd_opt(2025, 9, 1).unwrap(),
            NaiveDate::from_ymd_opt(2025, 9, 3).unwrap(),
        );

        // Act
        let result = service
            .get_available_room(hotel_id, date_range.unwrap())
            .await
            .unwrap();

        // Assert
        assert!(!result.is_empty());
    }

    #[tokio::test]
    async fn test_get_available_room_invalid_hotel() {
        let hotel_uuid = Uuid::new_v4();
        let hotel_repo = Arc::new(MockHotelRepository {
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
                    Some(101u64.into()),
                    Some(1u64.into()),
                    "大床房".to_string(),
                    2,
                    Decimal::new(1000, 2),
                )],
                "日升全程为您服务".to_string(),
            )),
            hotel_id: Some(1u64.into()),
        });

        let order_repo = Arc::new(MockOrderRepository::new());
        let mut occupied_repo = MockOccupiedRoomRepository::new();
        occupied_repo
            .expect_find_possible_occupied_range()
            .returning(|_, _| {
                Ok(vec![OccupiedRoom::new(
                    Some(1u64.into()),
                    1u64.into(),
                    1u64.into(),
                    HotelDateRange::new(
                        NaiveDate::from_ymd_opt(2025, 9, 1).unwrap(),
                        NaiveDate::from_ymd_opt(2025, 9, 2).unwrap(),
                    )
                    .unwrap(),
                    1u64.into(),
                )])
            });

        let service = make_service(hotel_repo, order_repo, Arc::new(occupied_repo));
        let hotel_id = 99u64.into();
        let date_range = HotelDateRange::new(
            NaiveDate::from_ymd_opt(2025, 9, 1).unwrap(),
            NaiveDate::from_ymd_opt(2025, 9, 3).unwrap(),
        );

        let result = service
            .get_available_room(hotel_id, date_range.unwrap())
            .await;

        println!("{:?}", result);

        assert!(matches!(
            result,
            Err(HotelBookingServiceError::InvalidHotelId(_))
        ));
    }

    #[tokio::test]
    async fn test_booking_hotel_success() {
        use chrono::NaiveDate;
        use uuid::Uuid;

        // 构造一个测试用的 Uuid
        let order_uuid = Uuid::new_v4();

        let hotel_uuid = Uuid::new_v4();
        let hotel_repo = Arc::new(MockHotelRepository {
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
                    Some(101u64.into()),
                    Some(1u64.into()),
                    "大床房".to_string(),
                    2,
                    Decimal::new(1000, 2),
                )],
                "日升全程为您服务".to_string(),
            )),
            hotel_id: Some(1u64.into()),
        });

        let base_order = BaseOrder::new(
            Some(1u64.into()),
            Uuid::new_v4(),
            OrderStatus::Paid,
            OrderTimeInfo::new(Transaction::now(), Transaction::now(), Transaction::now()),
            Decimal::new(1000, 2),
            Decimal::ONE,
            PaymentInfo::new(Some(1u64.into()), None),
            1u64.into(),
        );

        let mut order_repo = mock_order_repository();
        // 让 order_repo 在调用 find_hotel_order_by_uuid 时返回一个 Paid 状态的订单
        order_repo
            .expect_find_hotel_order_by_uuid()
            .withf(move |uuid| *uuid == order_uuid)
            .returning(move |_| {
                Ok(Some(HotelOrder::new(
                    base_order.clone(),
                    1u64.into(),
                    101u64.into(),
                    HotelDateRange::new(
                        NaiveDate::from_ymd_opt(2025, 9, 1).unwrap(),
                        NaiveDate::from_ymd_opt(2025, 9, 2).unwrap(),
                    )
                    .unwrap(),
                )))
            });

        // update 也需要 mock 掉
        order_repo.expect_update().returning(|_| Ok(()));

        let order_repo = Arc::new(order_repo);
        let mut occupied_repo = MockOccupiedRoomRepository::new();
        occupied_repo
            .expect_find_possible_occupied_range()
            .returning(|_, _| {
                Ok(vec![OccupiedRoom::new(
                    Some(1u64.into()),
                    1u64.into(),
                    101u64.into(),
                    HotelDateRange::new(
                        NaiveDate::from_ymd_opt(2025, 9, 1).unwrap(),
                        NaiveDate::from_ymd_opt(2025, 9, 2).unwrap(),
                    )
                    .unwrap(),
                    1u64.into(),
                )])
            });
        occupied_repo.expect_find_by_order_uuid().returning(|_| {
            Ok(vec![OccupiedRoom::new(
                Some(1u64.into()),
                1u64.into(),
                101u64.into(),
                HotelDateRange::new(
                    NaiveDate::from_ymd_opt(2025, 9, 1).unwrap(),
                    NaiveDate::from_ymd_opt(2025, 9, 2).unwrap(),
                )
                .unwrap(),
                1u64.into(),
            )])
        });
        occupied_repo.expect_save().returning(|_| Ok(101u64.into()));

        let service = make_service(hotel_repo, order_repo, Arc::new(occupied_repo));

        // 传入 order_uuid
        let result = service.booking_hotel(order_uuid).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_booking_hotel_invalid_status() {
        use chrono::NaiveDate;
        use uuid::Uuid;

        // 生成测试用的订单 UUID
        let order_uuid = Uuid::new_v4();

        let hotel_uuid = Uuid::new_v4();
        let hotel_repo = Arc::new(MockHotelRepository {
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
                    Some(101u64.into()),
                    Some(1u64.into()),
                    "大床房".to_string(),
                    2,
                    Decimal::new(1000, 2),
                )],
                "日升全程为您服务".to_string(),
            )),
            hotel_id: Some(1u64.into()),
        });

        // 创建 MockOrderRepository 并设置返回非 Paid 状态的订单
        let base_order = BaseOrder::new(
            Some(1u64.into()),
            Uuid::new_v4(),
            OrderStatus::Cancelled,
            OrderTimeInfo::new(Transaction::now(), Transaction::now(), Transaction::now()),
            Decimal::new(1000, 2),
            Decimal::ONE,
            PaymentInfo::new(Some(1u64.into()), None),
            1u64.into(),
        );

        let mut order_repo = mock_order_repository();
        // 让 order_repo 在调用 find_hotel_order_by_uuid 时返回一个 Paid 状态的订单
        order_repo
            .expect_find_hotel_order_by_uuid()
            .withf(move |uuid| *uuid == order_uuid)
            .returning(move |_| {
                Ok(Some(HotelOrder::new(
                    base_order.clone(),
                    1u64.into(),
                    101u64.into(),
                    HotelDateRange::new(
                        NaiveDate::from_ymd_opt(2025, 9, 1).unwrap(),
                        NaiveDate::from_ymd_opt(2025, 9, 2).unwrap(),
                    )
                    .unwrap(),
                )))
            });
        order_repo.expect_update().returning(|_| Ok(()));

        let order_repo = Arc::new(order_repo);
        let mut occupied_repo = MockOccupiedRoomRepository::new();
        occupied_repo
            .expect_find_possible_occupied_range()
            .returning(|_, _| {
                Ok(vec![OccupiedRoom::new(
                    Some(1u64.into()),
                    1u64.into(),
                    101u64.into(),
                    HotelDateRange::new(
                        NaiveDate::from_ymd_opt(2025, 9, 1).unwrap(),
                        NaiveDate::from_ymd_opt(2025, 9, 2).unwrap(),
                    )
                    .unwrap(),
                    1u64.into(),
                )])
            });
        occupied_repo.expect_save().returning(|_| Ok(101u64.into()));

        let service = make_service(hotel_repo, order_repo, Arc::new(occupied_repo));

        // 调用 booking_hotel 并传入 order_uuid
        let result = service.booking_hotel(order_uuid).await;

        // 断言返回错误类型为 InvalidOrderStatus
        assert!(matches!(
            result,
            Err(HotelBookingServiceError::InvalidOrderStatus(_, _))
        ));
    }

    #[tokio::test]
    async fn test_cancel_hotel_success() {
        use chrono::NaiveDate;
        use rust_decimal::Decimal;
        use uuid::Uuid;

        // 生成测试用的订单 UUID
        let order_uuid = Uuid::new_v4();

        let hotel_repo = Arc::new(MockHotelRepository {
            hotel: Some(Hotel::new(
                "日升大酒店".to_string(),
                City::new(
                    Some(1u64.into()),
                    "北京".to_string().into(),
                    "北京市".to_string().into(),
                ),
                Station::new(Some(1u64.into()), "北京南".to_string(), 1u64.into()),
                "日升路".to_string(),
                "日升全程为您服务".to_string(),
            )),
            hotel_id: Some(1u64.into()),
        });

        // 创建 BaseOrder，设置状态为 Ongoing
        let base_order = BaseOrder::new(
            Some(1u64.into()),
            order_uuid,
            OrderStatus::Ongoing,
            OrderTimeInfo::new(Transaction::now(), Transaction::now(), Transaction::now()),
            Decimal::new(1000, 2),
            Decimal::ONE,
            PaymentInfo::new(Some(1u64.into()), None),
            1u64.into(),
        );

        // 创建 MockOrderRepository 并设置返回订单
        let mut order_repo = mock_order_repository();
        order_repo
            .expect_find_hotel_order_by_uuid()
            .withf(move |uuid| *uuid == order_uuid)
            .returning(move |_| {
                Ok(Some(HotelOrder::new(
                    base_order.clone(),
                    1u64.into(),
                    101u64.into(),
                    HotelDateRange::new(
                        NaiveDate::from_ymd_opt(2025, 9, 1).unwrap(),
                        NaiveDate::from_ymd_opt(2025, 9, 2).unwrap(),
                    )
                    .unwrap(),
                )))
            });

        // MockOccupiedRoomRepository 返回已占用房间
        let mut occupied_repo = MockOccupiedRoomRepository::new();
        occupied_repo.expect_find_by_order_uuid().returning(|_| {
            Ok(vec![OccupiedRoom::new(
                Some(1u64.into()),
                1u64.into(),
                1u64.into(),
                HotelDateRange::new(
                    NaiveDate::from_ymd_opt(2025, 9, 1).unwrap(),
                    NaiveDate::from_ymd_opt(2025, 9, 2).unwrap(),
                )
                .unwrap(),
                1u64.into(),
            )])
        });

        occupied_repo.expect_remove_many().returning(|_| Ok(()));

        let mut order_repo = order_repo;
        order_repo.expect_update().returning(|_| Ok(()));

        let service = make_service(hotel_repo, Arc::new(order_repo), Arc::new(occupied_repo));

        // 调用 cancel_hotel 并传入 order_uuid
        let result = service.cancel_hotel(order_uuid).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cancel_hotel_invalid_status() {
        use chrono::NaiveDate;
        use rust_decimal::Decimal;
        use uuid::Uuid;

        let order_uuid = Uuid::new_v4();

        let hotel_repo = Arc::new(MockHotelRepository {
            hotel: Some(Hotel::new(
                "日升大酒店".to_string(),
                City::new(
                    Some(1u64.into()),
                    "北京".to_string().into(),
                    "北京市".to_string().into(),
                ),
                Station::new(Some(1u64.into()), "北京南".to_string(), 1u64.into()),
                "日升路".to_string(),
                "日升全程为您服务".to_string(),
            )),
            hotel_id: Some(1u64.into()),
        });

        let base_order = BaseOrder::new(
            Some(1u64.into()),
            order_uuid,
            OrderStatus::Cancelled, // 非 Ongoing
            OrderTimeInfo::new(Transaction::now(), Transaction::now(), Transaction::now()),
            Decimal::new(1000, 2),
            Decimal::ONE,
            PaymentInfo::new(Some(1u64.into()), None),
            1u64.into(),
        );

        let mut order_repo = mock_order_repository();
        order_repo
            .expect_find_hotel_order_by_uuid()
            .withf(move |uuid| *uuid == order_uuid)
            .returning(move |_| {
                Ok(Some(HotelOrder::new(
                    base_order.clone(),
                    1u64.into(),
                    101u64.into(),
                    HotelDateRange::new(
                        NaiveDate::from_ymd_opt(2025, 9, 1).unwrap(),
                        NaiveDate::from_ymd_opt(2025, 9, 2).unwrap(),
                    )
                    .unwrap(),
                )))
            });

        let occupied_repo = Arc::new(MockOccupiedRoomRepository::new());
        let order_repo = Arc::new(order_repo);

        let service = make_service(hotel_repo, order_repo, occupied_repo);

        let result = service.cancel_hotel(order_uuid).await;
        assert!(matches!(
            result,
            Err(HotelBookingServiceError::InvalidOrderStatus(_, _))
        ));
    }

    #[tokio::test]
    async fn test_booking_group_partial_fail_atomic_false() {
        use chrono::NaiveDate;
        use rust_decimal::Decimal;
        use uuid::Uuid;

        let order_uuid1 = Uuid::new_v4();
        let order_uuid2 = Uuid::new_v4();

        let hotel_uuid = Uuid::new_v4();
        // ========== 酒店数据 ==========
        let hotel_repo = Arc::new(MockHotelRepository {
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
                    Some(101u64.into()),
                    Some(1u64.into()),
                    "大床房".to_string(),
                    2,
                    Decimal::new(1000, 2),
                )],
                "日升全程为您服务".to_string(),
            )),
            hotel_id: Some(1u64.into()),
        });

        // ========== 订单数据 ==========
        let base_order1 = BaseOrder::new(
            Some(1u64.into()),
            order_uuid1,
            OrderStatus::Paid,
            OrderTimeInfo::new(Transaction::now(), Transaction::now(), Transaction::now()),
            Decimal::new(1000, 2),
            Decimal::ONE,
            PaymentInfo::new(Some(1u64.into()), None),
            101u64.into(), // 房间 ID 对应下面 OccupiedRoom
        );

        let base_order2 = BaseOrder::new(
            Some(2u64.into()),
            order_uuid2,
            OrderStatus::Cancelled, // invalid
            OrderTimeInfo::new(Transaction::now(), Transaction::now(), Transaction::now()),
            Decimal::new(500, 2),
            Decimal::ONE,
            PaymentInfo::new(Some(2u64.into()), None),
            101u64.into(),
        );

        // ========== Mock OrderRepository ==========
        let mut order_repo = mock_order_repository();
        order_repo
            .expect_find_hotel_order_by_uuid()
            .withf(move |uuid| *uuid == order_uuid1)
            .returning(move |_| {
                Ok(Some(HotelOrder::new(
                    base_order1.clone(),
                    1u64.into(),
                    101u64.into(),
                    HotelDateRange::new(
                        NaiveDate::from_ymd_opt(2025, 9, 1).unwrap(),
                        NaiveDate::from_ymd_opt(2025, 9, 2).unwrap(),
                    )
                    .unwrap(),
                )))
            });
        order_repo
            .expect_find_hotel_order_by_uuid()
            .withf(move |uuid| *uuid == order_uuid2)
            .returning(move |_| {
                Ok(Some(HotelOrder::new(
                    base_order2.clone(),
                    1u64.into(),
                    101u64.into(),
                    HotelDateRange::new(
                        NaiveDate::from_ymd_opt(2025, 9, 1).unwrap(),
                        NaiveDate::from_ymd_opt(2025, 9, 2).unwrap(),
                    )
                    .unwrap(),
                )))
            });
        order_repo.expect_update().returning(|_| Ok(()));

        let order_repo = Arc::new(order_repo);

        // ========== Mock OccupiedRoomRepository ==========
        let mut occupied_repo = MockOccupiedRoomRepository::new();
        occupied_repo
            .expect_find_possible_occupied_range()
            .returning(|_, _| {
                Ok(vec![OccupiedRoom::new(
                    Some(1u64.into()),
                    1u64.into(),
                    101u64.into(), // 与订单 room_id 对应
                    HotelDateRange::new(
                        NaiveDate::from_ymd_opt(2025, 9, 1).unwrap(),
                        NaiveDate::from_ymd_opt(2025, 9, 2).unwrap(),
                    )
                    .unwrap(),
                    1u64.into(),
                )])
            });
        occupied_repo.expect_save().returning(|_| Ok(101u64.into()));

        let occupied_repo = Arc::new(occupied_repo);

        // ========== 创建 Service ==========
        let service = make_service(hotel_repo, order_repo, occupied_repo);

        // ========== 执行测试 ==========
        let result = service
            .booking_group(vec![order_uuid1, order_uuid2], false)
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_booking_group_partial_fail_atomic_true() {
        use chrono::NaiveDate;
        use rust_decimal::Decimal;
        use uuid::Uuid;

        let order_uuid1 = Uuid::new_v4();
        let order_uuid2 = Uuid::new_v4();

        let hotel_uuid = Uuid::new_v4();
        let hotel_repo = Arc::new(MockHotelRepository {
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
                    Some(101u64.into()),
                    Some(1u64.into()),
                    "大床房".to_string(),
                    2,
                    Decimal::new(1000, 2),
                )],
                "日升全程为您服务".to_string(),
            )),
            hotel_id: Some(1u64.into()),
        });

        let base_order1 = BaseOrder::new(
            Some(1u64.into()),
            order_uuid1,
            OrderStatus::Ongoing,
            OrderTimeInfo::new(Transaction::now(), Transaction::now(), Transaction::now()),
            Decimal::new(1000, 2),
            Decimal::ONE,
            PaymentInfo::new(Some(1u64.into()), None),
            1u64.into(),
        );

        let base_order2 = BaseOrder::new(
            Some(2u64.into()),
            order_uuid2,
            OrderStatus::Cancelled, // invalid
            OrderTimeInfo::new(Transaction::now(), Transaction::now(), Transaction::now()),
            Decimal::new(500, 2),
            Decimal::ONE,
            PaymentInfo::new(Some(2u64.into()), None),
            1u64.into(),
        );

        let mut order_repo = mock_order_repository();
        order_repo
            .expect_find_hotel_order_by_uuid()
            .withf(move |uuid| *uuid == order_uuid1)
            .returning(move |_| {
                Ok(Some(HotelOrder::new(
                    base_order1.clone(),
                    1u64.into(),
                    101u64.into(),
                    HotelDateRange::new(
                        NaiveDate::from_ymd_opt(2025, 9, 1).unwrap(),
                        NaiveDate::from_ymd_opt(2025, 9, 2).unwrap(),
                    )
                    .unwrap(),
                )))
            });
        order_repo
            .expect_find_hotel_order_by_uuid()
            .withf(move |uuid| *uuid == order_uuid2)
            .returning(move |_| {
                Ok(Some(HotelOrder::new(
                    base_order2.clone(),
                    1u64.into(),
                    101u64.into(),
                    HotelDateRange::new(
                        NaiveDate::from_ymd_opt(2025, 9, 1).unwrap(),
                        NaiveDate::from_ymd_opt(2025, 9, 2).unwrap(),
                    )
                    .unwrap(),
                )))
            });
        order_repo.expect_update().returning(|_| Ok(()));

        let mut occupied_repo = MockOccupiedRoomRepository::new();
        occupied_repo
            .expect_find_possible_occupied_range()
            .returning(|_, _| {
                Ok(vec![OccupiedRoom::new(
                    Some(1u64.into()),
                    1u64.into(),
                    101u64.into(),
                    HotelDateRange::new(
                        NaiveDate::from_ymd_opt(2025, 9, 1).unwrap(),
                        NaiveDate::from_ymd_opt(2025, 9, 2).unwrap(),
                    )
                    .unwrap(),
                    1u64.into(),
                )])
            });
        occupied_repo.expect_save().returning(|_| Ok(101u64.into()));

        let order_repo = Arc::new(order_repo);

        let service = make_service(hotel_repo, order_repo, Arc::new(occupied_repo));

        let result = service
            .booking_group(vec![order_uuid1, order_uuid2], true)
            .await;

        assert!(result.is_ok());
    }
}
