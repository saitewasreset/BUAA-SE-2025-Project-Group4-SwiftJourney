/*
 * Even if we lose our way, we keep on moving.
 */
use actix_web::{App, HttpServer, web};
use hotel_base::application::service::hotel::HotelService;
use hotel_base::application::service::hotel_data::HotelDataService;
use hotel_base::application::service::hotel_order::HotelOrderService;
use hotel_base::application::service::internal::HotelInternalService;
use hotel_base::infrastructure::application::service::hotel::HotelServiceImpl;
use hotel_base::infrastructure::application::service::hotel_data::HotelDataServiceImpl;
use hotel_base::infrastructure::application::service::hotel_order::HotelOrderServiceImpl;
use hotel_base::infrastructure::application::service::internal::HotelInternalServiceImpl;
use hotel_base::infrastructure::messaging::order_status::HotelOrderStatusConsumer;
use hotel_base::infrastructure::repository::hotel::HotelRepositoryImpl;
use hotel_base::infrastructure::repository::hotel_rating::HotelRatingRepositoryImpl;
use hotel_base::infrastructure::repository::occupied_room::OccupiedRoomRepositoryImpl;
use hotel_base::infrastructure::service::event::HotelEventServiceImpl;
use hotel_base::infrastructure::service::hotel_booking::HotelBookingServiceImpl;
use hotel_base::infrastructure::service::hotel_query::HotelQueryServiceImpl;
use hotel_base::infrastructure::service::hotel_rating::HotelRatingServiceImpl;
use hotel_migration::MigratorTrait;
use sea_orm::{Database, DatabaseConnection};
use shared::MicroService;
use shared::api::{AppConfig, MAX_BODY_LENGTH, read_file_env};
use shared::event::queue::EventService;
use shared::messaging::order_status::RabbitMQOrderStatusConsumer;
use shared::messaging::order_status_consumer_service::OrderStatusConsumerService;
use shared::ports::impls::geo::HttpGeoPortImpl;
use shared::ports::impls::object_storage::HttpObjectStoragePortImpl;
use shared::ports::impls::order::HttpOrderPortImpl;
use shared::ports::impls::user::HttpUserPortImpl;
use std::env;
use std::env::VarError;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use tracing_actix_web::TracingLogger;

#[actix_web::main]

