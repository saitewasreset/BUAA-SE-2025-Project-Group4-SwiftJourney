use actix_web::{App, HttpServer, web};
use dish_base::application::service::dish_query::DishQueryService;
use dish_base::application::service::train_dish::TrainDishApplicationService;
use dish_base::infrastructure::application::service::dish_query::DishQueryServiceImpl;
use dish_base::infrastructure::repository::dish::DishRepositoryImpl;
use dish_base::infrastructure::repository::takeaway::TakeawayShopRepositoryImpl;
use dish_base::infrastructure::service::dish_booking::DishBookingServiceImpl;
use dish_base::infrastructure::service::takeaway_booking::TakeawayBookingServiceImpl;
use migration::MigratorTrait;
use sea_orm::Database;
use shared::MAX_CONCURRENT_WEBSOCKET_SESSION_PER_USER;
use shared::MicroService;
use shared::api::{AppConfig, MAX_BODY_LENGTH, SuperClient};
use shared::ports::geo::GeoPort;
use shared::ports::impls::geo::HttpGeoPortImpl;
use shared::ports::impls::order::HttpOrderPortImpl;
use shared::ports::impls::train::HttpTrainPortImpl;
use shared::ports::impls::user::HttpUserPortImpl;
use std::env::VarError;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::{env, fs};
use tracing::{error, instrument, warn};
use tracing_actix_web::TracingLogger;

#[actix_web::main]

async fn main() -> std::io::Result<()> {
    // env_logger::init_from_env(Env::default().default_filter_or("info"));
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt::init();

    let server_name = read_file_env("SERVER_NAME").expect("cannot get server name");

    let database_url = read_file_env("DATABASE_URL").expect("cannot get database url");
    read_file_env("RABBITMQ_URL").expect("cannot get rabbitmq url");
    let tz_offset_hour_str = read_file_env("TZ_OFFSET_HOUR");

    let auto_schedule_days_str = read_file_env("AUTO_SCHEDULE_DAYS");

    let mini_io_endpoint = read_file_env("MINIO_ENDPOINT").expect("cannot get minio endpoint");
    read_file_env("MINIO_ACCESS_KEY").expect("cannot get minio access key");
    read_file_env("MINIO_SECRET_KEY").expect("cannot get minio secret key");

    let data_base_path = read_file_env("DATA_PATH").expect("cannot get data path");

    PathBuf::from_str(&data_base_path).expect("cannot parse data path");

    let tz_offset_hour = match tz_offset_hour_str {
        Some(hour_str) => hour_str
            .parse::<i32>()
            .expect("cannot parse tz offset hour"),
        // UTC+8: China Standard Time
        None => 8,
    };

    match auto_schedule_days_str {
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

    let conn = Database::connect(&database_url)
        .await
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));

    migration::Migrator::up(&conn, None)
        .await
        .unwrap_or_else(|_| panic!("Error applying migration to {}", database_url));

    let dish_repository_impl = Arc::new(DishRepositoryImpl::new(conn.clone()));
    let takeaway_repository_impl = Arc::new(TakeawayShopRepositoryImpl::new(conn.clone()));

    let geo_port_impl = Arc::new(HttpGeoPortImpl::new(
        MicroService::Geo.internal_api_endpoint(),
    ));
    let order_port_impl = Arc::new(HttpOrderPortImpl::new(
        MicroService::Order.internal_api_endpoint(),
    ));
    let train_port_impl = Arc::new(HttpTrainPortImpl::new(
        MicroService::Train.internal_api_endpoint(),
    ));
    let user_port_impl = Arc::new(HttpUserPortImpl::new(
        MicroService::User.internal_api_endpoint(),
    ));

    let dish_booking_service_impl =
        Arc::new(DishBookingServiceImpl::new(Arc::clone(&order_port_impl)));

    let takeaway_booking_service_impl = Arc::new(TakeawayBookingServiceImpl::new(Arc::clone(
        &order_port_impl,
    )));

    let dish_query_service_impl = Arc::new(DishQueryServiceImpl::new(
        Arc::clone(&dish_repository_impl),
        Arc::clone(&takeaway_repository_impl),
        Arc::clone(&geo_port_impl),
        Arc::clone(&order_port_impl),
        Arc::clone(&train_port_impl),
        Arc::clone(&user_port_impl),
    ));

    let train_dish_application_service_impl = Arc::new(
        dish_base::infrastructure::application::service::internal::DishInternalServiceImpl::new(
            Arc::clone(&dish_repository_impl),
            Arc::clone(&takeaway_repository_impl),
            Arc::clone(&train_port_impl),
            Arc::clone(&user_port_impl),
        ),
    );

    let dish_query_service: web::Data<dyn DishQueryService> =
        web::Data::from(dish_query_service_impl as Arc<dyn DishQueryService>);

    let train_dish_application_service: web::Data<dyn TrainDishApplicationService> =
        web::Data::from(
            train_dish_application_service_impl as Arc<dyn TrainDishApplicationService>,
        );

    tokio::task::spawn(async move {
        HttpServer::new(move || {
            App::new()
                .app_data(dish_internal_service.clone())
                .app_data(web::PayloadConfig::default().limit(MAX_BODY_LENGTH))
                .wrap(TracingLogger::default())
                .service(web::scope("/internal").configure(dish_api::internal::scoped_config))
        })
        .bind(("0.0.0.0", 23333))
        .unwrap()
        .run()
        .await
        .unwrap();
    });

    HttpServer::new(move || {
        App::new()
            .app_data(dish_query_service.clone())
            .app_data(train_dish_application_service.clone())
            .app_data(conn.clone())
            // Thinking 1.2.1D - 8: `App::new().app_data(...).app_data(...)`是什么设计模式的体现？
            // Good! Next, build your API endpoint in `api::train::schedule`
            .app_data(web::PayloadConfig::default().limit(MAX_BODY_LENGTH))
            .wrap(TracingLogger::default())
            .service(
                web::scope("/api")
                    .service(web::scope("/dish").configure(dish_api::dish::scoped_config)),
            )
            .service(actix_files::Files::new("/", "/static").index_file("index.html"))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await?;

    Ok(())
}

#[instrument]
fn read_file_env(target_env: &str) -> Option<String> {
    let mut result: Option<String> = None;
    if let Ok(file_path) = env::var(format!("{}_FILE", target_env)) {
        match fs::read_to_string(&file_path) {
            Ok(val) => result = Some(val.trim().to_string()),
            Err(e) => {
                warn!("cannot read env file {}: {}", file_path, e)
            }
        }
    }

    if result.is_none()
        && let Ok(env_str) = env::var(target_env)
    {
        result = Some(env_str);
    }

    result
}
