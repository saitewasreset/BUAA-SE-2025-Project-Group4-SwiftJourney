use crate::application::commands::transaction::{
    BalanceQuery, CancelOrderCommand, GenerateDebugTransactionCommand, PayTransactionCommand,
    RechargeCommand, SetPaymentPasswordCommand, TransactionDetailQuery, TransactionQuery,
};
use crate::application::{ApplicationError, GeneralError};
use crate::domain::model::transaction::Transaction;
use crate::domain::service::order::order_dto::TransactionDataDto;
use crate::domain::service::transaction::TransactionServiceError;
use async_trait::async_trait;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct RechargeDTO {
    pub amount: f64,
    #[serde(rename = "externalPaymentId")]
    pub external_payment_id: Option<Uuid>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PaymentConfirmationDTO {
    #[serde(rename = "userPassword")]
    pub user_password: Option<String>,
    #[serde(rename = "paymentPassword")]
    pub payment_password: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PaymentPasswordInfoDTO {
    #[serde(rename = "userPassword")]
    pub user_password: String,
    #[serde(rename = "paymentPassword")]
    pub payment_password: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TransactionGenerateDTO {
    pub amount: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BalanceInfoDTO {
    pub balance: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TransactionInfoDTO {
    pub transaction_id: Uuid,
    pub amount: f64,
    pub status: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CancelOrderDTO {
    pub order_id: Uuid,
}

impl From<Transaction> for TransactionInfoDTO {
    fn from(value: Transaction) -> Self {
        TransactionInfoDTO {
            transaction_id: value.uuid(),
            amount: value.raw_amount().to_f64().unwrap(),
            status: value.status().to_string(),
        }
    }
}

#[derive(Error, Debug)]
pub enum TransactionApplicationServiceError {
    #[error("wrong payment password")]
    WrongPaymentPassword,
    #[error("wrong user password")]
    WrongUserPassword,
    #[error("too many payment password attempts")]
    TooManyPaymentPasswordAttempts,
    #[error("insufficient funds")]
    InsufficientFunds,
    #[error("cannot refund this transaction: {0}")]
    RefundError(String),
    #[error("{0}")]
    InvalidTransactionStatus(String),
    #[error("invalid payment password format")]
    InvalidPaymentPasswordFormat,
}

impl From<TransactionServiceError> for Box<dyn ApplicationError> {
    fn from(value: TransactionServiceError) -> Self {
        match value {
            TransactionServiceError::InvalidUser(x) => {
                Box::new(GeneralError::BadRequest(format!("invalid user id: {}", x)))
            }
            TransactionServiceError::InvalidTransactionId(x) => {
                Box::new(GeneralError::BadRequest(format!("invalid user id: {}", x)))
            }
            e @ TransactionServiceError::InvalidTransactionStatus {
                op: _,
                status: _,
                transaction_id: _,
            } => Box::new(
                TransactionApplicationServiceError::InvalidTransactionStatus(e.to_string()),
            ),
            TransactionServiceError::InsufficientFunds {
                transaction_id: _,
                balance: _,
                amount: _,
            } => Box::new(TransactionApplicationServiceError::InsufficientFunds),
            e @ TransactionServiceError::RefundError(..) => Box::new(
                TransactionApplicationServiceError::RefundError(e.to_string()),
            ),
            _ => Box::new(GeneralError::InternalServerError),
        }
    }
}

impl ApplicationError for TransactionApplicationServiceError {
    fn error_code(&self) -> u32 {
        match self {
            TransactionApplicationServiceError::WrongPaymentPassword => 11001,
            TransactionApplicationServiceError::WrongUserPassword => 11002,
            TransactionApplicationServiceError::TooManyPaymentPasswordAttempts => 11003,
            TransactionApplicationServiceError::InsufficientFunds => 11004,
            TransactionApplicationServiceError::RefundError(_) => 11005,
            TransactionApplicationServiceError::InvalidTransactionStatus(_) => 11006,
            TransactionApplicationServiceError::InvalidPaymentPasswordFormat => 11007,
        }
    }

    fn error_message(&self) -> String {
        self.to_string()
    }
}

#[async_trait]
pub trait TransactionApplicationService: 'static + Send + Sync {
    async fn recharge(&self, command: RechargeCommand) -> Result<(), Box<dyn ApplicationError>>;

    async fn query_balance(
        &self,
        query: BalanceQuery,
    ) -> Result<BalanceInfoDTO, Box<dyn ApplicationError>>;

    async fn query_transactions(
        &self,
        query: TransactionQuery,
    ) -> Result<Vec<TransactionInfoDTO>, Box<dyn ApplicationError>>;

    async fn set_payment_password(
        &self,
        command: SetPaymentPasswordCommand,
    ) -> Result<(), Box<dyn ApplicationError>>;

    async fn pay_transaction(
        &self,
        command: PayTransactionCommand,
    ) -> Result<(), Box<dyn ApplicationError>>;

    async fn generate_debug_transaction(
        &self,
        command: GenerateDebugTransactionCommand,
    ) -> Result<TransactionInfoDTO, Box<dyn ApplicationError>>;

    async fn query_transaction_details(
        &self,
        query: TransactionDetailQuery,
    ) -> Result<Vec<TransactionDataDto>, Box<dyn ApplicationError>>;

    async fn cancel_order(
        &self,
        command: CancelOrderCommand,
    ) -> Result<(), Box<dyn ApplicationError>>;
}



#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use uuid::Uuid;

    // ===== Mock 实现：成功版本 =====
    struct MockSuccessTransactionService;

    #[async_trait]
    impl TransactionApplicationService for MockSuccessTransactionService {
        async fn recharge(&self, _command: RechargeCommand) -> Result<(), Box<dyn ApplicationError>> {
            Ok(())
        }

        async fn query_balance(
            &self,
            _query: BalanceQuery,
        ) -> Result<BalanceInfoDTO, Box<dyn ApplicationError>> {
            Ok(BalanceInfoDTO { balance: 100.0 })
        }

        async fn query_transactions(
            &self,
            _query: TransactionQuery,
        ) -> Result<Vec<TransactionInfoDTO>, Box<dyn ApplicationError>> {
            Ok(vec![TransactionInfoDTO {
                transaction_id: Uuid::new_v4(),
                amount: 50.0,
                status: "SUCCESS".to_string(),
            }])
        }

        async fn set_payment_password(
            &self,
            _command: SetPaymentPasswordCommand,
        ) -> Result<(), Box<dyn ApplicationError>> {
            Ok(())
        }

        async fn pay_transaction(
            &self,
            _command: PayTransactionCommand,
        ) -> Result<(), Box<dyn ApplicationError>> {
            Ok(())
        }

        async fn generate_debug_transaction(
            &self,
            _command: GenerateDebugTransactionCommand,
        ) -> Result<TransactionInfoDTO, Box<dyn ApplicationError>> {
            Ok(TransactionInfoDTO {
                transaction_id: Uuid::new_v4(),
                amount: 999.0,
                status: "DEBUG".to_string(),
            })
        }

        async fn query_transaction_details(
            &self,
            _query: TransactionDetailQuery,
        ) -> Result<Vec<TransactionDataDto>, Box<dyn ApplicationError>> {
            Ok(vec![])
        }

        async fn cancel_order(
            &self,
            _command: CancelOrderCommand,
        ) -> Result<(), Box<dyn ApplicationError>> {
            Ok(())
        }
    }

    // ===== Mock 实现：失败版本 =====
    struct MockFailTransactionService;

    #[async_trait]
    impl TransactionApplicationService for MockFailTransactionService {
        async fn recharge(&self, _command: RechargeCommand) -> Result<(), Box<dyn ApplicationError>> {
            Err(Box::new(TransactionApplicationServiceError::InsufficientFunds))
        }

        async fn query_balance(
            &self,
            _query: BalanceQuery,
        ) -> Result<BalanceInfoDTO, Box<dyn ApplicationError>> {
            Err(Box::new(TransactionApplicationServiceError::WrongUserPassword))
        }

        async fn query_transactions(
            &self,
            _query: TransactionQuery,
        ) -> Result<Vec<TransactionInfoDTO>, Box<dyn ApplicationError>> {
            Err(Box::new(TransactionApplicationServiceError::InvalidTransactionStatus("FAILED".to_string())))
        }

        async fn set_payment_password(
            &self,
            _command: SetPaymentPasswordCommand,
        ) -> Result<(), Box<dyn ApplicationError>> {
            Err(Box::new(TransactionApplicationServiceError::InvalidPaymentPasswordFormat))
        }

        async fn pay_transaction(
            &self,
            _command: PayTransactionCommand,
        ) -> Result<(), Box<dyn ApplicationError>> {
            Err(Box::new(TransactionApplicationServiceError::WrongPaymentPassword))
        }

        async fn generate_debug_transaction(
            &self,
            _command: GenerateDebugTransactionCommand,
        ) -> Result<TransactionInfoDTO, Box<dyn ApplicationError>> {
            Err(Box::new(TransactionApplicationServiceError::RefundError("debug failed".into())))
        }

        async fn query_transaction_details(
            &self,
            _query: TransactionDetailQuery,
        ) -> Result<Vec<TransactionDataDto>, Box<dyn ApplicationError>> {
            Err(Box::new(TransactionApplicationServiceError::TooManyPaymentPasswordAttempts))
        }

        async fn cancel_order(
            &self,
            _command: CancelOrderCommand,
        ) -> Result<(), Box<dyn ApplicationError>> {
            Err(Box::new(TransactionApplicationServiceError::InvalidTransactionStatus("CANNOT_CANCEL".into())))
        }
    }

    // ===== 正反测试样例 =====

    #[tokio::test]
    async fn test_recharge_success() {
        let service = MockSuccessTransactionService;
        assert!(service.recharge(RechargeCommand {
            session_id: "123456".to_string(),
            amount: 100.0
        }).await.is_ok());
    }

    #[tokio::test]
    async fn test_recharge_failure() {
        let service = MockFailTransactionService;
        let err = service.recharge(RechargeCommand {
            session_id: "123456".to_string(),
            amount: 100.0
        }).await.unwrap_err();
        assert_eq!(err.error_code(), 11004);
    }

    #[tokio::test]
    async fn test_query_balance_success() {
        let service = MockSuccessTransactionService;
        let dto = service.query_balance(BalanceQuery { session_id: "123456".to_string() }).await.unwrap();
        assert_eq!(dto.balance, 100.0);
    }

    #[tokio::test]
    async fn test_query_balance_failure() {
        let service = MockFailTransactionService;
        let err = service.query_balance(BalanceQuery { session_id: "123456".to_string() }).await.unwrap_err();
        assert_eq!(err.error_code(), 11002);
    }

    #[tokio::test]
    async fn test_query_transactions_success() {
        let service = MockSuccessTransactionService;
        let list = service.query_transactions(TransactionQuery { session_id: "123456".to_string() }).await.unwrap();
        assert_eq!(list.len(), 1);
    }

    #[tokio::test]
    async fn test_query_transactions_failure() {
        let service = MockFailTransactionService;
        let err = service.query_transactions(TransactionQuery { session_id: "123456".to_string() }).await.unwrap_err();
        assert_eq!(err.error_code(), 11006);
    }

    #[tokio::test]
    async fn test_set_payment_password_success() {
        let service = MockSuccessTransactionService;
        assert!(service.set_payment_password(SetPaymentPasswordCommand { session_id: "123456".to_string(), user_password: "123456".into(), payment_password: "123456".into() }).await.is_ok());
    }

    #[tokio::test]
    async fn test_set_payment_password_failure() {
        let service = MockFailTransactionService;
        let err = service.set_payment_password(SetPaymentPasswordCommand { session_id: "123456".to_string(), user_password: "123456".into(), payment_password: "123456".into() }).await.unwrap_err();
        assert_eq!(err.error_code(), 11007);
    }

    #[tokio::test]
    async fn test_pay_transaction_success() {
        let service = MockSuccessTransactionService;
        assert!(service.pay_transaction(PayTransactionCommand {
            session_id: "123456".to_string(),
            transaction_id: Uuid::new_v4(),
            user_password: Some("123456".into()),
            payment_password: Some("123456".into()),
        }).await.is_ok());
    }

    #[tokio::test]
    async fn test_pay_transaction_failure() {
        let service = MockFailTransactionService;
        let err = service.pay_transaction(PayTransactionCommand {
            session_id: "123456".to_string(),
            transaction_id: Uuid::new_v4(),
            user_password: Some("123456".into()),
            payment_password: Some("123456".into()),
        }).await.unwrap_err();
        assert_eq!(err.error_code(), 11001);
    }

    #[tokio::test]
    async fn test_generate_debug_transaction_success() {
        let service = MockSuccessTransactionService;
        let dto = service.generate_debug_transaction(GenerateDebugTransactionCommand {
            session_id: "123456".to_string(),
            amount: 999.0
        }).await.unwrap();
        assert_eq!(dto.amount, 999.0);
    }

    #[tokio::test]
    async fn test_generate_debug_transaction_failure() {
        let service = MockFailTransactionService;
        let err = service.generate_debug_transaction(GenerateDebugTransactionCommand {
            session_id: "123456".to_string(),
            amount: 999.0
        }).await.unwrap_err();
        assert_eq!(err.error_code(), 11005);
    }

    #[tokio::test]
    async fn test_query_transaction_details_success() {
        let service = MockSuccessTransactionService;
        let list = service.query_transaction_details(TransactionDetailQuery {
            session_id: "123456".to_string(),
        }).await.unwrap();
        assert_eq!(list.len(), 0);
    }

    #[tokio::test]
    async fn test_cancel_order_success() {
        let service = MockSuccessTransactionService;
        assert!(service.cancel_order(CancelOrderCommand {
            session_id: "123456".to_string(),
            order_id: Uuid::new_v4()
        }).await.is_ok());
    }

    #[tokio::test]
    async fn test_cancel_order_failure() {
        let service = MockFailTransactionService;
        let err = service.cancel_order(CancelOrderCommand {
            session_id: "123456".to_string(),
            order_id: Uuid::new_v4()
        }).await.unwrap_err();
        assert_eq!(err.error_code(), 11006);
    }

    // ===== 非异步函数：error_code & error_message 测试 =====
    #[test]
    fn test_error_code_and_message() {
        let e = TransactionApplicationServiceError::InsufficientFunds;
        assert_eq!(e.error_code(), 11004);
        assert_eq!(e.error_message(), "insufficient funds");
    }

    #[test]
    fn test_error_code_and_message_refund() {
        let e = TransactionApplicationServiceError::RefundError("refund denied".into());
        assert_eq!(e.error_code(), 11005);
    }
}
