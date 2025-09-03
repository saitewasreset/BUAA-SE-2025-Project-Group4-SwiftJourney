use crate::domain::repository::transaction::TransactionRepository;
use crate::domain::service::order::OrderService;
use crate::domain::service::order::order_dto::TransactionDataDto;
use crate::domain::service::order_status::OrderStatusManagerService;
use crate::domain::service::transaction::{TransactionService, TransactionServiceError};
use async_trait::async_trait;
use rust_decimal::Decimal;
use rust_decimal::prelude::{ToPrimitive, Zero};
use shared::domain::ServiceError;
use shared::domain::model::order::{Order, OrderStatus};
use shared::domain::model::transaction::{Transaction, TransactionAmountAbs, TransactionError};
use shared::domain::model::user::UserId;
use shared::internal::user::command::UserInfoQuery;
use shared::ports::user::UserPort;
use std::collections::HashSet;
use std::sync::Arc;
use tracing::{debug, error, info, instrument};
use uuid::Uuid;

pub struct TransactionServiceImpl<UP, R, O, OS>
where
    UP: UserPort,
    R: TransactionRepository,
    O: OrderService,
    OS: OrderStatusManagerService,
{
    user_port: Arc<UP>,
    transaction_repository: Arc<R>,
    order_service: Arc<O>,
    order_status_manager_service: Arc<OS>,
}

impl<UP, R, O, OS> TransactionServiceImpl<UP, R, O, OS>
where
    UP: UserPort,
    R: TransactionRepository,
    O: OrderService,
    OS: OrderStatusManagerService,
{
    pub fn new(
        user_port: Arc<UP>,
        transaction_repository: Arc<R>,
        order_service: Arc<O>,
        order_status_manager_service: Arc<OS>,
    ) -> Self {
        Self {
            user_port,
            transaction_repository,
            order_service,
            order_status_manager_service,
        }
    }

    async fn check_user_exist(&self, user_id: UserId) -> Result<(), TransactionServiceError> {
        if self
            .user_port
            .get_user_info(UserInfoQuery {
                user_id: user_id.into(),
            })
            .await
            .inspect_err(|e| {
                error!("Failed to get user info: {:?}", e);
            })
            .map_err(|e| {
                TransactionServiceError::InfrastructureError(ServiceError::RelatedServiceError(
                    e.into(),
                ))
            })?
            .is_none()
        {
            Err(TransactionServiceError::InvalidUser(user_id))
        } else {
            Ok(())
        }
    }
}

