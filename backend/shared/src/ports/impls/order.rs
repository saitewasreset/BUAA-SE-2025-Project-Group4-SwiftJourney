use crate::api::{ApiEndpoint, InternalApiError, OrderInternalServiceApi, SuperClient};
use crate::internal::order::command::{
    NewTransactionCommand, OrderByUuidQuery, RefundTransactionCommand, UpdateOrdersCommand,
    UserOrderListQuery, VerifyTrainOrderQuery,
};
use crate::internal::order::dto::InternalOrderDTO;
use crate::ports::order::OrderPort;
use async_trait::async_trait;
use tracing::error;
use uuid::Uuid;

pub struct HttpOrderPortImpl {
    super_client: SuperClient,
}

impl HttpOrderPortImpl {
    pub fn new(api_endpoint: ApiEndpoint) -> Self {
        let super_client = SuperClient::new(api_endpoint);

        Self { super_client }
    }
}

#[async_trait]
impl OrderPort for HttpOrderPortImpl {
    async fn new_transaction(
        &self,
        command: NewTransactionCommand,
    ) -> Result<Uuid, InternalApiError> {
        self.super_client
            .post(OrderInternalServiceApi::NewTransaction, command)
            .await
            .inspect_err(|e| error!("Failed to create new transaction: {:?}", e))
    }

    async fn refund_transaction(
        &self,
        command: RefundTransactionCommand,
    ) -> Result<Uuid, InternalApiError> {
        self.super_client
            .post(OrderInternalServiceApi::RefundTransaction, command)
            .await
            .inspect_err(|e| error!("Failed to refund transaction: {:?}", e))
    }

    async fn get_order_by_uuid(
        &self,
        query: OrderByUuidQuery,
    ) -> Result<Option<InternalOrderDTO>, InternalApiError> {
        let order_uuid = query.order_uuid;

        self.super_client
            .post(OrderInternalServiceApi::GetOrderByUuid, query)
            .await
            .inspect_err(|e| {
                error!(
                    "Failed to query order for uuid: {:?} uuid = {}",
                    e, order_uuid
                )
            })
    }

    async fn verify_train_order(
        &self,
        query: VerifyTrainOrderQuery,
    ) -> Result<bool, InternalApiError> {
        self.super_client
            .post(OrderInternalServiceApi::VerifyTrainOrder, query)
            .await
            .inspect_err(|e| error!("Failed to verify train order: {:?}", e))
    }

    async fn update_orders(&self, command: UpdateOrdersCommand) -> Result<(), InternalApiError> {
        self.super_client
            .post(OrderInternalServiceApi::UpdateOrders, command)
            .await
            .inspect_err(|e| error!("Failed to update orders: {:?}", e))
    }

    async fn get_order_list_by_user_id(
        &self,
        query: UserOrderListQuery,
    ) -> Result<Vec<InternalOrderDTO>, InternalApiError> {
        self.super_client
            .post(OrderInternalServiceApi::GetOrderListByUserId, query)
            .await
            .inspect_err(|e| error!("Failed to get order list by user id: {:?}", e))
    }
}
