use crate::internal::order::dto::InternalOrderDTO;
use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewTransactionCommand {
    pub user_id: u64,
    pub orders: Vec<InternalOrderDTO>,
    pub atomic: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RefundTransactionCommand {
    pub transaction_id: Uuid,
    pub to_refund_orders: Vec<InternalOrderDTO>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrderByUuidQuery {
    pub order_uuid: Uuid,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerifyTrainOrderQuery {
    pub user_id: u64,
    pub train_number: String,
    pub origin_departure_time: DateTimeWithTimeZone,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateOrdersCommand {
    pub orders: Vec<InternalOrderDTO>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserOrderListQuery {
    pub user_id: u64,
}
