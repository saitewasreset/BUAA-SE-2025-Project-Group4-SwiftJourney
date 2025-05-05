use crate::domain::model::train_schedule::{Seat, SeatAvailabilityId, SeatLocationInfo};
use crate::domain::service::ServiceError;
use async_trait::async_trait;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TrainSeatServiceError {
    /// 底层基础设施错误（如数据库访问失败）
    #[error("an infrastructure error occurred: {0}")]
    InfrastructureError(ServiceError),
    #[error("no available seat")]
    NoAvailableSeat,
    #[error("seat hasn't been reserved")]
    UnreservedSeat,
    #[error("specified seat not in seat availability {0}")]
    InvalidSeat(SeatAvailabilityId),
}

#[async_trait]
pub trait TrainSeatService {
    /// 获取指定区间和座位类型的可用座位数
    ///
    /// # Note
    /// 此函数只统计精确占用StationRange的座位数，更小和更大范围内占用的都**不会**被统计
    async fn available_seats_count(
        &self,
        seat_availability_id: SeatAvailabilityId,
    ) -> Result<u32, TrainSeatServiceError>;

    /// 添加座位占用记录
    ///
    /// # Note
    /// - 此函数只精确占用StationRange，更小和更大范围都**不会**被自动标记占用
    async fn reserve_seat(
        &self,
        seat_availability_id: SeatAvailabilityId,
        seat_location_info: SeatLocationInfo,
    ) -> Result<Seat, TrainSeatServiceError>;

    /// 移除座位占用记录
    async fn free_seat(
        &self,
        seat_availability_id: SeatAvailabilityId,
        seat: Seat,
    ) -> Result<(), TrainSeatServiceError>;
}
