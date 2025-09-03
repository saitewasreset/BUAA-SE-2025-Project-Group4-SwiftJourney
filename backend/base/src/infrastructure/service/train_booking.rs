use crate::domain::model::order::{Order, OrderStatus, TrainOrder};
use crate::domain::repository::order::OrderRepository;
use crate::domain::repository::seat_availability::SeatAvailabilityRepository;
use crate::domain::repository::train::TrainRepository;
use crate::domain::repository::train_schedule::TrainScheduleRepository;
use crate::domain::service::train_booking::{TrainBookingService, TrainBookingServiceError};
use crate::domain::service::train_seat::{TrainSeatService, TrainSeatServiceError};
use crate::domain::service::train_type::TrainTypeConfigurationService;
use crate::domain::service::ServiceError;
use crate::domain::{DbId, Identifiable, RepositoryError};
use anyhow::anyhow;
use async_trait::async_trait;
use std::ops::Deref;
use std::sync::Arc;
use tracing::{error, info, instrument};
use uuid::Uuid;

pub struct TrainBookingServiceImpl<TSR, TSS, TRR, OR, SAR, TTCS>
where
    TSR: TrainScheduleRepository,
    TSS: TrainSeatService,
    TRR: TrainRepository,
    OR: OrderRepository,
    SAR: SeatAvailabilityRepository,
    TTCS: TrainTypeConfigurationService,
{
    train_schedule_repository: Arc<TSR>,
    train_seat_service: Arc<TSS>,
    train_repository: Arc<TRR>,
    order_repository: Arc<OR>,
    seat_availability_repository: Arc<SAR>,
    train_type_configuration_service: Arc<TTCS>,
}

impl<TSR, TSS, TRR, OR, SAR, TTCS> TrainBookingServiceImpl<TSR, TSS, TRR, OR, SAR, TTCS>
where
    TSR: TrainScheduleRepository,
    TSS: TrainSeatService,
    TRR: TrainRepository,
    OR: OrderRepository,
    SAR: SeatAvailabilityRepository,
    TTCS: TrainTypeConfigurationService,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        train_schedule_repository: Arc<TSR>,
        train_seat_service: Arc<TSS>,
        train_repository: Arc<TRR>,
        order_repository: Arc<OR>,
        seat_availability_repository: Arc<SAR>,
        train_type_configuration_service: Arc<TTCS>,
    ) -> Self {
        Self {
            train_schedule_repository,
            train_seat_service,
            train_repository,
            order_repository,
            seat_availability_repository,
            train_type_configuration_service,
        }
    }
}

#[async_trait]
impl<TSR, TSS, TRR, OR, SAR, TTCS> TrainBookingService
    for TrainBookingServiceImpl<TSR, TSS, TRR, OR, SAR, TTCS>
