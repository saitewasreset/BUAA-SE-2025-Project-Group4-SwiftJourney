use actix_web::{App, test, web};
use api::train;
use base::application::commands::train_query::DirectTrainQueryCommand;
use base::application::service::train_query::{DirectTrainQueryDTO, TrainQueryService};
use serde_json::json;
use std::sync::Arc;

// A minimal mock for TrainQueryService to make responses deterministic.
struct MockTrainQueryService;

#[async_trait::async_trait]
impl TrainQueryService for MockTrainQueryService {
    async fn query_direct_trains(
        &self,
        _command: DirectTrainQueryCommand,
    ) -> Result<DirectTrainQueryDTO, Box<dyn base::application::ApplicationError>> {
        Ok(DirectTrainQueryDTO { solutions: vec![] })
    }

    async fn query_transfer_trains(
        &self,
        _command: base::application::commands::train_query::TransferTrainQueryCommand,
    ) -> Result<
        base::application::service::train_query::TransferTrainQueryDTO,
        Box<dyn base::application::ApplicationError>,
    > {
        Ok(base::application::service::train_query::TransferTrainQueryDTO { solutions: vec![] })
    }

    async fn query_train(
        &self,
        _command: base::application::commands::train_query::TrainScheduleQueryCommand,
    ) -> Result<
        base::application::service::train_query::TrainQueryResponseDTO,
        Box<dyn base::application::ApplicationError>,
    > {
        Ok(
            base::application::service::train_query::TrainQueryResponseDTO {
                origin_station: "北京南".into(),
                origin_departure_time: "2025-05-01 08:30:00".into(),
                departure_date: "2025-05-01".into(),
                terminal_station: "天津".into(),
                terminal_arrival_time: "2025-05-01 09:30:00".into(),
                route: vec![],
            },
        )
    }
}

#[actix_rt::test]
async fn query_direct_200_snapshot() {
    let app = test::init_service(
        App::new()
            .service(
                web::scope("/api")
                    .service(web::scope("/train").service(
                        web::scope("/schedule").configure(train::schedule::scoped_config),
                    )),
            )
            // inject mock service
            .app_data(web::Data::from(
                Arc::new(MockTrainQueryService) as Arc<dyn TrainQueryService>
            )),
    )
    .await;

    let payload = json!({
        "departureStation": "北京南",
        "arrivalStation": "天津",
        "departureDate": "2025-05-01"
    });

    let req = test::TestRequest::post()
        .uri("/api/train/schedule/query_direct")
        .cookie(actix_web::cookie::Cookie::new("session_id", "test_session"))
        .set_json(&payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let bytes = test::read_body(resp).await;
    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

    assert_eq!(value["code"], 200);
    assert_eq!(value["message"], "For Super Earth!");
    // data.soutions 应为 []
    let data = &value["data"];
    assert!(data.is_object());
    assert_eq!(data["solutions"], serde_json::Value::Array(vec![]));
}

#[actix_rt::test]
async fn query_direct_400_snapshot() {
    let app = test::init_service(
        App::new()
            .service(
                web::scope("/api")
                    .service(web::scope("/train").service(
                        web::scope("/schedule").configure(train::schedule::scoped_config),
                    )),
            )
            .app_data(web::Data::from(
                Arc::new(MockTrainQueryService) as Arc<dyn TrainQueryService>
            )),
    )
    .await;

    // invalid date format triggers GeneralError::BadRequest("Invalid date format")
    let payload = json!({
        "departureStation": "北京南",
        "arrivalStation": "天津",
        "departureDate": "2025/05/01" // invalid separator
    });

    let req = test::TestRequest::post()
        .uri("/api/train/schedule/query_direct")
        .cookie(actix_web::cookie::Cookie::new("session_id", "test_session"))
        .set_json(&payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let bytes = test::read_body(resp).await;
    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

    assert_eq!(value["code"], 400);
    assert_eq!(value["message"], "Invalid date format");
    assert!(value["data"].is_null());
}
