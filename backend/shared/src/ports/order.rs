use crate::api::InternalApiError;
use crate::internal::order::command::{
    NewTransactionCommand, OrderByUuidQuery, RefundTransactionCommand, UpdateOrdersCommand,
    UserOrderListQuery, VerifyTrainOrderQuery,
};
use crate::internal::order::dto::InternalOrderDTO;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait OrderPort: 'static + Send + Sync {
    async fn new_transaction(
        &self,
        command: NewTransactionCommand,
    ) -> Result<Uuid, InternalApiError>;

    async fn refund_transaction(
        &self,
        command: RefundTransactionCommand,
    ) -> Result<Uuid, InternalApiError>;

    async fn get_order_by_uuid(
        &self,
        query: OrderByUuidQuery,
    ) -> Result<Option<InternalOrderDTO>, InternalApiError>;

    async fn verify_train_order(
        &self,
        query: VerifyTrainOrderQuery,
    ) -> Result<bool, InternalApiError>;

    async fn update_orders(&self, command: UpdateOrdersCommand) -> Result<(), InternalApiError>;

    async fn get_order_list_by_user_id(
        &self,
        query: UserOrderListQuery,
    ) -> Result<Vec<InternalOrderDTO>, InternalApiError>;
}
