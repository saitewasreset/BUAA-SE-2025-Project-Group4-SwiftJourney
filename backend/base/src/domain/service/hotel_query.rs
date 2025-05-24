use crate::application::commands::hotel::TargetType;
use crate::application::service::hotel::HotelGeneralInfoDTO;
use crate::domain::model::hotel::{Hotel, HotelDateRange, HotelId};
use async_trait::async_trait;
use rust_decimal::Decimal;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HotelQueryError {
    #[error("Target not found: {0}")]
    TargetNotFound(String),

    #[error("Invalid date range: {0}")]
    InvalidDateRange(String),

    #[error("Repository error: {0}")]
    RepositoryError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

#[async_trait]
pub trait HotelQueryService: Send + Sync + 'static {
    /// 根据目标类型和名称查找酒店
    async fn find_hotels_by_target(
        &self,
        target: &str,
        target_type: &TargetType,
        search_term: Option<&str>,
    ) -> Result<Vec<Hotel>, HotelQueryError>;

    /// 计算指定日期范围内酒店房间的最低价格
    async fn calculate_minimum_prices(
        &self,
        hotels: &[Hotel],
        date_range: Option<&HotelDateRange>,
    ) -> Result<HashMap<HotelId, Decimal>, HotelQueryError>;

    /// 查询酒店信息
    async fn query_hotels(
        &self,
        target: &str,
        target_type: &TargetType,
        search_term: Option<&str>,
        date_range: Option<&HotelDateRange>,
    ) -> Result<Vec<HotelGeneralInfoDTO>, HotelQueryError>;
}
