/*
 * Even if we lose our way, we keep on moving.
 */
use actix_web::{App, HttpServer, web};
use migration::MigratorTrait;
use sea_orm::Database;
use shared::api::{MAX_BODY_LENGTH, read_file_env};
use std::sync::Arc;
use tracing_actix_web::TracingLogger;
use hotel_base::application::service::internal::UserInternalService;
use hotel_base::application::service::personal_info::PersonalInfoService;
use hotel_base::application::service::user_manager::UserManagerService;
use hotel_base::application::service::user_profile::UserProfileService;
use hotel_base::domain::model::session_config::SessionConfig;
use hotel_base::domain::repository::session::SessionRepositoryConfig;
use hotel_base::domain::repository::user::UserRepository;
use hotel_base::domain::service::session::SessionManagerService;
use hotel_base::domain::service::user::UserService;
use hotel_base::infrastructure::application::service::personal_info::PersonalInfoServiceImpl;
use hotel_base::infrastructure::application::service::user_manager::UserManagerServiceImpl;
use hotel_base::infrastructure::application::service::user_profile::UserProfileServiceImpl;
use hotel_base::infrastructure::repository::personal_info::PersonalInfoRepositoryImpl;
use hotel_base::infrastructure::repository::session::SessionRepositoryImpl;
use hotel_base::infrastructure::repository::user::UserRepositoryImpl;
use hotel_base::infrastructure::service::internal::UserInternalServiceImpl;
use hotel_base::infrastructure::service::password::Argon2PasswordServiceImpl;
use hotel_base::infrastructure::service::session::SessionManagerServiceImpl;
use hotel_base::infrastructure::service::user::UserServiceImpl;

#[actix_web::main]

async fn main() -> std::io::Result<()> {
    // env_logger::init_from_env(Env::default().default_filter_or("info"));
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt::init();

    let database_url = read_file_env("DATABASE_URL").expect("cannot get database url");

    let conn = Database::connect(&database_url)
        .await
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));

    migration::Migrator::up(&conn, None)
        .await
        .unwrap_or_else(|_| panic!("Error applying migration to {}", database_url));

    let session_manager_service_impl =
        Arc::new(SessionManagerServiceImpl::<SessionRepositoryImpl>::new(
            Arc::new(SessionRepositoryImpl::new(
                SessionRepositoryConfig::default(),
            )),
            SessionConfig::default(),
        ));

    let hotel_data_service_impl = Arc::new(HotelDataServiceImpl::new(
        app_config.debug,
        data_base_path,
        Arc::clone(&city_repository_impl),
        Arc::clone(&station_repository_impl),
        Arc::clone(&s3_object_storage_service_impl),
    ));

    let hotel_rating_service_impl = Arc::new(HotelRatingServiceImpl::new(
        Arc::clone(&hotel_repository_impl),
        Arc::clone(&hotel_rating_repository_impl),
        Arc::clone(&order_repository_impl),
    ));

    let hotel_booking_service_impl = Arc::new(HotelBookingServiceImpl::new(
        Arc::clone(&hotel_repository_impl),
        Arc::clone(&order_repository_impl),
        Arc::clone(&occupied_room_repository_impl),
    ));

    let hotel_query_service_impl = Arc::new(HotelQueryServiceImpl::new(
        Arc::clone(&hotel_repository_impl),
        Arc::clone(&hotel_rating_repository_impl),
        Arc::clone(&city_repository_impl),
        Arc::clone(&station_repository_impl),
        Arc::clone(&occupied_room_repository_impl),
    ));

    let hotel_service_impl = Arc::new(HotelServiceImpl::new(
        Arc::clone(&hotel_rating_service_impl),
        Arc::clone(&hotel_query_service_impl),
        Arc::clone(&hotel_booking_service_impl),
        Arc::clone(&hotel_repository_impl),
        Arc::clone(&user_repository_impl),
        Arc::clone(&session_manager_service_impl),
    ));

    let train_seat_service_impl = Arc::new(TrainSeatServiceImpl::new(
        Arc::clone(&seat_availability_repository_impl),
        Arc::clone(&route_repository_impl),
        Arc::clone(&train_type_service_impl),
        Arc::clone(&train_schedule_repository_impl),
    ));

    let train_type_configuration_service_impl = Arc::new(TrainTypeConfigurationServiceImpl::new(
        Arc::clone(&train_repository_impl),
    ));

    let train_booking_service_impl = Arc::new(TrainBookingServiceImpl::new(
        Arc::clone(&train_schedule_repository_impl),
        Arc::clone(&train_seat_service_impl),
        Arc::clone(&train_repository_impl),
        Arc::clone(&order_repository_impl),
        Arc::clone(&seat_availability_repository_impl),
        Arc::clone(&train_type_configuration_service_impl),
    ));

    tokio::task::spawn(async move {
        HttpServer::new(move || {
            App::new()
                .app_data(user_internal_service.clone())
                .app_data(web::PayloadConfig::default().limit(MAX_BODY_LENGTH))
                .wrap(TracingLogger::default())
                .service(web::scope("/internal").configure(user_api::internal::scoped_config))
        })
        .bind(("0.0.0.0", 23333))
        .unwrap()
        .run()
        .await
        .unwrap();
    });

    HttpServer::new(move || {
        App::new()
            .app_data(session_manager_service.clone())
            .app_data(user_repository.clone())
            .app_data(user_service.clone())
            .app_data(user_manager_service.clone())
            .app_data(user_profile_service.clone())
            .app_data(personal_info_service.clone())
            .app_data(conn.clone())
            .app_data(web::PayloadConfig::default().limit(MAX_BODY_LENGTH))
            .wrap(TracingLogger::default())
            .service(
                web::scope("/api")
                    .service(web::scope("/user").configure(user_api::user::scoped_config)),
            )
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await?;

    Ok(())
}
