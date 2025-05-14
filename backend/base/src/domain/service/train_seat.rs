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
    #[error("seat availability id {0} not found")]
    InvalidSeatAvailability(SeatAvailabilityId),
}

#[async_trait]
pub trait TrainSeatService: 'static + Send + Sync {
    /// 获取指定区间和座位类型的可用座位数
    ///
    /// # Note
    /// - 处理了部分占用问题
    async fn available_seats_count(
        &self,
        seat_availability_id: SeatAvailabilityId,
    ) -> Result<u32, TrainSeatServiceError>;

    /// 添加座位占用记录
    ///
    /// # Note
    /// - 标记占用相关区间
    async fn reserve_seat(
        &self,
        seat_availability_id: SeatAvailabilityId,
        seat_location_info: SeatLocationInfo,
    ) -> Result<Seat, TrainSeatServiceError>;

    /// 移除座位占用记录
    /// # Note
    /// - 更新占用相关区间座位情况
    async fn free_seat(
        &self,
        seat_availability_id: SeatAvailabilityId,
        seat: Seat,
    ) -> Result<(), TrainSeatServiceError>;
}
