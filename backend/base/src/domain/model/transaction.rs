//! # 交易实体模块
//!
//! 该模块定义了火车票订购系统中的交易相关实体数据结构及其相关操作。主要包含以下内容：
//!
//! - `TransactionStatus`: 枚举类型，表示交易的状态。
//! - `TransactionStatusError`: 枚举类型，表示交易状态错误。
//! - `TransactionAmountError`: 枚举类型，表示交易金额错误。
//! - `TransactionError`: 枚举类型，表示交易错误。
//! - `RefundError`: 枚举类型，表示退款错误。
//! - `TransactionAmountAbs`: 结构体，表示交易金额的绝对值。
//! - `Transaction`: 结构体，表示交易实体。
//!
//! ## 关于交易和订单的约定
//!
//! - 一笔订单明确对应一个服务，例如：一张火车票、一个房间预订、一份火车餐。
//! - 交易对应一笔支付，一个交易可包含多个订单，例如：添加多个乘车人后点击“预订”，产生多个订单，但只有一个交易；交易有“未支付”、“已支付”两种状态。
//! - 只能取消“订单”，而不能直接取消“交易”。若需“取消”交易，需通过退款交易实现。
//! - 取消订单、失败订单的退款通过新的退款交易返还，原始支付交易不变。
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

/// 枚举类型，表示交易的状态。
///
/// 主要包含以下状态：
/// - `Unpaid`: 交易尚未支付。
/// - `Paid`: 交易已支付。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TransactionStatus {
    Unpaid,
    Paid,
}

/// 枚举类型，表示交易状态错误。
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
    /// 将 `TransactionStatus` 枚举类型转换为字符串。
    ///
    /// Returns:
    /// - 对应的字符串。
    fn from(status: TransactionStatus) -> Self {
        match status {
            TransactionStatus::Unpaid => "Unpaid",
            TransactionStatus::Paid => "Paid",
        }
    }
}

impl TryFrom<&str> for TransactionStatus {
    type Error = &'static str;

    /// 将字符串尝试转换为 `TransactionStatus` 枚举类型。
    ///
    /// Arguments:
    /// - `value`: 要转换的字符串。
    ///
    /// Returns:
    /// - 成功时返回 `TransactionStatus` 枚举类型。
    /// - 失败时返回错误字符串。
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Unpaid" => Ok(TransactionStatus::Unpaid),
            "Paid" => Ok(TransactionStatus::Paid),
            _ => Err("Invalid transaction status"),
        }
    }
}

define_id_type!(Transaction);

/// 结构体，表示交易实体。
///
/// 包含以下字段：
/// - `transaction_id`: 交易的唯一标识符，可以为空。
/// - `uuid`: 交易的 UUID。
/// - `create_time`: 交易创建时间。
/// - `finish_time`: 交易完成时间，可能为空。
/// - `amount`: 交易金额。
/// - `status`: 交易状态。
/// - `user_id`: 用户的唯一标识符。
/// - `orders`: 交易包含的订单列表。
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

/// 枚举类型，表示交易金额错误。
#[derive(Error, Debug)]
pub enum TransactionAmountError {
    #[error("Invalid negative transaction amount: {0}")]
    NegativeValue(Decimal),
}
/// 枚举类型，表示交易错误。
#[derive(Error, Debug)]
pub enum TransactionError {
    #[error("Transaction already paid: {0}")]
    AlreadyPaid(Uuid),
    #[error("Cannot refund transaction: {0}")]
    RefundError(#[from] RefundError),
}

/// 枚举类型，表示退款错误。
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

/// 结构体，表示交易金额的绝对值。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TransactionAmountAbs(Decimal);

#[derive(Error, Debug)]
pub enum TransactionAmountAbsError {
    #[error("negative value")]
    NegativeValue,
    #[error("invalid value")]
    InvalidValue,
}

impl TransactionAmountAbs {
    pub fn from_f64_checked(value: f64) -> Result<TransactionAmountAbs, TransactionAmountAbsError> {
        if value < 0.0 {
            return Err(TransactionAmountAbsError::NegativeValue);
        }

        let dec = Decimal::try_from(value).map_err(|e| TransactionAmountAbsError::InvalidValue)?;

        Ok(TransactionAmountAbs(dec))
    }
}

impl From<Decimal> for TransactionAmountAbs {
    /// 将 `Decimal` 类型转换为 `TransactionAmountAbs`。
    ///
    /// Arguments:
    /// - `value`: 要转换的 `Decimal` 值。
    ///
    /// Returns:
    /// - `TransactionAmountAbs` 实例。
    fn from(value: Decimal) -> Self {
        Self(value.abs())
    }
}

impl From<TransactionAmountAbs> for Decimal {
    /// 将 `TransactionAmountAbs` 转换为 `Decimal`。
    ///
    /// Arguments:
    /// - `value`: 要转换的 `TransactionAmountAbs` 实例。
    ///
    /// Returns:
    /// - `Decimal` 值。
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
    /// 获取当前系统时间并返回带有时区的时间。
    ///
    /// Returns:
    /// - 当前系统时间，带有时区。
    fn now() -> DateTimeWithTimeZone {
        let local_now = Local::now();
        let offset = *local_now.offset(); // 获取系统当前时区偏移
        local_now.with_timezone(&offset)
    }

