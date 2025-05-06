use crate::domain::model::order::{Order, OrderStatus};
use crate::domain::model::user::UserId;
use crate::domain::{Aggregate, Entity, Identifiable, Identifier};
use chrono::Local;
use id_macro::define_id_type;
use sea_orm::prelude::{DateTimeWithTimeZone, Decimal};
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use thiserror::Error;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TransactionStatus {
    Unpaid,
    Paid,
}

#[derive(Error, Debug)]
pub enum TransactionStatusError {
    #[error("Invalid transaction status: {0}")]
    InvalidStatus(String),
}

impl Display for TransactionStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionStatus::Unpaid => write!(f, "Unpaid"),
            TransactionStatus::Paid => write!(f, "Paid"),
        }
    }
}

impl From<TransactionStatus> for &'static str {
    fn from(status: TransactionStatus) -> Self {
        match status {
            TransactionStatus::Unpaid => "Unpaid",
            TransactionStatus::Paid => "Paid",
        }
    }
}

impl TryFrom<&str> for TransactionStatus {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Unpaid" => Ok(TransactionStatus::Unpaid),
            "Paid" => Ok(TransactionStatus::Paid),
            _ => Err("Invalid transaction status"),
        }
    }
}

define_id_type!(Transaction);

#[derive(Debug, Clone)]
pub struct Transaction {
    transaction_id: Option<TransactionId>,
    uuid: Uuid,
    create_time: DateTimeWithTimeZone,
    finish_time: Option<DateTimeWithTimeZone>,
    amount: Decimal,
    status: TransactionStatus,
    user_id: UserId,
    orders: Vec<Box<dyn Order>>,
}

impl Identifiable for Transaction {
    type ID = TransactionId;

    fn get_id(&self) -> Option<Self::ID> {
        self.transaction_id
    }

    fn set_id(&mut self, id: Self::ID) {
        self.transaction_id = Some(id);
    }
}

impl Entity for Transaction {}

impl Aggregate for Transaction {}

#[derive(Error, Debug)]
pub enum TransactionAmountError {
    #[error("Invalid negative transaction amount: {0}")]
    NegativeValue(Decimal),
}

