use actix_web::{web, App, HttpServer};
use sea_orm::Database;
use std::env;
use std::sync::Arc;
use tokio;
use tracing_actix_web::TracingLogger;

use shared::api::{read_file_env, MAX_BODY_LENGTH};
use train_api::internal::scoped_config as internal_scoped_config;
use train_api::scoped_config;
use train_base::application::service::internal::TrainInternalService;
use train_base::application::service::train_order::TrainOrderService;
use train_base::application::service::train_query::TrainQueryService;
use train_base::infrastructure::application::service::internal::TrainInternalServiceImpl;
use train_base::infrastructure::application::service::train_order::TrainOrderServiceImpl;
use train_base::infrastructure::application::service::train_query::TrainQueryServiceImpl;
use train_base::infrastructure::repository::route::RouteRepositoryImpl;
use train_base::infrastructure::repository::seat_availability::SeatAvailabilityRepositoryImpl;
use train_base::infrastructure::repository::station::StationRepositoryImpl;
use train_base::infrastructure::repository::train::TrainRepositoryImpl;
use train_base::infrastructure::repository::train_schedule::TrainScheduleRepositoryImpl;
use train_base::infrastructure::service::route::RouteServiceImpl;
use train_base::infrastructure::service::station::StationServiceImpl;
use train_base::infrastructure::service::train_booking::TrainBookingServiceImpl;
use train_base::infrastructure::service::train_schedule::TrainScheduleServiceImpl;
use train_base::infrastructure::service::train_seat::TrainSeatServiceImpl;
use train_base::infrastructure::service::train_type::TrainTypeConfigurationServiceImpl;

// Mock dependencies for services that are not part of this microservice
// In a real implementation, these would be clients to other microservices' internal APIs.
use shared::domain::service::session::SessionManagerService;
use user_base::domain::service::session::SessionManagerService as UserSessionManagerService;
use user_base::infrastructure::service::session::SessionManagerServiceImpl as UserSessionManagerServiceImpl;
use user_base::domain::repository::session::SessionRepositoryConfig;
use user_base::infrastructure::repository::session::SessionRepositoryImpl as UserSessionRepositoryImpl;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let database_url = read_file_env("DATABASE_URL").expect("cannot get database url");
    let conn = Database::connect(&database_url)
        .await
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));

    // Repositories
    let train_repo = Arc::new(TrainRepositoryImpl::new(conn.clone()));
    let train_schedule_repo = Arc::new(TrainScheduleRepositoryImpl::new(conn.clone()));
    let route_repo = Arc::new(RouteRepositoryImpl::new(conn.clone()));
    let station_repo = Arc::new(StationRepositoryImpl::new(conn.clone()));
    let seat_availability_repo = Arc::new(SeatAvailabilityRepositoryImpl::new(conn.clone()));
    let order_repo_mock = Arc::new(train_base::infrastructure::repository::order::OrderRepositoryImpl::new(conn.clone()));
    let personal_info_repo_mock = Arc::new(user_base::infrastructure::repository::personal_info::PersonalInfoRepositoryImpl::new(conn.clone()));

    // Mock Services
    let session_manager_service_impl = Arc::new(UserSessionManagerServiceImpl::<UserSessionRepositoryImpl>::new(
        Arc::new(UserSessionRepositoryImpl::new(SessionRepositoryConfig::default())),
        Default::default(),
    ));
    let transaction_service_mock = Arc::new(order_base::infrastructure::service::transaction::TransactionServiceImpl::new(
        Arc::new(user_base::infrastructure::repository::user::UserRepositoryImpl::new(conn.clone())),
        Arc::new(order_base::infrastructure::repository::transaction::TransactionRepositoryImpl::new(conn.clone())),
        Arc::new(order_base::infrastructure::service::order::OrderServiceImpl::new(Arc::new(order_base::infrastructure::repository::order::OrderRepositoryImpl::new(conn.clone())), 8)),
        Arc::new(order_base::infrastructure::service::order_status::OrderStatusManagerServiceImpl::new(
            Arc::new(order_base::infrastructure::service::order_status_producer_service::OrderStatusProducerService::new("amqp://guest:guest@localhost:5672").await.unwrap()),
            Arc::new(order_base::infrastructure::repository::order::OrderRepositoryImpl::new(conn.clone()))
        ))
    ));

    // Domain Services
    let station_service = Arc::new(StationServiceImpl::new(station_repo.clone()));
    let route_service = Arc::new(RouteServiceImpl::new(station_service.clone(), route_repo.clone()));
    let train_type_config_service = Arc::new(TrainTypeConfigurationServiceImpl::new(train_repo.clone()));
    let train_schedule_service = Arc::new(TrainScheduleServiceImpl::new(
        route_service.clone(),
        train_repo.clone(),
        train_schedule_repo.clone(),
        route_repo.clone(),
        8, // Assuming UTC+8
    ));
    let train_seat_service = Arc::new(TrainSeatServiceImpl::new(
        seat_availability_repo.clone(),
        route_repo.clone(),
        train_type_config_service.clone(),
        train_schedule_repo.clone(),
    ));
    let train_booking_service = Arc::new(TrainBookingServiceImpl::new(
        train_schedule_repo.clone(),
        train_seat_service.clone(),
        train_repo.clone(),
        order_repo_mock.clone(),
        seat_availability_repo.clone(),
        train_type_config_service.clone(),
    ));

    // Application Services
    let train_query_service_impl = Arc::new(TrainQueryServiceImpl::new(
        train_schedule_service.clone(),
        station_service.clone(),
        route_service.clone(),
        session_manager_service_impl.clone(),
        route_repo.clone(),
        train_repo.clone(),
        station_repo.clone(),
        8, // Assuming UTC+8
    ));

    let train_order_service_impl = Arc::new(TrainOrderServiceImpl::new(
        train_schedule_repo.clone(),
        train_booking_service.clone(),
        train_repo.clone(),
        route_repo.clone(),
        station_repo.clone(),
        order_repo_mock.clone(),
        transaction_service_mock.clone(),
        session_manager_service_impl.clone(),
        personal_info_repo_mock.clone(),
        train_schedule_service.clone(),
        train_type_config_service.clone(),
    ));

    let train_internal_service_impl = Arc::new(TrainInternalServiceImpl::new(
        train_repo.clone(),
        train_schedule_repo.clone(),
    ));

    let train_query_service: web::Data<dyn TrainQueryService> = web::Data::from(train_query_service_impl);
    let train_order_service: web::Data<dyn TrainOrderService> = web::Data::from(train_order_service_impl);
    let train_internal_service: web::Data<dyn TrainInternalService> = web::Data::from(train_internal_service_impl);

    tokio::task::spawn(async move {
        HttpServer::new(move || {
            App::new()
                .app_data(train_internal_service.clone())
                .app_data(web::PayloadConfig::default().limit(MAX_BODY_LENGTH))
                .wrap(TracingLogger::default())
                .service(web::scope("/internal").configure(internal_scoped_config))
        })
        .bind(("0.0.0.0", 23334))
        .unwrap()
        .run()
        .await
        .unwrap();
    });

    HttpServer::new(move || {
        App::new()
            .app_data(train_query_service.clone())
            .app_data(train_order_service.clone())
            .app_data(web::PayloadConfig::default().limit(MAX_BODY_LENGTH))
            .wrap(TracingLogger::default())
            .service(web::scope("/api/train").configure(scoped_config))
    })
    .bind(("0.0.0.0", 8081))?
    .run()
    .await
}