async fn main() -> std::io::Result<()> {
    // env_logger::init_from_env(Env::default().default_filter_or("info"));
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt::init();

    let server_name = read_file_env("SERVER_NAME").expect("cannot get server name");

    let database_url = read_file_env("DATABASE_URL").expect("cannot get database url");
    let rabbitmq_url = read_file_env("RABBITMQ_URL").expect("cannot get rabbitmq url");

    let data_base_path = read_file_env("DATA_PATH").expect("cannot get data path");

    let data_base_path = PathBuf::from_str(&data_base_path).expect("cannot parse data path");

    let debug_mode = match env::var("DEBUG") {
        Ok(_) => true,
        Err(VarError::NotPresent) => false,
        Err(VarError::NotUnicode(_)) => true,
    };

    let conn = Database::connect(&database_url)
        .await
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));

    hotel_migration::Migrator::up(&conn, None)
        .await
        .unwrap_or_else(|_| panic!("Error applying migration to {}", database_url));

    let app_config = AppConfig {
        debug: debug_mode,
        server_name,
    };

    let order_port_impl = Arc::new(HttpOrderPortImpl::new(
        MicroService::Order.internal_api_endpoint(),
    ));

    let user_port_impl = Arc::new(HttpUserPortImpl::new(
        MicroService::User.internal_api_endpoint(),
    ));

    let geo_port_impl = Arc::new(HttpGeoPortImpl::new(
        MicroService::Geo.internal_api_endpoint(),
    ));

    let event_service_impl = HotelEventServiceImpl::new(
        &rabbitmq_url,
        conn.clone(),
        Arc::clone(&geo_port_impl),
        Arc::clone(&user_port_impl),
    )
    .await
    .expect("failed to create event service");

    event_service_impl
        .clone()
        .init_consumer()
        .await
        .expect("failed to init consumer");

    let object_storage_impl = Arc::new(HttpObjectStoragePortImpl::new(
        MicroService::ObjectStorage.internal_api_endpoint(),
    ));

    let hotel_repository_impl = Arc::new(HotelRepositoryImpl::new(
        conn.clone(),
        Arc::clone(&geo_port_impl),
    ));
    let hotel_rating_repository_impl = Arc::new(HotelRatingRepositoryImpl::new(conn.clone()));
    let occupied_room_repository_impl = Arc::new(OccupiedRoomRepositoryImpl::new(conn.clone()));

    let hotel_data_service_impl = Arc::new(HotelDataServiceImpl::new(
        app_config.debug,
        data_base_path,
        Arc::clone(&geo_port_impl),
        Arc::clone(&object_storage_impl),
        Arc::clone(&event_service_impl),
    ));

    let hotel_rating_service_impl = Arc::new(HotelRatingServiceImpl::new(
        Arc::clone(&hotel_repository_impl),
        Arc::clone(&hotel_rating_repository_impl),
        Arc::clone(&order_port_impl),
    ));

    let hotel_booking_service_impl = Arc::new(HotelBookingServiceImpl::new(
        Arc::clone(&hotel_repository_impl),
        Arc::clone(&order_port_impl),
        Arc::clone(&occupied_room_repository_impl),
    ));

    let hotel_query_service_impl = Arc::new(HotelQueryServiceImpl::new(
        Arc::clone(&hotel_repository_impl),
        Arc::clone(&hotel_rating_repository_impl),
        Arc::clone(&geo_port_impl),
        Arc::clone(&occupied_room_repository_impl),
    ));

    let hotel_service_impl = Arc::new(HotelServiceImpl::new(
        Arc::clone(&hotel_rating_service_impl),
        Arc::clone(&hotel_query_service_impl),
        Arc::clone(&hotel_booking_service_impl),
        Arc::clone(&hotel_repository_impl),
        Arc::clone(&user_port_impl),
        Arc::clone(&event_service_impl),
    ));

    let hotel_order_service_impl = Arc::new(HotelOrderServiceImpl::new(
        Arc::clone(&hotel_repository_impl),
        Arc::clone(&hotel_booking_service_impl),
        Arc::clone(&order_port_impl),
        Arc::clone(&user_port_impl),
        Arc::clone(&event_service_impl),
    ));

    let hotel_internal_service_impl = Arc::new(HotelInternalServiceImpl::new(Arc::clone(
        &hotel_repository_impl,
    )));

    let hotel_data_service: web::Data<dyn HotelDataService> =
        web::Data::from(hotel_data_service_impl as Arc<dyn HotelDataService>);

    let hotel_service: web::Data<dyn HotelService> =
        web::Data::from(hotel_service_impl as Arc<dyn HotelService>);

    let hotel_order_service: web::Data<dyn HotelOrderService> =
        web::Data::from(hotel_order_service_impl as Arc<dyn HotelOrderService>);

    let hotel_internal_service: web::Data<dyn HotelInternalService> =
        web::Data::from(hotel_internal_service_impl as Arc<dyn HotelInternalService>);

    let db_data: web::Data<DatabaseConnection> = web::Data::new(conn.clone());

    let hotel_order_status_consumer = Box::new(HotelOrderStatusConsumer::new(
        Arc::clone(&hotel_booking_service_impl),
        Arc::clone(&order_port_impl),
    ));

    let order_status_consumer =
        vec![hotel_order_status_consumer as Box<dyn RabbitMQOrderStatusConsumer>];

    let _ = OrderStatusConsumerService::start(&rabbitmq_url, order_status_consumer)
        .await
        .expect("Failed to start order status consumer service");

    tokio::task::spawn(async move {
        HttpServer::new(move || {
            App::new()
                .app_data(hotel_internal_service.clone())
                .app_data(web::PayloadConfig::default().limit(MAX_BODY_LENGTH))
                .wrap(TracingLogger::default())
                .service(web::scope("/internal").configure(hotel_api::internal::scoped_config))
        })
        .bind(("0.0.0.0", 23333))
        .unwrap()
        .run()
        .await
        .unwrap();
    });

    HttpServer::new(move || {
        App::new()
            .app_data(hotel_data_service.clone())
            .app_data(hotel_service.clone())
            .app_data(hotel_order_service.clone())
            .app_data(db_data.clone())
            .app_data(web::PayloadConfig::default().limit(MAX_BODY_LENGTH))
            .wrap(TracingLogger::default())
            .service(
                web::scope("/api")
                    .service(web::scope("/data").configure(hotel_api::data::scoped_config))
                    .service(web::scope("/hotel").configure(hotel_api::hotel::scoped_config)),
            )
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await?;

    Ok(())
}