    /// 创建一个新的充值交易实例。
    ///
    /// Arguments:
    /// - `user_id`: 用户的唯一标识符。
    /// - `recharge_amount`: 充值金额的绝对值。
    ///
    /// Returns:
    /// - 新创建的充值交易实例。
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

    /// 创建一个新的调试交易实例。
    ///
    /// Arguments:
    /// - `user_id`: 用户的唯一标识符。
    /// - `recharge_amount`: 充值金额的绝对值。
    ///
    /// Returns:
    /// - 新创建的调试交易实例。
    pub fn new_debug(user_id: UserId, amount: TransactionAmountAbs) -> Transaction {
        Transaction {
            transaction_id: None,
            uuid: Uuid::new_v4(),
            create_time: Self::now(),
            finish_time: Some(Self::now()),
            amount: Decimal::from(amount),
            status: TransactionStatus::Unpaid,
            user_id,
            orders: vec![],
        }
    }

    /// 创建一个新的交易实例。
    ///
    /// Arguments:
    /// - `user_id`: 用户的唯一标识符。
    /// - `orders`: 交易包含的订单列表。
    ///
    /// Returns:
    /// - 新创建的交易实例。
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

    /// 创建一个新的完整交易实例。
    ///
    /// Arguments:
    /// - `transaction_id`: 交易的唯一标识符，可以为空。
    /// - `uuid`: 交易的 UUID。
    /// - `create_time`: 交易创建时间。
    /// - `finish_time`: 交易完成时间，可能为空。
    /// - `amount`: 交易金额。
    /// - `status`: 交易状态。
    /// - `user_id`: 用户的唯一标识符。
    /// - `orders`: 交易包含的订单列表。
    ///
    /// Returns:
    /// - 新创建的完整交易实例。
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

    /// 标记交易为已支付。
    ///
    /// Returns:
    /// - 成功时返回 `Ok(())`。
    /// - 失败时返回 `TransactionError`。
    ///
    /// Notes:
    /// 不会检查用户是否有足够的余额
    pub fn pay(&mut self) -> Result<(), TransactionError> {
        if self.status == TransactionStatus::Paid {
            return Err(TransactionError::AlreadyPaid(self.uuid));
        }

        self.status = TransactionStatus::Paid;
        self.finish_time = Some(Self::now());

        Ok(())
    }

    /// 创建一个新的退款交易实例。
    ///
    /// Returns:
    /// - 成功时返回新的退款交易实例。
    /// - 失败时返回 `RefundError`。
    pub fn refund_transaction(&mut self) -> Result<Transaction, RefundError> {
        if self.amount.is_sign_negative() {
            return Err(RefundError::RechargeTransaction(self.uuid));
        }

        let to_refund_order_list = self.orders.clone();

        self.refund_transaction_partial(&to_refund_order_list)
    }

    /// 创建一个新的部分退款交易实例。
    ///
    /// Arguments:
    /// - `to_refund_orders`: 要退款的订单列表。
    ///
    /// Returns:
    /// - 成功时返回新的部分退款交易实例。
    /// - 失败时返回 `RefundError`。
    ///
    /// Notes:
    /// 调用者需要保证传入的订单是当前交易的订单
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

    /// 获取交易的 UUID。
    ///
    /// Returns:
    /// - 交易的 UUID。
    pub fn uuid(&self) -> Uuid {
        self.uuid
    }

    /// 获取交易创建时间。
    ///
    /// Returns:
    /// - 交易创建时间。
    pub fn create_time(&self) -> DateTimeWithTimeZone {
        self.create_time
    }

    /// 获取交易完成时间。
    ///
    /// Returns:
    /// - 交易完成时间，可能为空。
    pub fn finish_time(&self) -> Option<DateTimeWithTimeZone> {
        self.finish_time
    }

    /// 获取交易金额。
    ///
    /// Returns:
    /// - 交易金额。
    pub fn raw_amount(&self) -> Decimal {
        self.amount
    }

    /// 获取用户的唯一标识符。
    ///
    /// Returns:
    /// - 用户的唯一标识符。
    pub fn user_id(&self) -> UserId {
        self.user_id
    }

    /// 获取交易包含的订单列表。
    ///
    /// Returns:
    /// - 交易包含的订单列表的不可变引用。
    pub fn orders(&self) -> &[Box<dyn Order>] {
        &self.orders
    }

    /// 获取交易状态。
    ///
    /// Returns:
    /// - 交易状态。
    pub fn status(&self) -> TransactionStatus {
        self.status
    }

    /// 获取交易包含的订单列表并转移所有权。
    ///
    /// Returns:
    /// - 交易包含的订单列表。
    pub fn into_orders(self) -> Vec<Box<dyn Order>> {
        self.orders
    }
}
