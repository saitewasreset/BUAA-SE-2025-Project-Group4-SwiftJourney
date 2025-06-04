use super::transaction::TransactionInfoDTO;
use crate::application::ApplicationError;
use crate::application::commands::hotel_order::HotelOrderRequestsDTO;
use async_trait::async_trait;

#[async_trait]
pub trait HotelOrderService: 'static + Send + Sync {
    /// 处理酒店预订订单
    ///
    /// 此方法接收会话ID和酒店订单请求，验证并创建订单，然后创建交易
    async fn process_hotel_orders(
        &self,
        session_id: String,
        hotel_orders: HotelOrderRequestsDTO,
    ) -> Result<TransactionInfoDTO, Box<dyn ApplicationError>>;
}
