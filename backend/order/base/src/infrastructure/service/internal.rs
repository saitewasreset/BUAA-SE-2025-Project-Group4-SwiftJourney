use crate::application::service::internal::{OrderInternalService, OrderInternalServiceError};
use crate::domain::repository::order::OrderRepository;
use crate::domain::service::order::OrderService;
use crate::domain::service::transaction::TransactionService;
use async_trait::async_trait;
use shared::domain::model::order::Order;
use shared::domain::model::user::UserId;
use shared::internal::order::command::{
    NewTransactionCommand, OrderByUuidQuery, RefundTransactionCommand, UpdateOrdersCommand,
    UserOrderListQuery, VerifyTrainOrderQuery,
};
use shared::internal::order::dto::InternalOrderDTO;
use std::sync::Arc;
use tracing::error;
use uuid::Uuid;

pub struct OrderInternalServiceImpl<TS, OS, OR>
where
    TS: TransactionService,
    OS: OrderService,
    OR: OrderRepository,
{
    transaction_service: Arc<TS>,
    order_service: Arc<OS>,
    order_repository: Arc<OR>,
}

impl<TS, OS, OR> OrderInternalServiceImpl<TS, OS, OR>
where
    TS: TransactionService,
    OS: OrderService,
    OR: OrderRepository,
{
    pub fn new(
        transaction_service: Arc<TS>,
        order_service: Arc<OS>,
        order_repository: Arc<OR>,
    ) -> Self {
        Self {
            transaction_service,
            order_service,
            order_repository,
        }
    }
}

#[async_trait]
impl<TS, OS, OR> OrderInternalService for OrderInternalServiceImpl<TS, OS, OR>
where
    TS: TransactionService,
    OS: OrderService,
    OR: OrderRepository,
{
    async fn new_transaction(
        &self,
        command: NewTransactionCommand,
    ) -> Result<Uuid, OrderInternalServiceError> {
        let dyn_orders = command
            .orders
            .into_iter()
            .map(|order| order.into())
            .collect();

        let transaction_uuid = self
            .transaction_service
            .new_transaction(UserId::from(command.user_id), dyn_orders, command.atomic)
            .await
            .inspect_err(|e| error!("Failed to create a transaction: {:?}", e))?;

        Ok(transaction_uuid)
    }

    async fn refund_transaction(
        &self,
        command: RefundTransactionCommand,
    ) -> Result<Uuid, OrderInternalServiceError> {
        let dyn_orders = command
            .to_refund_orders
            .into_iter()
            .map(|order| order.into())
            .collect::<Vec<_>>();

        let transaction_uuid = self
            .transaction_service
            .refund_transaction(command.transaction_id, &dyn_orders)
            .await
            .inspect_err(|e| error!("Failed to create a transaction: {:?}", e))?;

        Ok(transaction_uuid)
    }

    async fn get_order_by_uuid(
        &self,
        query: OrderByUuidQuery,
    ) -> Result<Option<InternalOrderDTO>, OrderInternalServiceError> {
        let dyn_order_opt = self
            .order_repository
            .load_order_by_uuid(query.order_uuid)
            .await
            .inspect_err(|e| {
                error!(
                    "Failed to get order by uuid: {:?} uuid = {}",
                    e, query.order_uuid
                )
            })
            .map_err(|e| OrderInternalServiceError::RelatedServiceError(e.into()))?;

        Ok(dyn_order_opt.map(|x| x.as_ref().into()))
    }

    async fn verify_train_order(
        &self,
        query: VerifyTrainOrderQuery,
    ) -> Result<bool, OrderInternalServiceError> {
        let result = self
            .order_service
            .verify_train_order(
                UserId::from(query.user_id),
                query.train_number,
                query.origin_departure_time,
            )
            .await
            .inspect_err(|e| error!("Failed to verify train order: {:?}", e))
            .map_err(|e| OrderInternalServiceError::RelatedServiceError(e.into()))?;

        Ok(result)
    }

    async fn update_orders(
        &self,
        command: UpdateOrdersCommand,
    ) -> Result<(), OrderInternalServiceError> {
        let dyn_orders: Vec<Box<dyn Order>> = command
            .orders
            .into_iter()
            .map(|order| order.into())
            .collect();

        for order in dyn_orders {
            let order_id = order.order_id();
            self.order_repository
                .update(order)
                .await
                .inspect_err(|e| {
                    error!("Failed to update order: {:?} order_id = {:?}", e, order_id)
                })
                .map_err(|e| OrderInternalServiceError::RelatedServiceError(e.into()))?;
        }

        Ok(())
    }

    async fn get_order_list_by_user_id(
        &self,
        query: UserOrderListQuery,
    ) -> Result<Vec<InternalOrderDTO>, OrderInternalServiceError> {
        let dyn_orders = self
            .order_repository
            .load_orders_by_user_id(UserId::from(query.user_id))
            .await
            .inspect_err(|e| error!("Failed to load orders: {:?} user_id = {}", e, query.user_id))
            .map_err(|e| OrderInternalServiceError::RelatedServiceError(e.into()))?;

        Ok(dyn_orders
            .iter()
            .map(|order| order.as_ref().into())
            .collect())
    }
}
