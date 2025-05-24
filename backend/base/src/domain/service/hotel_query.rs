use crate::application::commands::hotel::{HotelQuery, TargetType};
use crate::application::service::hotel::HotelGeneralInfoDTO;
use crate::domain::model::hotel::{Hotel, HotelId};
use async_trait::async_trait;
use chrono::NaiveDate;
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
    /// 验证日期范围的有效性
    async fn validate_date_range(
        &self,
        begin_date: Option<NaiveDate>,
        end_date: Option<NaiveDate>,
    ) -> Result<(), HotelQueryError>;

    /// 验证目标（城市/车站）是否存在
    async fn validate_target(
        &self,
        target: &str,
        target_type: &TargetType,
    ) -> Result<(), HotelQueryError>;

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
        begin_date: Option<NaiveDate>,
        end_date: Option<NaiveDate>,
    ) -> Result<HashMap<HotelId, Decimal>, HotelQueryError>;

    /// 查询酒店信息
    async fn query_hotels(
        &self,
        query: &HotelQuery,
    ) -> Result<Vec<HotelGeneralInfoDTO>, HotelQueryError>;
}