#[async_trait]
impl<UP, R, O, OS> TransactionService for TransactionServiceImpl<UP, R, O, OS>
where
    UP: UserPort,
    R: TransactionRepository,
    O: OrderService,
    OS: OrderStatusManagerService,
{
    #[instrument(skip(self))]
    async fn recharge(
        &self,
        user_id: UserId,
        amount: TransactionAmountAbs,
    ) -> Result<Uuid, TransactionServiceError> {
        self.check_user_exist(user_id).await?;

        let mut tx = Transaction::new_recharge(user_id, amount);

        self.transaction_repository
            .save(&mut tx)
            .await
            .inspect_err(|e| error!("failed to save transaction: {}", e))?;

        Ok(tx.uuid())
    }

    #[instrument(skip(self))]
    async fn get_balance(&self, user_id: UserId) -> Result<Decimal, TransactionServiceError> {
        Ok(self
            .transaction_repository
            .get_user_balance(user_id)
            .await
            .inspect_err(|e| {
                error!("Failed to get user balance: {:?}", e);
            })?
            .unwrap_or(Decimal::zero()))
    }

    #[instrument(skip(self))]
    async fn new_transaction(
        &self,
        user_id: UserId,
        orders: Vec<Box<dyn Order>>,
        atomic: bool,
    ) -> Result<Uuid, TransactionServiceError> {
        self.check_user_exist(user_id).await?;

        for order in &*orders {
            if order.order_status() != OrderStatus::Unpaid {
                return Err(TransactionServiceError::InvalidOrderStatus {
                    op: "new",
                    status: order.order_status(),
                    order_id: order.uuid(),
                    transaction_id: None,
                });
            }
        }

        let mut tx = Transaction::new(user_id, orders.clone(), atomic);

        self.transaction_repository
            .save(&mut tx)
            .await
            .inspect_err(|e| {
                error!("Failed to save transaction: {:?}", e);
            })?;

        Ok(tx.uuid())
    }

    #[instrument(skip(self))]
    async fn pay_transaction(&self, transaction_id: Uuid) -> Result<(), TransactionServiceError> {
        info!("Paying transaction: {}", transaction_id);
        let mut tx = self
            .transaction_repository
            .find_by_uuid(transaction_id)
            .await
            .inspect_err(|e| {
                error!("Failed to find transaction: {:?}", e);
            })?
            .ok_or(TransactionServiceError::InvalidTransactionId(
                transaction_id,
            ))
            .inspect_err(|e| {
                error!("No transaction found: {:?}", e);
            })?;

        info!("Transaction loaded");

        let available_balance = self.get_balance(tx.user_id()).await.inspect_err(|e| {
            error!("Failed to get user balance: {:?}", e);
        })?;

        // 注意，若交易是充值/退款交易，其raw_amount < 0，将通过此检查，而在之后的pay方法中被拒绝
        if available_balance < tx.raw_amount() {
            return Err(TransactionServiceError::InsufficientFunds {
                transaction_id: tx.uuid(),
                balance: available_balance,
                amount: TransactionAmountAbs::from(tx.raw_amount()),
            });
        }

        info!("Balance check passed");

        tx.pay().map_err(|e| match e {
            TransactionError::AlreadyPaid(_) => TransactionServiceError::InvalidTransactionStatus {
                op: "pay",
                status: tx.status(),
                transaction_id: tx.uuid(),
            },
            _ => panic!("Unexpected error: {:?}", e),
        })?;

        debug!("saving paid transaction: {:?}", tx);

        for order in tx.orders_mut() {
            order.set_status(OrderStatus::Paid);
        }

        self.transaction_repository
            .save(&mut tx)
            .await
            .inspect_err(|e| {
                error!("Failed to save transaction: {:?}", e);
            })?;

        let orders = tx
            .orders()
            .iter()
            .map(|order| order.as_ref())
            .collect::<Vec<_>>();

        self.order_status_manager_service
            .notify_status_change(transaction_id, tx.atomic(), &orders, OrderStatus::Paid)
            .await;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn refund_transaction(
        &self,
        transaction_id: Uuid,
        to_refund_orders: &[Box<dyn Order>],
    ) -> Result<Uuid, TransactionServiceError> {
        let mut tx = self
            .transaction_repository
            .find_by_uuid(transaction_id)
            .await
            .inspect_err(|e| {
                error!("Failed to find transaction: {:?}", e);
            })?
            .ok_or(TransactionServiceError::InvalidTransactionId(
                transaction_id,
            ))?;

        let to_refund_order_uuid_set = to_refund_orders
            .iter()
            .map(|o| o.uuid())
            .collect::<HashSet<_>>();

        let mut refund_tx = tx.refund_transaction_partial(to_refund_orders)?;

        let refund_tx_id = self
            .transaction_repository
            .save(&mut refund_tx)
            .await
            .inspect_err(|e| {
                error!("Failed to save transaction: {:?}", e);
            })?;

        for order in tx.orders_mut() {
            if to_refund_order_uuid_set.contains(&order.uuid()) {
                order
                    .payment_info_mut()
                    .set_refund_transaction_id(refund_tx_id);
            }
        }

        self.transaction_repository
            .save(&mut tx)
            .await
            .inspect_err(|e| {
                error!("Failed to save transaction: {:?}", e);
            })?;

        let orders = tx
            .orders()
            .iter()
            .filter(|order| to_refund_order_uuid_set.contains(&order.uuid()))
            .map(|order| order.as_ref())
            .collect::<Vec<_>>();

        self.order_status_manager_service
            .notify_status_change(transaction_id, tx.atomic(), &orders, OrderStatus::Cancelled)
            .await;

        Ok(refund_tx.uuid())
    }

    #[instrument(skip(self))]
    async fn convert_transaction_to_dto(
        &self,
        transaction: Transaction,
    ) -> Result<TransactionDataDto, TransactionServiceError> {
        let mut dto = TransactionDataDto {
            transaction_id: transaction.uuid().to_string(),
            status: transaction.status().to_string(),
            create_time: transaction.create_time().to_rfc3339(),
            pay_time: transaction.finish_time().map(|dt| dt.to_rfc3339()),
            amount: transaction.amount().to_f64().unwrap_or(0.0),
            orders: Vec::new(),
        };

        let origin_orders = transaction.into_orders();

        let mut orders = Vec::with_capacity(origin_orders.len());

        for order in origin_orders {
            debug!("Converting order to DTO: {:?}", order);
            orders.push(self.order_service.convert_order_to_dto(order).await?)
        }

        dto.orders = orders;

        Ok(dto)
    }
}
