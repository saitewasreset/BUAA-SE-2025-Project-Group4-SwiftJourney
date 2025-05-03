use crate::domain::model::transaction::TransactionId;
use sea_orm::prelude::TimeDateTimeWithTimeZone;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OrderStatus {
    /// 订单已经生成，还未支付
    Unpaid,
    /// 订单已经生成，且支付成功，此时订单将进入后端消息队列；系统处理完成后，根据实际情况进入“未出行”或“失败”状态（如是否有可用座位）
    Paid,
    /// 订单已被后端处理，且成功（例如，火车订票有符合条件的座位，订票成功），还没有出行
    Ongoing,
    /// 行程正在进行（例如，火车订票已经过始发站，还未到达终到站）
    Active,
    /// 行程已经完成（例如，火车订票已到达终到站）
    Completed,
    /// 订单已被后端处理，且失败（例如，火车订票没有符合条件的座位，订票失败）
    Failed,
    /// 订单被用户取消
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OrderId(u64);

pub struct OrderTimeInfo {
    create_time: TimeDateTimeWithTimeZone,
    active_time: TimeDateTimeWithTimeZone,
    complete_time: TimeDateTimeWithTimeZone,
}

pub struct PaymentInfo {
    pay_transaction_id: TransactionId,
    refund_transaction_id: Option<TransactionId>,
}

pub struct TrainOrder {}
