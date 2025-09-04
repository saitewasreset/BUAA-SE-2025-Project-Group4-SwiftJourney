use actix_web::{App, HttpServer, web};
use sea_orm::Database;
use shared::MicroService;
use shared::api::{MAX_BODY_LENGTH, read_file_env};
use shared::event::queue::EventService;
use shared::ports::impls::dish::HttpDishPortImpl;
use shared::ports::impls::geo::HttpGeoPortImpl;
use shared::ports::impls::order::HttpOrderPortImpl;
use shared::ports::impls::user::HttpUserPortImpl;
use std::env;
use std::env::VarError;
use std::sync::Arc;
use tokio;
use tracing_actix_web::TracingLogger;
use train_api::internal::scoped_config as internal_scoped_config;
use train_base::application::service::internal::TrainInternalService;
use train_base::application::service::train_data::TrainDataService;
use train_base::application::service::train_order::TrainOrderService;
use train_base::application::service::train_query::TrainQueryService;
use train_base::domain::service::train_schedule::TrainScheduleService;
use train_base::infrastructure::application::service::train_data::TrainDataServiceImpl;
use train_base::infrastructure::application::service::train_order::TrainOrderServiceImpl;
use train_base::infrastructure::application::service::train_query::TrainQueryServiceImpl;
use train_base::infrastructure::repository::route::RouteRepositoryImpl;
use train_base::infrastructure::repository::seat_availability::SeatAvailabilityRepositoryImpl;
use train_base::infrastructure::repository::train::TrainRepositoryImpl;
use train_base::infrastructure::repository::train_schedule::TrainScheduleRepositoryImpl;
use train_base::infrastructure::service::event::TrainEventServiceImpl;
use train_base::infrastructure::service::internal::TrainInternalServiceImpl;
use train_base::infrastructure::service::route::RouteServiceImpl;
use train_base::infrastructure::service::train_booking::TrainBookingServiceImpl;
use train_base::infrastructure::service::train_schedule::TrainScheduleServiceImpl;
use train_base::infrastructure::service::train_seat::TrainSeatServiceImpl;
use train_base::infrastructure::service::train_type::TrainTypeConfigurationServiceImpl;
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let tz_offset_hour_str = read_file_env("TZ_OFFSET_HOUR");

    let tz_offset_hour = match tz_offset_hour_str {
        Some(hour_str) => hour_str
            .parse::<i32>()
            .expect("cannot parse tz offset hour"),
        // UTC+8: China Standard Time
        None => 8,
    };

    let auto_schedule_days_str = read_file_env("AUTO_SCHEDULE_DAYS");

    let auto_schedule_days = match auto_schedule_days_str {
        Some(days_str) => days_str
            .parse::<i32>()
            .expect("cannot parse auto schedule days"),
        None => 14,
    };

    let debug_mode = match env::var("DEBUG") {
        Ok(_) => true,
        Err(VarError::NotPresent) => false,
        Err(VarError::NotUnicode(_)) => true,
    };

    let database_url = read_file_env("DATABASE_URL").expect("cannot get database url");
    let rabbitmq_url = read_file_env("RABBITMQ_URL").expect("cannot get rabbitmq url");

    let conn = Database::connect(&database_url)
        .await
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));

    // Repositories
    let train_repo = Arc::new(TrainRepositoryImpl::new(conn.clone()));
    let train_schedule_repo = Arc::new(TrainScheduleRepositoryImpl::new(conn.clone()));
    let route_repo = Arc::new(RouteRepositoryImpl::new(conn.clone()));
    let seat_availability_repo = Arc::new(SeatAvailabilityRepositoryImpl::new(conn.clone()));

    let geo_port = Arc::new(HttpGeoPortImpl::new(
        MicroService::Geo.internal_api_endpoint(),
    ));

    let order_port = Arc::new(HttpOrderPortImpl::new(
        MicroService::Order.internal_api_endpoint(),
    ));

    let user_port = Arc::new(HttpUserPortImpl::new(
        MicroService::User.internal_api_endpoint(),
    ));

    let dish_port = Arc::new(HttpDishPortImpl::new(
        MicroService::Dish.internal_api_endpoint(),
    ));

    // Domain Services
    let route_service = Arc::new(RouteServiceImpl::new(geo_port.clone(), route_repo.clone()));
    let train_type_config_service =
        Arc::new(TrainTypeConfigurationServiceImpl::new(train_repo.clone()));

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
        order_port.clone(),
        seat_availability_repo.clone(),
        train_type_config_service.clone(),
    ));

    let train_event_service_impl = TrainEventServiceImpl::new(
        &rabbitmq_url,
        conn.clone(),
        Arc::clone(&geo_port),
        Arc::clone(&user_port),
    )
    .await
    .expect("cannot connect to rabbitmq");

    train_event_service_impl
        .clone()
        .init_consumer()
        .await
        .expect("failed to init event consumer");

    let train_schedule_service_impl = Arc::new(TrainScheduleServiceImpl::new(
        Arc::clone(&route_service),
        Arc::clone(&train_repo),
        Arc::clone(&train_schedule_repo),
        Arc::clone(&route_repo),
        Arc::clone(&train_event_service_impl),
        tz_offset_hour,
    ));

    // Application Services
    let train_query_service_impl = Arc::new(TrainQueryServiceImpl::new(
        train_schedule_service_impl.clone(),
        geo_port.clone(),
        route_service.clone(),
        route_repo.clone(),
        train_repo.clone(),
        tz_offset_hour,
    ));

    let train_order_service_impl = Arc::new(TrainOrderServiceImpl::new(
        train_schedule_repo.clone(),
        train_booking_service.clone(),
        train_repo.clone(),
        route_repo.clone(),
        train_schedule_service_impl.clone(),
        train_type_config_service.clone(),
        geo_port.clone(),
        order_port.clone(),
        user_port.clone(),
    ));

    let train_internal_service_impl = Arc::new(TrainInternalServiceImpl::new(
        train_repo.clone(),
        train_type_config_service.clone(),
        train_schedule_repo.clone(),
        route_repo.clone(),
    ));

    let train_data_service_impl = Arc::new(TrainDataServiceImpl::new(
        debug_mode,
        Arc::clone(&geo_port),
        Arc::clone(&route_repo),
        Arc::clone(&dish_port),
        Arc::clone(&train_event_service_impl),
        conn.clone(),
    ));

    let train_query_service: web::Data<dyn TrainQueryService> =
        web::Data::from(train_query_service_impl as Arc<dyn TrainQueryService>);
    let train_order_service: web::Data<dyn TrainOrderService> =
        web::Data::from(train_order_service_impl as Arc<dyn TrainOrderService>);
    let train_internal_service: web::Data<dyn TrainInternalService> =
        web::Data::from(train_internal_service_impl as Arc<dyn TrainInternalService>);
    let train_data_service: web::Data<dyn TrainDataService> =
        web::Data::from(train_data_service_impl as Arc<dyn TrainDataService>);

    {
        let train_schedule_service_impl = Arc::clone(&train_schedule_service_impl);

        tokio::task::spawn(async move {
            train_schedule_service_impl
                .auto_plan_schedule_daemon(auto_schedule_days)
                .await;
        });
    }

    tokio::task::spawn(async move {
        HttpServer::new(move || {
            App::new()
                .app_data(train_internal_service.clone())
                .app_data(web::PayloadConfig::default().limit(MAX_BODY_LENGTH))
                .wrap(TracingLogger::default())
                .service(web::scope("/internal").configure(internal_scoped_config))
        })
        .bind(("0.0.0.0", 23333))
        .unwrap()
        .run()
        .await
        .unwrap();
    });

    HttpServer::new(move || {
        App::new()
            .app_data(train_query_service.clone())
            .app_data(train_order_service.clone())
            .app_data(train_data_service.clone())
            .app_data(web::PayloadConfig::default().limit(MAX_BODY_LENGTH))
            .wrap(TracingLogger::default())
            .service(web::scope("/api/train").configure(train_api::train::scoped_config))
            .service(web::scope("/api/data").configure(train_api::data::scoped_config))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
