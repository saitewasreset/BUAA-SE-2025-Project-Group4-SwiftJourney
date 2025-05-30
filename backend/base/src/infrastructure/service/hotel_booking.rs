use crate::domain::model::hotel::{
    HotelDateRange, HotelId, HotelRoomStatus, HotelRoomTypeId, OccupiedRoom,
};
use crate::domain::model::order::{HotelOrder, Order, OrderStatus};
use crate::domain::repository::hotel::HotelRepository;
use crate::domain::repository::occupied_room::OccupiedRoomRepository;
use crate::domain::repository::order::OrderRepository;
use crate::domain::service::hotel_booking::{HotelBookingService, HotelBookingServiceError};
use crate::domain::{Identifiable, RepositoryError};
use anyhow::{Context, anyhow};
use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::prelude::ToPrimitive;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, instrument};
use uuid::Uuid;

pub struct HotelBookingServiceImpl<HR, OR, ORR>
where
    HR: HotelRepository,
    OR: OrderRepository,
    ORR: OccupiedRoomRepository,
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

        for (room_type_id, date_to_occupied_count) in room_type_id_to_date_to_occupied_count {
            let total_count = *room_type_id_to_capacity
                .get(&room_type_id)
                .expect("room type id should exist");
            let occupied_count = date_to_occupied_count
                .iter()
                .filter(|(date, _)| {
                    date >= &&booking_date_range.begin_date()
                        && date <= &&booking_date_range.end_date()
                })
                .map(|(_, count)| *count)
                .max()
                .unwrap_or_default();

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

        Ok(result)
    }

    async fn booking_hotel(&self, order_uuid: Uuid) -> Result<(), HotelBookingServiceError> {
        let order = self
            .order_repository
            .find_hotel_order_by_uuid(order_uuid)
            .await?
            .ok_or(HotelBookingServiceError::InvalidOrder(order_uuid))?;

        if order.order_status() != OrderStatus::Paid {
            return Err(HotelBookingServiceError::InvalidOrderStatus(
                order_uuid,
                order.order_status(),
            ));
        }

        let available_room = self
            .get_available_room(order.hotel_id(), order.booking_date_range())
            .await?;

        let available_count = available_room
            .get(&order.room_id())
            .expect("room id should exist")
            .remain_count;

        let to_order_count = order.amount().to_i32().unwrap();

        if available_count < to_order_count {
            return Err(HotelBookingServiceError::NoAvailableRoom(order_uuid));
        }

        let mut occupied_room = OccupiedRoom::new(
            None,
            order.hotel_id(),
            order.room_id(),
            order.booking_date_range(),
            order.personal_info_id(),
        );

        for _ in 0..to_order_count {
            self.occupied_room_repository
                .save(&mut occupied_room)
                .await?;
        }

        Ok(())
    }

    async fn cancel_hotel(&self, order_uuid: Uuid) -> Result<(), HotelBookingServiceError> {
        let order = self
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

        if atomic {
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
