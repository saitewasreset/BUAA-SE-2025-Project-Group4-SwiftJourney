use actix_web::{App, test, web};
use api::hotel;
use async_trait::async_trait;
use base::application::ApplicationError;
use base::application::commands::hotel_order::HotelOrderRequestsDTO;
use base::application::service::hotel_order::HotelOrderService;
use base::application::service::transaction::TransactionInfoDTO;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

struct MockHotelOrderService {
    fail: bool,
}

#[async_trait]
impl HotelOrderService for MockHotelOrderService {
    async fn process_hotel_orders(
        &self,
        _session_id: String,
        _hotel_orders: HotelOrderRequestsDTO,
    ) -> Result<TransactionInfoDTO, Box<dyn ApplicationError>> {
        if self.fail {
            Err(Box::new(TestError {
                code: 500,
                msg: "order processing failed".into(),
            }))
        } else {
            Ok(TransactionInfoDTO {
                transaction_id: Uuid::parse_str("11111111-2222-3333-4444-555555555555").unwrap(),
                amount: 199.99,
                status: "OK".to_string(),
            })
        }
    }
}

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
    fn error_code(&self) -> u32 {
        self.code
    }
    fn error_message(&self) -> String {
        self.msg.clone()
    }
}

#[actix_rt::test]
async fn create_hotel_order_success() {
    let app = test::init_service(
        App::new()
            .service(
                web::scope("/api").service(web::scope("/hotel").configure(hotel::scoped_config)),
            )
            .app_data(web::Data::from(
                Arc::new(MockHotelOrderService { fail: false }) as Arc<dyn HotelOrderService>,
            )),
    )
    .await;

    let payload = json!([
        {
            "hotelId": Uuid::parse_str("aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee").unwrap().to_string(),
            "roomType": "Deluxe",
            "beginDate": "2025-09-01",
            "endDate": "2025-09-03",
            "personalId": Uuid::parse_str("ffffffff-0000-1111-2222-333333333333").unwrap().to_string(),
            "amount": 1
        }
    ]);

    let req = test::TestRequest::post()
        .uri("/api/hotel/order")
        .cookie(actix_web::cookie::Cookie::new("session_id", "test_session"))
        .set_json(&payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(value["code"], 200);
    assert_eq!(value["message"], "For Super Earth!");
    let data = &value["data"];
    assert!(data.is_object());
    assert_eq!(
        data["transactionId"],
        "11111111-2222-3333-4444-555555555555"
    );
    assert_eq!(data["amount"], 199.99);
    // API 层固定返回 unpaid（等待支付）
    assert_eq!(data["status"], "unpaid");
}

#[actix_rt::test]
async fn create_hotel_order_service_fail() {
    let app = test::init_service(
        App::new()
            .service(
                web::scope("/api").service(web::scope("/hotel").configure(hotel::scoped_config)),
            )
            .app_data(web::Data::from(
                Arc::new(MockHotelOrderService { fail: true }) as Arc<dyn HotelOrderService>,
            )),
    )
    .await;

    let payload = json!([
        {
            "hotelId": Uuid::parse_str("aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee").unwrap().to_string(),
            "roomType": "Deluxe",
            "beginDate": "2025-09-01",
            "endDate": "2025-09-03",
            "personalId": Uuid::parse_str("ffffffff-0000-1111-2222-333333333333").unwrap().to_string(),
            "amount": 1
        }
    ]);

    let req = test::TestRequest::post()
        .uri("/api/hotel/order")
        .cookie(actix_web::cookie::Cookie::new("session_id", "test_session"))
        .set_json(&payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(value["code"], 500);
    assert_eq!(value["message"], "order processing failed");
    assert!(value["data"].is_null());
}

#[actix_rt::test]
async fn create_hotel_order_no_session_cookie() {
    let app = test::init_service(
        App::new()
            .service(
                web::scope("/api").service(web::scope("/hotel").configure(hotel::scoped_config)),
            )
            .app_data(web::Data::from(
                Arc::new(MockHotelOrderService { fail: false }) as Arc<dyn HotelOrderService>,
            )),
    )
    .await;

    let payload = json!([]);

    let req = test::TestRequest::post()
        .uri("/api/hotel/order")
        .set_json(&payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(value["code"], 403);
    assert_eq!(
        value["message"],
        "Sorry, but this was meant to be a private game: no session id provided"
    );
    assert!(value["data"].is_null());
}

#[actix_rt::test]
async fn create_hotel_order_invalid_json() {
    let app = test::init_service(
        App::new()
            .service(
                web::scope("/api").service(web::scope("/hotel").configure(hotel::scoped_config)),
            )
            .app_data(web::Data::from(
                Arc::new(MockHotelOrderService { fail: false }) as Arc<dyn HotelOrderService>,
            )),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/hotel/order")
        .cookie(actix_web::cookie::Cookie::new("session_id", "test_session"))
        .insert_header(("Content-Type", "application/json"))
        .set_payload("not-json")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(value["code"], 400);
    assert_eq!(value["message"], "invalid json");
    assert!(value["data"].is_null());
}