#[derive(Error, Debug)]
pub enum TransactionError {
    #[error("Transaction already paid: {0}")]
    AlreadyPaid(Uuid),
    #[error("Cannot refund transaction: {0}")]
    RefundError(#[from] RefundError),
}

#[derive(Error, Debug)]
pub enum RefundError {
    #[error("transaction not paid: {0}")]
    NotPaid(Uuid),
    #[error("recharge transaction: {0}")]
    RechargeTransaction(Uuid),
    #[error("transaction already (partial) fulfilled. Fulfilled orders: {0:?}")]
    AlreadyFulfilled(Vec<Uuid>),
    #[error("transaction already (partial) refunded. Refunded orders: {0:?}")]
    AlreadyRefunded(Vec<Uuid>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TransactionAmountAbs(Decimal);

impl From<Decimal> for TransactionAmountAbs {
    fn from(value: Decimal) -> Self {
        Self(value.abs())
    }
}

impl From<TransactionAmountAbs> for Decimal {
    fn from(value: TransactionAmountAbs) -> Self {
        value.0
    }
}

impl Display for TransactionAmountAbs {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Transaction {
    fn now() -> DateTimeWithTimeZone {
        let local_now = Local::now();
        let offset = *local_now.offset(); // 获取系统当前时区偏移
        local_now.with_timezone(&offset)
    }

    pub fn new_recharge(user_id: UserId, recharge_amount: TransactionAmountAbs) -> Transaction {
        Transaction {
            transaction_id: None,
            uuid: Uuid::new_v4(),
            create_time: Self::now(),
            finish_time: Some(Self::now()),
            amount: -Decimal::from(recharge_amount),
            status: TransactionStatus::Paid,
            user_id,
            orders: vec![],
        }
    }

    pub fn new(user_id: UserId, orders: Vec<Box<dyn Order>>) -> Transaction {
        let total_amount = orders
            .iter()
            .map(|order| order.unit_price() * order.amount())
            .sum::<Decimal>();

        Transaction {
            transaction_id: None,
            uuid: Uuid::new_v4(),
            create_time: Self::now(),
            finish_time: None,
            amount: total_amount,
            status: TransactionStatus::Unpaid,
            user_id,
            orders,
        }
    }

    pub fn new_full(
        transaction_id: Option<TransactionId>,
        uuid: Uuid,
        create_time: DateTimeWithTimeZone,
        finish_time: Option<DateTimeWithTimeZone>,
        amount: Decimal,
        status: TransactionStatus,
        user_id: UserId,
        orders: Vec<Box<dyn Order>>,
    ) -> Transaction {
        Transaction {
            transaction_id,
            uuid,
            create_time,
            finish_time,
            amount,
            status,
            user_id,
            orders,
        }
    }

    // 注意：不会检查用户是否有足够的余额
    pub fn pay(&mut self) -> Result<(), TransactionError> {
        if self.status == TransactionStatus::Paid {
            return Err(TransactionError::AlreadyPaid(self.uuid));
        }

        self.status = TransactionStatus::Paid;
        self.finish_time = Some(Self::now());

        Ok(())
    }

    pub fn refund_transaction(&mut self) -> Result<Transaction, RefundError> {
        if self.amount.is_sign_negative() {
            return Err(RefundError::RechargeTransaction(self.uuid));
        }

        let to_refund_order_list = self.orders.clone();

        self.refund_transaction_partial(&to_refund_order_list)
    }

    // 调用者需要保证传入的订单是当前交易的订单
    pub fn refund_transaction_partial(
        &mut self,
        to_refund_orders: &[Box<dyn Order>],
    ) -> Result<Transaction, RefundError> {
        let transaction_order_uuid_set =
            self.orders.iter().map(|e| e.uuid()).collect::<HashSet<_>>();

        for order in to_refund_orders {
            if !transaction_order_uuid_set.contains(&order.uuid()) {
                panic!("Order {} not in transaction {}", order.uuid(), self.uuid);
            }
        }

        if self.status == TransactionStatus::Unpaid {
            return Err(RefundError::NotPaid(self.uuid));
        }

        let mut fulfilled_order_list = Vec::new();
        let mut refunded_order_list = Vec::new();

        for order in &self.orders {
            if order.order_status() == OrderStatus::Active
                || order.order_status() == OrderStatus::Completed
            {
                fulfilled_order_list.push(order.uuid());
            }

            if order.already_refund() {
                refunded_order_list.push(order.uuid());
            }
        }

        if !fulfilled_order_list.is_empty() {
            return Err(RefundError::AlreadyFulfilled(fulfilled_order_list));
        }

        if !refunded_order_list.is_empty() {
            return Err(RefundError::AlreadyRefunded(refunded_order_list));
        }

        let refund_amount_abs = to_refund_orders
            .iter()
            .map(|order| order.unit_price() * order.amount())
            .sum::<Decimal>();

        Ok(Transaction {
            transaction_id: None,
            uuid: Uuid::new_v4(),
            create_time: Self::now(),
            finish_time: Some(Self::now()),
            amount: -refund_amount_abs,
            status: TransactionStatus::Paid,
            user_id: self.user_id,
            orders: self.orders.clone(),
        })
    }

    pub fn uuid(&self) -> Uuid {
        self.uuid
    }

    pub fn create_time(&self) -> DateTimeWithTimeZone {
        self.create_time
    }

    pub fn finish_time(&self) -> Option<DateTimeWithTimeZone> {
        self.finish_time
    }

    pub fn raw_amount(&self) -> Decimal {
        self.amount
    }

    pub fn user_id(&self) -> UserId {
        self.user_id
    }

    pub fn orders(&self) -> &[Box<dyn Order>] {
        &self.orders
    }

    pub fn status(&self) -> TransactionStatus {
        self.status
    }

    pub fn into_orders(self) -> Vec<Box<dyn Order>> {
        self.orders
    }
}
