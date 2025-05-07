use crate::domain::model::order::{Order, OrderStatus};
use crate::domain::model::transaction::{Transaction, TransactionAmountAbs, TransactionError};
use crate::domain::model::user::UserId;
use crate::domain::repository::transaction::TransactionRepository;
use crate::domain::repository::user::UserRepository;
use crate::domain::service::order_status::OrderStatusManagerService;
use crate::domain::service::transaction::{TransactionService, TransactionServiceError};
use async_trait::async_trait;
use rust_decimal::Decimal;
use std::sync::Arc;
use uuid::Uuid;

pub struct TransactionServiceImpl<U, R, OS>
where
    U: UserRepository,
    R: TransactionRepository,
    OS: OrderStatusManagerService,
{
    user_repository: Arc<U>,
    transaction_repository: Arc<R>,
    order_status_manager_service: Arc<OS>,
}

impl<U, R, OS> TransactionServiceImpl<U, R, OS>
where
    U: UserRepository,
    R: TransactionRepository,
    OS: OrderStatusManagerService,
{
    pub fn new(
        user_repository: Arc<U>,
        transaction_repository: Arc<R>,
        order_status_manager_service: Arc<OS>,
    ) -> Self {
        Self {
            user_repository,
            transaction_repository,
            order_status_manager_service,
        }
    }
}

#[async_trait]
impl<U, R, OS> TransactionService for TransactionServiceImpl<U, R, OS>
where
    U: UserRepository,
    R: TransactionRepository,
    OS: OrderStatusManagerService,
{
    async fn recharge(
        &self,
        user_id: UserId,
        amount: TransactionAmountAbs,
    ) -> Result<Uuid, TransactionServiceError> {
        if self.user_repository.find(user_id).await?.is_none() {
            return Err(TransactionServiceError::InvalidUser(user_id));
        }

        let mut tx = Transaction::new_recharge(user_id, amount);

        self.transaction_repository.save(&mut tx).await?;

        Ok(tx.uuid())
    }

    async fn get_balance(&self, user_id: UserId) -> Result<Decimal, TransactionServiceError> {
        self.transaction_repository
            .get_user_balance(user_id)
            .await?
            .ok_or(TransactionServiceError::InvalidUser(user_id))
    }

    async fn new_transaction(
        &self,
        user_id: UserId,
        orders: Vec<Box<dyn Order>>,
    ) -> Result<Uuid, TransactionServiceError> {
        for order in &orders {
            if order.order_status() != OrderStatus::Unpaid {
                return Err(TransactionServiceError::InvalidOrderStatus {
                    op: "new",
                    status: order.order_status(),
                    order_id: order.uuid(),
                    transaction_id: None,
                });
            }
        }

        if self.user_repository.find(user_id).await?.is_none() {
            return Err(TransactionServiceError::InvalidUser(user_id));
        }

        let mut tx = Transaction::new(user_id, orders.clone());

        self.transaction_repository.save(&mut tx).await?;

        for order in &orders {
            self.order_status_manager_service
                .attach(order.as_ref())
                .await
                .expect("Order with status: Unpaid should be attached");
        }

        Ok(tx.uuid())
    }

    async fn pay_transaction(&self, transaction_id: Uuid) -> Result<(), TransactionServiceError> {
        let mut tx = self
            .transaction_repository
            .find_by_uuid(transaction_id)
            .await?
            .ok_or(TransactionServiceError::InvalidTransactionId(
                transaction_id,
            ))?;

        let available_balance = self.get_balance(tx.user_id()).await?;

        // 注意，若交易是充值/退款交易，其raw_amount < 0，将通过此检查，而在之后的pay方法中被拒绝
        if available_balance < tx.raw_amount() {
            return Err(TransactionServiceError::InsufficientFunds {
                transaction_id: tx.uuid(),
                balance: available_balance,
                amount: TransactionAmountAbs::from(tx.raw_amount()),
            });
        }

        tx.pay().map_err(|e| match e {
            TransactionError::AlreadyPaid(_) => TransactionServiceError::InvalidTransactionStatus {
                op: "pay",
                status: tx.status(),
                transaction_id: tx.uuid(),
            },
            _ => panic!("Unexpected error: {:?}", e),
        })?;

        self.transaction_repository.save(&mut tx).await?;

        for order in tx.orders() {
            self.order_status_manager_service
                .notify_status_change(order.uuid(), OrderStatus::Paid)
                .await;
        }

        Ok(())
    }

    async fn refund_transaction(
        &self,
        transaction_id: Uuid,
        to_refund_orders: &[Box<dyn Order>],
    ) -> Result<Uuid, TransactionServiceError> {
        let mut tx = self
            .transaction_repository
            .find_by_uuid(transaction_id)
            .await?
            .ok_or(TransactionServiceError::InvalidTransactionId(
                transaction_id,
            ))?;

        let refund_tx = tx.refund_transaction_partial(to_refund_orders)?;

        self.transaction_repository.save(&mut tx).await?;

        for order in to_refund_orders {
            self.order_status_manager_service
                .notify_status_change(order.uuid(), OrderStatus::Cancelled)
                .await;
        }

        Ok(refund_tx.uuid())
    }
}
