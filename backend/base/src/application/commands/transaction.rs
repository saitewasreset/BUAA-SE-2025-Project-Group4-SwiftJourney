use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub struct RechargeCommand {
    pub session_id: String,
    pub amount: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BalanceQuery {
    pub session_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TransactionQuery {
    pub session_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TransactionDetailQuery {
    pub session_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SetPaymentPasswordCommand {
    pub session_id: String,
    pub user_password: String,
    pub payment_password: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PayTransactionCommand {
    pub session_id: String,
    pub transaction_id: Uuid,
    pub user_password: Option<String>,
    pub payment_password: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OrderListQuery {
    pub session_id: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GenerateDebugTransactionCommand {
    pub session_id: String,
    pub amount: f64,
}
