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



#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use uuid::Uuid;
    use crate::application::commands::hotel_order::HotelOrderRequestDTO;

    // ---- 测试服务实现 ----
    struct TestHotelOrderService {
        fail_process: bool,
    }

    #[async_trait]
    impl HotelOrderService for TestHotelOrderService {
        async fn process_hotel_orders(
            &self,
            _session_id: String,
            _hotel_orders: HotelOrderRequestsDTO,
        ) -> Result<TransactionInfoDTO, Box<dyn ApplicationError>> {
            if self.fail_process {
                Err(Box::new(TestError { code: 500, msg: "Order processing failed".into() }))
            } else {
                Ok(TransactionInfoDTO {
                    transaction_id: Uuid::new_v4(),
                    amount: 100.0,
                    status: "SUCCESS".into(),
                })
            }
        }
    }

    // ---- 测试用 ApplicationError ----
    #[derive(Debug)]
    struct TestError {
        code: u32,
        msg: String,
    }

    impl std::fmt::Display for TestError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.msg)
        }
    }

    impl std::error::Error for TestError {}

    impl ApplicationError for TestError {
        fn error_code(&self) -> u32 { self.code }
        fn error_message(&self) -> String { self.msg.clone() }
    }

    // ---- 测试用例 ----
    #[tokio::test]
    async fn test_process_hotel_orders_success() {
        let service = TestHotelOrderService { fail_process: false };

        let order = HotelOrderRequestDTO {
            hotel_id: Uuid::new_v4().to_string(),
            room_type: "Deluxe".into(),
            begin_date: Some("2025-09-01".into()),
            end_date: Some("2025-09-03".into()),
            personal_id: Uuid::new_v4().to_string(),
            amount: 2,
        };

        let orders: HotelOrderRequestsDTO = vec![order];

        let res = service.process_hotel_orders("session1".into(), orders).await;
        assert!(res.is_ok());
        let txn = res.unwrap();
        assert_eq!(txn.status, "SUCCESS");
        assert!(txn.amount > 0.0);
    }

    #[tokio::test]
    async fn test_process_hotel_orders_failure() {
        let service = TestHotelOrderService { fail_process: true };

        let order = HotelOrderRequestDTO {
            hotel_id: Uuid::new_v4().to_string(),
            room_type: "Deluxe".into(),
            begin_date: Some("2025-09-01".into()),
            end_date: Some("2025-09-03".into()),
            personal_id: Uuid::new_v4().to_string(),
            amount: 2,
        };

        let orders: HotelOrderRequestsDTO = vec![order];

        let res = service.process_hotel_orders("session2".into(), orders).await;
        assert!(res.is_err());
        let err = res.err().unwrap();
        assert_eq!(err.error_message(), "Order processing failed");
    }
}
