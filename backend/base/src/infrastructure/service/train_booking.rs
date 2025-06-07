use crate::domain::model::order::{Order, OrderStatus, TrainOrder};
use crate::domain::repository::order::OrderRepository;
use crate::domain::repository::seat_availability::SeatAvailabilityRepository;
use crate::domain::repository::train::TrainRepository;
use crate::domain::repository::train_schedule::TrainScheduleRepository;
use crate::domain::service::ServiceError;
use crate::domain::service::train_booking::{TrainBookingService, TrainBookingServiceError};
use crate::domain::service::train_seat::{TrainSeatService, TrainSeatServiceError};
use crate::domain::service::train_type::TrainTypeConfigurationService;
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