where
    TSR: TrainScheduleRepository,
    TSS: TrainSeatService,
    TRR: TrainRepository,
    OR: OrderRepository,
    SAR: SeatAvailabilityRepository,
    TTCS: TrainTypeConfigurationService,
{
    #[instrument(skip(self))]
    async fn booking_ticket(&self, order_uuid: Uuid) -> Result<(), TrainBookingServiceError> {
        info!("Booking train order: {}", order_uuid);

        let mut train_order = match self
            .order_repository
            .find_train_order_by_uuid(order_uuid)
            .await
        {
            Ok(Some(order)) => order,
            Ok(None) => return Err(TrainBookingServiceError::InvalidOrder(order_uuid)),
            Err(err) => return Err(TrainBookingServiceError::InfrastructureError(err.into())),
        };

        info!("Found train order: {:?}", train_order);

        if train_order.order_status() != OrderStatus::Paid {
            return Err(TrainBookingServiceError::InvalidOrderStatus(
                order_uuid,
                train_order.order_status(),
            ));
        }
        let station_range = train_order.station_range();
        let train_schedule_id = train_order.train_schedule_id();

        let mut train_schedule = match self.train_schedule_repository.find(train_schedule_id).await
        {
            Ok(Some(schedule)) => schedule,
            Ok(None) => {
                return Err(TrainBookingServiceError::InfrastructureError(
                    ServiceError::RelatedServiceError(anyhow!("Train schedule not found")),
                ));
            }
            Err(err) => return Err(TrainBookingServiceError::InfrastructureError(err.into())),
        };

        let train_id = train_schedule.train_id();
        let train = match self.train_repository.find(train_id).await {
            Ok(Some(train)) => train,
            Ok(None) => {
                return Err(TrainBookingServiceError::InfrastructureError(
                    ServiceError::RelatedServiceError(anyhow!(
                        "Train not found for id {}",
                        train_id
                    )),
                ));
            }
            Err(err) => return Err(TrainBookingServiceError::InfrastructureError(err.into())),
        };

        let seat_type_name = train_order.order_seat_type_name();
        let seat_type = match train.seats().get(seat_type_name.deref()) {
            Some(seat_type) => seat_type.clone(),
            None => {
                return Err(TrainBookingServiceError::InfrastructureError(
                    ServiceError::RelatedServiceError(anyhow!(
                        "Seat type '{}' not found in train",
                        seat_type_name.deref()
                    )),
                ));
            }
        };
        let preferred_location = train_order.preferred_seat_location();

        let seat_id_map = match self
            .train_type_configuration_service
            .get_seat_id_map(train_id)
            .await
        {
            Ok(seat_id_map) => seat_id_map,
            Err(err) => {
                return Err(TrainBookingServiceError::InfrastructureError(
                    ServiceError::RelatedServiceError(anyhow!(
                        "Failed to get seat ID map: {}",
                        err
                    )),
                ));
            }
        };

        let train_schedule_occupied_seat = match self
            .seat_availability_repository
            .get_train_schedule_occupied_seat(train_schedule_id)
            .await
        {
            Ok(seat) => seat,
            Err(err) => {
                return Err(TrainBookingServiceError::InfrastructureError(
                    ServiceError::RelatedServiceError(anyhow!(
                        "Failed to get occupied seat: {}",
                        err
                    )),
                ));
            }
        };

        let seat_locations = match seat_id_map.get(seat_type_name) {
            Some(seats) => seats,
            None => {
                return Err(TrainBookingServiceError::InfrastructureError(
                    ServiceError::RelatedServiceError(anyhow!(
                        "Seat type '{}' not found in seat ID map",
                        seat_type_name.deref()
                    )),
                ));
            }
        };

        let seat_type_id = seat_type.get_id().expect("Seat type should have ID");
        let station_begin_id = station_range.get_from_station_id();
        let station_end_id = station_range.get_to_station_id();

        let occupied_seats = match train_schedule_occupied_seat
            .get(&seat_type_id.to_db_value())
            .and_then(|map| {
                map.get(&(station_begin_id.to_db_value(), station_end_id.to_db_value()))
            }) {
            Some(seats) => seats,
            None => &Vec::new(),
        };

        let available_seats: Vec<_> = seat_locations
            .iter()
            .filter(|(id, _)| !occupied_seats.contains(&id.to_db_value()))
            .collect();

        if available_seats.is_empty() {
            train_order.set_status(OrderStatus::Failed);

            return Err(TrainBookingServiceError::NoAvailableTickets(order_uuid));
        }

        let selected_seat = match preferred_location {
            Some(pref_loc) => {
                let pref_char = char::from(*pref_loc);

                available_seats
                    .iter()
                    .find(|(_, info)| info.location == pref_char)
                    .or_else(|| available_seats.first()) // 没有匹配的就选第一个可用的
                    // SAFETY: 前面的检查保证了 available_seats 不为空
                    .unwrap()
            }
            None => available_seats.first().unwrap(),
        };

        info!("Selected seat: {:?}", selected_seat);

        let seat_location_info = selected_seat.1;

        let seat = match self
            .train_seat_service
            .reserve_seat(
                &mut train_schedule,
                station_range,
                seat_type.clone(),
                seat_location_info,
                train_order.personal_info_id(),
            )
            .await
        {
            Ok(seat) => seat,
            Err(err) => {
                if let TrainSeatServiceError::NoAvailableSeat = err {
                    train_order.set_status(OrderStatus::Failed);

                    return Err(TrainBookingServiceError::NoAvailableTickets(order_uuid));
                }

                return Err(TrainBookingServiceError::InfrastructureError(
                    ServiceError::RelatedServiceError(anyhow!("Seat reservation failed: {}", err)),
                ));
            }
        };

        train_order.set_status(OrderStatus::Ongoing);
        train_order.set_seat(Some(seat.clone()));

        self.order_repository
            .update(Box::new(train_order))
            .await
            .map_err(|err| TrainBookingServiceError::InfrastructureError(err.into()))?;

        info!(
            "Train order {} successfully booked with seat: {:?}",
            order_uuid, seat
        );
        Ok(())
    }

    #[instrument(skip(self))]
    async fn cancel_ticket(&self, order_uuid: Uuid) -> Result<(), TrainBookingServiceError> {
        let mut train_order = match self
            .order_repository
            .find_train_order_by_uuid(order_uuid)
            .await
        {
            Ok(Some(order)) => order,
            Ok(None) => return Err(TrainBookingServiceError::InvalidOrder(order_uuid)),
            Err(err) => return Err(TrainBookingServiceError::InfrastructureError(err.into())),
        };

        info!("Cancelling train order: {:?}", train_order);

        let status = train_order.order_status();
        // 只有Unpaid（未支付）、Paid（已支付）、Ongoing（未出行）状态的订单可以取消
        if !(status == OrderStatus::Unpaid
            || status == OrderStatus::Paid
            || status == OrderStatus::Ongoing)
        {
            return Err(TrainBookingServiceError::InvalidOrderStatus(
                order_uuid, status,
            ));
        }

        // 释放座位
        if status == OrderStatus::Ongoing {
            let train_schedule_id = train_order.train_schedule_id();

            let seat = match train_order.seat() {
                Some(seat) => seat,
                None => {
                    return Err(TrainBookingServiceError::InfrastructureError(
                        ServiceError::RelatedServiceError(anyhow!("Seat information is missing")),
                    ));
                }
            };
            let seat_type = seat.seat_type();

            let station_range = train_order.station_range();

            let train_schedule = match self.train_schedule_repository.find(train_schedule_id).await
            {
                Ok(Some(schedule)) => schedule,
                Ok(None) => {
                    return Err(TrainBookingServiceError::InfrastructureError(
                        ServiceError::RelatedServiceError(anyhow!("Train schedule not found")),
                    ));
                }
                Err(err) => return Err(TrainBookingServiceError::InfrastructureError(err.into())),
            };

            let seat_availability_id =
                train_schedule.get_seat_availability_id(station_range, seat_type.clone());

            if let Err(err) = self
                .train_seat_service
                .free_seat(seat_availability_id, seat.clone())
                .await
            {
                return Err(TrainBookingServiceError::InfrastructureError(
                    ServiceError::RelatedServiceError(anyhow!("Failed to release seat: {}", err)),
                ));
            }
        }

        train_order.set_status(OrderStatus::Cancelled);

        self.order_repository
            .update(Box::new(train_order))
            .await
            .map_err(|err| TrainBookingServiceError::InfrastructureError(err.into()))?;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn booking_group(
        &self,
        order_uuid_list: Vec<Uuid>,
        atomic: bool,
    ) -> Result<Vec<TrainOrder>, TrainBookingServiceError> {
        info!("Booking group of train orders: {:?}", order_uuid_list);

        let mut successful_orders: Vec<TrainOrder> = Vec::new();

        let mut failed_orders: Vec<TrainOrder> = Vec::new();

        for order_uuid in order_uuid_list.iter() {
            let order = self
                .order_repository
                .find_train_order_by_uuid(*order_uuid)
                .await
                .inspect_err(|e| error!("Failed to find train order by uuid {}: {}", order_uuid, e))
                .map_err(|e| {
                    TrainBookingServiceError::InfrastructureError(ServiceError::RepositoryError(e))
                })?
                .ok_or_else(|| {
                    error!("Inconsistent: no order with uuid {}", order_uuid);

                    TrainBookingServiceError::InfrastructureError(ServiceError::RepositoryError(
                        RepositoryError::InconsistentState(anyhow!(
                            "Inconsistent: no order with uuid {}",
                            order_uuid
                        )),
                    ))
                })?;

            let result = self.booking_ticket(*order_uuid).await;

            if let Err(err) = result {
                error!("Failed to booking ticket {}: {}", order_uuid, err);

                failed_orders.push(order);

                if atomic {
                    for order in &successful_orders {
                        let _ = self.cancel_ticket(order.uuid()).await;
                    }
                    return Err(err);
                } else {
                    continue;
                }
            } else {
                successful_orders.push(order);
            }
        }

        Ok(failed_orders)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::order::{
        BaseOrder, OrderStatus, OrderTimeInfo, PaymentInfo, TrainOrder,
    };
    use crate::domain::model::personal_info::PreferredSeatLocation;
    use crate::domain::model::train::{SeatType, SeatTypeName, Train, TrainNumber, TrainType};
    use crate::domain::model::train_schedule::{
        Seat, SeatLocationInfo, SeatStatus, StationRange, TrainSchedule,
    };
    use crate::domain::model::transaction::Transaction;
    use crate::domain::repository::mock::{
        order::MockOrderRepository, seat_availability::MockSeatAvailabilityRepository,
        train::MockTrainRepository, train_schedule::MockTrainScheduleRepository,
    };
    use crate::domain::service::mock::{
        train_seat::MockTrainSeatService, train_type::MockTrainTypeConfigurationService,
    };
    use chrono::NaiveDate;
    use rust_decimal::Decimal;
    use std::collections::HashMap;
    use std::sync::Arc;
    use uuid::Uuid;

    fn build_order(order_uuid: Uuid, status: OrderStatus) -> TrainOrder {
        let base_order = BaseOrder::new(
            Some(1u64.into()),
            order_uuid,
            status,
            OrderTimeInfo::new(Transaction::now(), Transaction::now(), Transaction::now()),
            Decimal::new(1000, 2),
            Decimal::ONE,
            PaymentInfo::new(Some(1u64.into()), None),
            1u64.into(),
        );

        TrainOrder::new(
            base_order,
            1u64.into(),
            Some(Seat::new(
                1u64.into(),
                SeatType::new(
                    Some(1u64.into()),
                    SeatTypeName::from_unchecked("二等座".to_string()),
                    1,
                    Decimal::new(1000, 2),
                ),
                SeatLocationInfo {
                    carriage: 3,
                    row: 11,
                    location: 'A',
                },
                SeatStatus::Occupied,
            )),
            SeatTypeName::from_unchecked("二等座".to_string()),
            Some(PreferredSeatLocation::A),
            StationRange::from_unchecked(1u64.into(), 1u64.into()),
        )
    }

    fn build_service(
        order_repo: MockOrderRepository,
        schedule_repo: MockTrainScheduleRepository,
        train_repo: MockTrainRepository,
        seat_repo: MockSeatAvailabilityRepository,
        seat_service: MockTrainSeatService,
        type_conf_service: MockTrainTypeConfigurationService,
    ) -> TrainBookingServiceImpl<
        MockTrainScheduleRepository,
        MockTrainSeatService,
        MockTrainRepository,
        MockOrderRepository,
        MockSeatAvailabilityRepository,
        MockTrainTypeConfigurationService,
    > {
        TrainBookingServiceImpl::new(
            Arc::new(schedule_repo),
            Arc::new(seat_service),
            Arc::new(train_repo),
            Arc::new(order_repo),
            Arc::new(seat_repo),
            Arc::new(type_conf_service),
        )
    }

    // ---------------- booking_ticket ----------------

    #[tokio::test]
    async fn test_booking_ticket_success() {
        let order_uuid = Uuid::new_v4();
        let order = build_order(order_uuid, OrderStatus::Paid);

        let mut order_repo = MockOrderRepository::new();
        order_repo
            .expect_find_train_order_by_uuid()
            .returning(move |_| Ok(Some(order.clone())));
        order_repo.expect_update().returning(|_| Ok(()));

        let mut schedule_repo = MockTrainScheduleRepository::new();
        schedule_repo.expect_find().returning(|_| {
            Ok(Some(TrainSchedule::new(
                Some(1u64.into()),
                1u64.into(),
                NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
                0,
                1u64.into(),
            )))
        });

        let mut seats = HashMap::new();
        seats.insert(
            "二等座".to_string(),
            SeatType::new(
                Some(1u64.into()),
                SeatTypeName::from_unchecked("二等座".to_string()),
                1,
                Decimal::new(1000, 2),
            ),
        );

        let mut train_repo = MockTrainRepository::new();
        train_repo.expect_find().returning(move |_| {
            Ok(Some(Train::new(
                Some(1u64.into()),
                TrainNumber::from_unchecked("G1".to_string()),
                TrainType::from_unchecked("G".to_string()),
                seats.clone(),
                1u64.into(),
                0,
            )))
        });

        let mut seat_repo = MockSeatAvailabilityRepository::new();
        seat_repo
            .expect_get_train_schedule_occupied_seat()
            .returning(|_| Ok(HashMap::new()));

        let mut seat_service = MockTrainSeatService::new();
        seat_service
            .expect_reserve_seat()
            .returning(|_, _, _, _, _| {
                Ok(Seat::new(
                    1u64.into(),
                    SeatType::new(
                        Some(1u64.into()),
                        SeatTypeName::from_unchecked("二等座".to_string()),
                        1,
                        Decimal::new(1000, 2),
                    ),
                    SeatLocationInfo {
                        carriage: 3,
                        row: 11,
                        location: 'A',
                    },
                    SeatStatus::Available,
                ))
            });

        let mut type_conf_service = MockTrainTypeConfigurationService::new();
        type_conf_service.expect_get_seat_id_map().returning(|_| {
            Ok(HashMap::from([(
                SeatTypeName::from_unchecked("二等座".to_string()),
                vec![(
                    1u64.into(),
                    SeatLocationInfo {
                        carriage: 3,
                        row: 11,
                        location: 'A',
                    },
                )],
            )]))
        });

        let service = build_service(
            order_repo,
            schedule_repo,
            train_repo,
            seat_repo,
            seat_service,
            type_conf_service,
        );

        let res = service.booking_ticket(order_uuid).await;

        println!("{:?}", res);

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_booking_ticket_invalid_status() {
        let order_uuid = Uuid::new_v4();
        let order = build_order(order_uuid, OrderStatus::Cancelled);

        let mut order_repo = MockOrderRepository::new();
        order_repo
            .expect_find_train_order_by_uuid()
            .returning(move |_| Ok(Some(order.clone())));

        let service = build_service(
            order_repo,
            MockTrainScheduleRepository::new(),
            MockTrainRepository::new(),
            MockSeatAvailabilityRepository::new(),
            MockTrainSeatService::new(),
            MockTrainTypeConfigurationService::new(),
        );

        let res = service.booking_ticket(order_uuid).await;
        assert!(matches!(
            res,
            Err(TrainBookingServiceError::InvalidOrderStatus(_, _))
        ));
    }

    // ---------------- cancel_ticket ----------------

    #[tokio::test]
    async fn test_cancel_ticket_success() {
        let order_uuid = Uuid::new_v4();
        let order = build_order(order_uuid, OrderStatus::Unpaid);

        let mut order_repo = MockOrderRepository::new();
        order_repo
            .expect_find_train_order_by_uuid()
            .returning(move |_| Ok(Some(order.clone())));
        order_repo.expect_update().returning(|_| Ok(()));

        let service = build_service(
            order_repo,
            MockTrainScheduleRepository::new(),
            MockTrainRepository::new(),
            MockSeatAvailabilityRepository::new(),
            MockTrainSeatService::new(),
            MockTrainTypeConfigurationService::new(),
        );

        let res = service.cancel_ticket(order_uuid).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_cancel_ticket_invalid_status() {
        let order_uuid = Uuid::new_v4();
        let order = build_order(order_uuid, OrderStatus::Cancelled);

        let mut order_repo = MockOrderRepository::new();
        order_repo
            .expect_find_train_order_by_uuid()
            .returning(move |_| Ok(Some(order.clone())));

        let service = build_service(
            order_repo,
            MockTrainScheduleRepository::new(),
            MockTrainRepository::new(),
            MockSeatAvailabilityRepository::new(),
            MockTrainSeatService::new(),
            MockTrainTypeConfigurationService::new(),
        );

        let res = service.cancel_ticket(order_uuid).await;
        assert!(matches!(
            res,
            Err(TrainBookingServiceError::InvalidOrderStatus(_, _))
        ));
    }

    // ---------------- booking_group ----------------

    #[tokio::test]
    async fn test_booking_group_success() {
        let order_uuid = Uuid::new_v4();
        let order = build_order(order_uuid, OrderStatus::Paid);

        let mut order_repo = MockOrderRepository::new();
        order_repo
            .expect_find_train_order_by_uuid()
            .returning(move |_| Ok(Some(order.clone())));
        order_repo.expect_update().returning(|_| Ok(()));

        // 其余 repo 模拟 booking_ticket 正常
        let mut schedule_repo = MockTrainScheduleRepository::new();
        schedule_repo.expect_find().returning(|_| {
            Ok(Some(TrainSchedule::new(
                Some(1u64.into()),
                1u64.into(),
                NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
                0,
                1u64.into(),
            )))
        });

        let mut seats = HashMap::new();
        seats.insert(
            "二等座".to_string(),
            SeatType::new(
                Some(1u64.into()),
                SeatTypeName::from_unchecked("二等座".to_string()),
                1,
                Decimal::new(1000, 2),
            ),
        );

        let mut train_repo = MockTrainRepository::new();
        train_repo.expect_find().returning(move |_| {
            Ok(Some(Train::new(
                Some(1u64.into()),
                TrainNumber::from_unchecked("G1".to_string()),
                TrainType::from_unchecked("G".to_string()),
                seats.clone(),
                1u64.into(),
                0,
            )))
        });

        let mut seat_repo = MockSeatAvailabilityRepository::new();
        seat_repo
            .expect_get_train_schedule_occupied_seat()
            .returning(|_| Ok(HashMap::new()));

        let mut seat_service = MockTrainSeatService::new();
        seat_service
            .expect_reserve_seat()
            .returning(|_, _, _, _, _| {
                Ok(Seat::new(
                    1u64.into(),
                    SeatType::new(
                        Some(1u64.into()),
                        SeatTypeName::from_unchecked("二等座".to_string()),
                        1,
                        Decimal::new(1000, 2),
                    ),
                    SeatLocationInfo {
                        carriage: 3,
                        row: 11,
                        location: 'A',
                    },
                    SeatStatus::Available,
                ))
            });

        let mut type_conf_service = MockTrainTypeConfigurationService::new();
        type_conf_service.expect_get_seat_id_map().returning(|_| {
            Ok(HashMap::from([(
                SeatTypeName::from_unchecked("二等座".to_string()),
                vec![(
                    1u64.into(),
                    SeatLocationInfo {
                        carriage: 3,
                        row: 11,
                        location: 'A',
                    },
                )],
            )]))
        });

        let service = build_service(
            order_repo,
            schedule_repo,
            train_repo,
            seat_repo,
            seat_service,
            type_conf_service,
        );

        let res = service.booking_group(vec![order_uuid], false).await;
        assert!(res.is_ok());
        assert!(res.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_booking_group_atomic_fail() {
        let order_uuid = Uuid::new_v4();

        let mut order_repo = MockOrderRepository::new();
        order_repo
            .expect_find_train_order_by_uuid()
            .returning(move |_| Ok(None)); // 模拟查不到订单

        let service = build_service(
            order_repo,
            MockTrainScheduleRepository::new(),
            MockTrainRepository::new(),
            MockSeatAvailabilityRepository::new(),
            MockTrainSeatService::new(),
            MockTrainTypeConfigurationService::new(),
        );

        let res = service.booking_group(vec![order_uuid], true).await;
        assert!(res.is_err());
    }
}
