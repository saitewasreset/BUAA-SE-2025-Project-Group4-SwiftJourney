use async_trait::async_trait;
use shared::application_error::ApplicationError;
use crate::application::service::train_order::{TrainOrderService, OrderPackDTO};
use shared::internal::order::dto::TransactionInfoDTO;

pub struct TrainOrderServiceImpl;

#[async_trait]
impl TrainOrderService for TrainOrderServiceImpl {
    async fn process_train_order_packs(
        &self,
        _session_id: String,
        _order_packs: Vec<OrderPackDTO>,
    ) -> Result<TransactionInfoDTO, Box<dyn ApplicationError>> {
        unimplemented!()
    }
}