/*
 * Super Earth.
 * Our home.
 * Prosperity.
 * Liberty.
 * (Hi there)
 * (Hey)
 * Democracy.
 * Our way of life.
 * (Hello)
 * But freedom doesn't come free.
 * No...
 * sweet Liberty...
 * NOOOO!
 * (laughs) Look familiar?
 * Scenes like these are happening all over the galaxy, right now!
 * You could be next.
 * That is, unless you make the most important decision of your life.
 * Prove to yourself that you have the strength and the courage to be free.
 * Join...the Helldivers.
 *  Become part of an elite peacekeeping force!
 * See exotic new lifeforms.
 * And spread Managed Democracy throughout the galaxy.
 * Become a HERO.
 * Become a LEGEND.
 * Become a Helldiver!
 */
use actix_web::{App, HttpServer, web};
use order_base::application::service::internal::OrderInternalService;
use order_base::application::service::transaction::TransactionApplicationService;
use order_base::infrastructure::application::service::transaction::TransactionApplicationServiceImpl;
use order_base::infrastructure::repository::order::OrderRepositoryImpl;
use order_base::infrastructure::repository::transaction::TransactionRepositoryImpl;
use order_base::infrastructure::service::event::OrderEventServiceImpl;
use order_base::infrastructure::service::internal::OrderInternalServiceImpl;
use order_base::infrastructure::service::order::OrderServiceImpl;
use order_base::infrastructure::service::order_status::OrderStatusManagerServiceImpl;
use order_base::infrastructure::service::transaction::TransactionServiceImpl;
use order_migration::MigratorTrait;
use sea_orm::Database;
use shared::MicroService;
use shared::api::{MAX_BODY_LENGTH, read_file_env};
use shared::event::queue::EventService;
use shared::messaging::order_status_producer_service::OrderStatusProducerService;
use shared::ports::impls::dish::HttpDishPortImpl;
use shared::ports::impls::geo::HttpGeoPortImpl;
use shared::ports::impls::hotel::HttpHotelPortImpl;
use shared::ports::impls::train::HttpTrainPortImpl;
use shared::ports::impls::user::HttpUserPortImpl;
use std::env;
use std::env::VarError;
use std::sync::Arc;
use tracing_actix_web::TracingLogger;

#[actix_web::main]

async fn main() -> std::io::Result<()> {
    // env_logger::init_from_env(Env::default().default_filter_or("info"));
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt::init();

    let database_url = read_file_env("DATABASE_URL").expect("cannot get database url");
    let rabbitmq_url = read_file_env("RABBITMQ_URL").expect("cannot get rabbitmq url");
    let tz_offset_hour_str = read_file_env("TZ_OFFSET_HOUR");

    let tz_offset_hour = match tz_offset_hour_str {
        Some(hour_str) => hour_str
            .parse::<i32>()
            .expect("cannot parse tz offset hour"),
        // UTC+8: China Standard Time
        None => 8,
    };

    let debug_mode = match env::var("DEBUG") {
        Ok(_) => true,
        Err(VarError::NotPresent) => false,
        Err(VarError::NotUnicode(_)) => true,
    };

    let conn = Database::connect(&database_url)
        .await
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));

    order_migration::Migrator::up(&conn, None)
        .await
        .unwrap_or_else(|_| panic!("Error applying migration to {}", database_url));

    let geo_port_impl = Arc::new(HttpGeoPortImpl::new(
        MicroService::Geo.internal_api_endpoint(),
    ));
    let user_port_impl = Arc::new(HttpUserPortImpl::new(
        MicroService::User.internal_api_endpoint(),
    ));
    let train_port_impl = Arc::new(HttpTrainPortImpl::new(
        MicroService::Train.internal_api_endpoint(),
    ));
    let hotel_port_impl = Arc::new(HttpHotelPortImpl::new(
        MicroService::Hotel.internal_api_endpoint(),
    ));
    let dish_port_impl = Arc::new(HttpDishPortImpl::new(
        MicroService::Dish.internal_api_endpoint(),
    ));

    let order_repository_impl = Arc::new(OrderRepositoryImpl::new(conn.clone()));
    let order_service_impl = Arc::new(OrderServiceImpl::new(
        Arc::clone(&order_repository_impl),
        tz_offset_hour,
    ));

    let order_status_producer_service = Arc::new(
        OrderStatusProducerService::new(&rabbitmq_url)
            .await
            .expect("Failed to start order status producer service"),
    );

    let order_status_manager_service_impl = Arc::new(OrderStatusManagerServiceImpl::new(
        Arc::clone(&order_status_producer_service),
        Arc::clone(&order_repository_impl),
    ));

    let transaction_repository_impl = Arc::new(TransactionRepositoryImpl::new(conn.clone()));

    let transaction_service_impl = Arc::new(TransactionServiceImpl::new(
        Arc::clone(&user_port_impl),
        Arc::clone(&transaction_repository_impl),
        Arc::clone(&order_service_impl),
        Arc::clone(&order_status_manager_service_impl),
    ));

    let transaction_application_service_impl = Arc::new(TransactionApplicationServiceImpl::new(
        debug_mode,
        Arc::clone(&transaction_service_impl),
        Arc::clone(&transaction_repository_impl),
        Arc::clone(&user_port_impl),
    ));

    let order_internal_service_impl = Arc::new(OrderInternalServiceImpl::new(
        Arc::clone(&transaction_service_impl),
        Arc::clone(&order_service_impl),
        Arc::clone(&order_repository_impl),
    ));

    let transaction_application_service: web::Data<dyn TransactionApplicationService> =
        web::Data::from(
            transaction_application_service_impl as Arc<dyn TransactionApplicationService>,
        );

    let order_internal_service: web::Data<dyn OrderInternalService> =
        web::Data::from(order_internal_service_impl as Arc<dyn OrderInternalService>);

    let order_event_service_impl = OrderEventServiceImpl::new(
        &rabbitmq_url,
        conn.clone(),
        Arc::clone(&geo_port_impl),
        Arc::clone(&user_port_impl),
        Arc::clone(&train_port_impl),
        Arc::clone(&hotel_port_impl),
        Arc::clone(&dish_port_impl),
    )
    .await
    .expect("Cannot connect to rabbitmq");

    order_event_service_impl
        .clone()
        .init_consumer()
        .await
        .expect("Failed to init consumer");

    tokio::task::spawn(async move {
        HttpServer::new(move || {
            App::new()
                .app_data(order_internal_service.clone())
                .app_data(web::PayloadConfig::default().limit(MAX_BODY_LENGTH))
                .wrap(TracingLogger::default())
                .service(web::scope("/internal").configure(order_api::internal::scoped_config))
        })
        .bind(("0.0.0.0", 23333))
        .unwrap()
        .run()
        .await
        .unwrap();
    });

    HttpServer::new(move || {
        App::new()
            .app_data(transaction_application_service.clone())
            .app_data(web::PayloadConfig::default().limit(MAX_BODY_LENGTH))
            .wrap(TracingLogger::default())
            .service(
                web::scope("/api")
                    .service(web::scope("/payment").configure(order_api::payment::scoped_config))
                    .service(web::scope("/order").configure(order_api::order::scoped_config)),
            )
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await?;

    Ok(())
}
