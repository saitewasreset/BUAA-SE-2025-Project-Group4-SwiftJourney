// Step 1: Read the sentences below.
// Thinking 1.2.1D - 6: 你认为下面的句子来自哪个游戏？其用意是什么？
// 用意是夹带私货，宣传“管理式民主”
// Thinking 1.2.1D - 7: 什么是“管理式民主”（Managed Democracy）？你认为它是真实的民主吗？
// “管理式民主”是一个讽刺的说法，指的是一种表面上看似民主的制度，但实际上是由少数人或特定利益集团控制的。它并不是真正的民主，因为它缺乏真正的选举自由和公民参与。
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
use api::{AppConfig, MAX_BODY_LENGTH, resource};
use base::MAX_CONCURRENT_WEBSOCKET_SESSION_PER_USER;
use base::application::service::dish_query::DishQueryService;
use base::application::service::geo::GeoApplicationService;
use base::application::service::hotel::HotelService;
use base::application::service::hotel_data::HotelDataService;
use base::application::service::hotel_order::HotelOrderService;
use base::application::service::message::MessageApplicationService;
use base::application::service::personal_info::PersonalInfoService;
use base::application::service::train_data::TrainDataService;
use base::application::service::train_dish::TrainDishApplicationService;
use base::application::service::train_order::TrainOrderService;
use base::application::service::train_query::TrainQueryService;
use base::application::service::transaction::TransactionApplicationService;
use base::application::service::user_manager::UserManagerService;
use base::application::service::user_profile::UserProfileService;
use base::domain::model::session_config::SessionConfig;
use base::domain::repository::session::SessionRepositoryConfig;
use base::domain::repository::user::UserRepository;
use base::domain::service::message::MessageListenerService;
use base::domain::service::object_storage::ObjectStorageService;
use base::domain::service::order_status::OrderStatusManagerService;
use base::domain::service::route::RouteService;
use base::domain::service::session::SessionManagerService;
use base::domain::service::train_schedule::TrainScheduleService;
use base::domain::service::train_type::TrainTypeConfigurationService;
use base::domain::service::user::UserService;
use base::infrastructure::application::service::dish_query::DishQueryServiceImpl;
use base::infrastructure::application::service::geo::GeoApplicationServiceImpl;
use base::infrastructure::application::service::hotel::HotelServiceImpl;
use base::infrastructure::application::service::hotel_data::HotelDataServiceImpl;
use base::infrastructure::application::service::hotel_order::HotelOrderServiceImpl;
use base::infrastructure::application::service::message::MessageApplicationServiceImpl;
use base::infrastructure::application::service::personal_info::PersonalInfoServiceImpl;
use base::infrastructure::application::service::train_data::TrainDataServiceImpl;
use base::infrastructure::application::service::train_order::TrainOrderServiceImpl;
use base::infrastructure::application::service::train_query::TrainQueryServiceImpl;
use base::infrastructure::application::service::transaction::TransactionApplicationServiceImpl;
use base::infrastructure::application::service::user_manager::UserManagerServiceImpl;
use base::infrastructure::application::service::user_profile::UserProfileServiceImpl;
use base::infrastructure::messaging::consumer::order_status::{
    DishOrderStatusConsumer, HotelOrderStatusConsumer, RabbitMQOrderStatusConsumer,
    TakeawayOrderStatusConsumer, TrainOrderStatusConsumer,
};
use base::infrastructure::repository::city::CityRepositoryImpl;
use base::infrastructure::repository::dish::DishRepositoryImpl;
use base::infrastructure::repository::hotel::HotelRepositoryImpl;
use base::infrastructure::repository::hotel_rating::HotelRatingRepositoryImpl;
use base::infrastructure::repository::notify::NotifyRepositoryImpl;
use base::infrastructure::repository::occupied_room::OccupiedRoomRepositoryImpl;
use base::infrastructure::repository::order::OrderRepositoryImpl;
use base::infrastructure::repository::personal_info::PersonalInfoRepositoryImpl;
use base::infrastructure::repository::route::RouteRepositoryImpl;
use base::infrastructure::repository::seat_availability::SeatAvailabilityRepositoryImpl;
use base::infrastructure::repository::session::SessionRepositoryImpl;
use base::infrastructure::repository::station::StationRepositoryImpl;
use base::infrastructure::repository::takeaway::TakeawayShopRepositoryImpl;
use base::infrastructure::repository::train::TrainRepositoryImpl;
use base::infrastructure::repository::train_schedule::TrainScheduleRepositoryImpl;
use base::infrastructure::repository::transaction::TransactionRepositoryImpl;
use base::infrastructure::repository::user::UserRepositoryImpl;
use base::infrastructure::service::dish_booking::DishBookingServiceImpl;
use base::infrastructure::service::geo::GeoServiceImpl;
use base::infrastructure::service::hotel_booking::HotelBookingServiceImpl;
use base::infrastructure::service::hotel_query::HotelQueryServiceImpl;
use base::infrastructure::service::hotel_rating::HotelRatingServiceImpl;
use base::infrastructure::service::message::{MessageListenerServiceImpl, MessageServiceImpl};
use base::infrastructure::service::object_storage::S3ObjectStorageServiceImpl;
use base::infrastructure::service::order::OrderServiceImpl;
use base::infrastructure::service::order_status::OrderStatusManagerServiceImpl;
use base::infrastructure::service::order_status_consumer_service::OrderStatusConsumerService;
use base::infrastructure::service::order_status_producer_service::OrderStatusProducerService;
use base::infrastructure::service::password::Argon2PasswordServiceImpl;
use base::infrastructure::service::route::RouteServiceImpl;
use base::infrastructure::service::session::SessionManagerServiceImpl;
use base::infrastructure::service::station::StationServiceImpl;
use base::infrastructure::service::takeaway_booking::TakeawayBookingServiceImpl;
use base::infrastructure::service::train_booking::TrainBookingServiceImpl;
use base::infrastructure::service::train_schedule::TrainScheduleServiceImpl;
use base::infrastructure::service::train_seat::TrainSeatServiceImpl;
use base::infrastructure::service::train_type::TrainTypeConfigurationServiceImpl;
use base::infrastructure::service::transaction::TransactionServiceImpl;
use base::infrastructure::service::user::UserServiceImpl;
use migration::MigratorTrait;
use sea_orm::Database;
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
    let rabbitmq_url = read_file_env("RABBITMQ_URL").expect("cannot get rabbitmq url");
    let tz_offset_hour_str = read_file_env("TZ_OFFSET_HOUR");

    let auto_schedule_days_str = read_file_env("AUTO_SCHEDULE_DAYS");

    let mini_io_endpoint = read_file_env("MINIO_ENDPOINT").expect("cannot get minio endpoint");
    let mini_io_access_key =
        read_file_env("MINIO_ACCESS_KEY").expect("cannot get minio access key");
    let mini_io_secret_key =
        read_file_env("MINIO_SECRET_KEY").expect("cannot get minio secret key");

    let data_base_path = read_file_env("DATA_PATH").expect("cannot get data path");

    let data_base_path = PathBuf::from_str(&data_base_path).expect("cannot parse data path");

    let tz_offset_hour = match tz_offset_hour_str {
        Some(hour_str) => hour_str
            .parse::<i32>()
            .expect("cannot parse tz offset hour"),
        // UTC+8: China Standard Time
        None => 8,
    };

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

    let conn = Database::connect(&database_url)
        .await
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));

    migration::Migrator::up(&conn, None)
        .await
        .unwrap_or_else(|_| panic!("Error applying migration to {}", database_url));

    let app_config = AppConfig {
        debug: debug_mode,
        server_name,
    };

    let session_manager_service_impl =
        Arc::new(SessionManagerServiceImpl::<SessionRepositoryImpl>::new(
            Arc::new(SessionRepositoryImpl::new(
                SessionRepositoryConfig::default(),
            )),
            SessionConfig::default(),
        ));

    let user_repository_impl = Arc::new(UserRepositoryImpl::new(conn.clone()));
    let city_repository_impl = Arc::new(CityRepositoryImpl::new(conn.clone()));
    let station_repository_impl = Arc::new(StationRepositoryImpl::new(conn.clone()));
    let train_repository_impl = Arc::new(TrainRepositoryImpl::new(conn.clone()));
    let route_repository_impl = Arc::new(RouteRepositoryImpl::new(conn.clone()));
    let transaction_repository_impl = Arc::new(TransactionRepositoryImpl::new(conn.clone()));
    let order_repository_impl = Arc::new(OrderRepositoryImpl::new(conn.clone()));
    let personal_info_repository_impl = Arc::new(PersonalInfoRepositoryImpl::new(conn.clone()));
    let hotel_repository_impl = Arc::new(HotelRepositoryImpl::new(conn.clone()));
    let hotel_rating_repository_impl = Arc::new(HotelRatingRepositoryImpl::new(conn.clone()));
    let seat_availability_repository_impl =
        Arc::new(SeatAvailabilityRepositoryImpl::new(conn.clone()));
    let train_schedule_repository_impl = Arc::new(TrainScheduleRepositoryImpl::new(conn.clone()));
    let dish_repository_impl = Arc::new(DishRepositoryImpl::new(conn.clone()));
    let takeaway_repository_impl = Arc::new(TakeawayShopRepositoryImpl::new(conn.clone()));
    let notify_repository_impl = Arc::new(NotifyRepositoryImpl::new(conn.clone()));
    let occupied_room_repository_impl = Arc::new(OccupiedRoomRepositoryImpl::new(conn.clone()));

    let s3_object_storage_service_impl = Arc::new(S3ObjectStorageServiceImpl::new(
        &mini_io_endpoint,
        &mini_io_access_key,
        &mini_io_secret_key,
    ));

    if let Err(e) = s3_object_storage_service_impl.init_buckets().await {
        error!("failed to initialize storage buckets: {}", e);
    }

    let user_service_impl = Arc::new(UserServiceImpl::<_, Argon2PasswordServiceImpl>::new(
        Arc::clone(&user_repository_impl),
    ));

    let user_manager_service_impl = Arc::new(UserManagerServiceImpl::new(
        Arc::clone(&user_service_impl),
        Arc::clone(&user_repository_impl),
        Arc::clone(&session_manager_service_impl),
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

    {
        let order_status_manager_service_impl = Arc::clone(&order_status_manager_service_impl);
        actix_web::rt::spawn(async move {
            order_status_manager_service_impl
                .order_status_daemon()
                .await;
        });
    }

    let user_profile_service_impl = Arc::new(UserProfileServiceImpl::new(
        Arc::clone(&session_manager_service_impl),
        Arc::clone(&user_repository_impl),
    ));

    let train_data_service_impl = Arc::new(TrainDataServiceImpl::new(
        debug_mode,
        data_base_path.clone(),
        Arc::clone(&city_repository_impl),
        Arc::clone(&station_repository_impl),
        Arc::clone(&train_repository_impl),
        Arc::clone(&route_repository_impl),
        Arc::clone(&dish_repository_impl),
        Arc::clone(&takeaway_repository_impl),
        Arc::clone(&s3_object_storage_service_impl),
    ));

    let geo_service_impl = Arc::new(GeoServiceImpl::new(Arc::clone(&city_repository_impl)));

    let station_service_impl = Arc::new(StationServiceImpl::new(
        Arc::clone(&station_repository_impl),
        Arc::clone(&geo_service_impl),
    ));

    let train_type_service_impl = Arc::new(TrainTypeConfigurationServiceImpl::new(Arc::clone(
        &train_repository_impl,
    )));

    let order_service_impl = Arc::new(OrderServiceImpl::new(
        Arc::clone(&order_repository_impl),
        tz_offset_hour,
    ));

    let transaction_service_impl = Arc::new(TransactionServiceImpl::new(
        Arc::clone(&user_repository_impl),
        Arc::clone(&transaction_repository_impl),
        Arc::clone(&order_service_impl),
        Arc::clone(&order_status_manager_service_impl),
    ));

    let transaction_application_service_impl = Arc::new(TransactionApplicationServiceImpl::new(
        app_config.debug,
        Arc::clone(&session_manager_service_impl),
        Arc::clone(&transaction_service_impl),
        Arc::clone(&transaction_repository_impl),
        Arc::clone(&user_service_impl),
        Arc::clone(&user_repository_impl),
    ));

    let geo_application_service_impl = Arc::new(GeoApplicationServiceImpl::new(
        Arc::clone(&geo_service_impl),
        Arc::clone(&station_service_impl),
    ));

    let personal_info_service_impl = Arc::new(PersonalInfoServiceImpl::new(
        Arc::clone(&session_manager_service_impl),
        Arc::clone(&personal_info_repository_impl),
    ));

    let route_service_impl = Arc::new(RouteServiceImpl::new(
        Arc::clone(&station_service_impl),
        Arc::clone(&route_repository_impl),
    ));

    let hotel_data_service_impl = Arc::new(HotelDataServiceImpl::new(
        app_config.debug,
        data_base_path,
        Arc::clone(&city_repository_impl),
        Arc::clone(&station_repository_impl),
        Arc::clone(&s3_object_storage_service_impl),
        Arc::clone(&hotel_repository_impl),
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

    let train_booking_service_impl = Arc::new(TrainBookingServiceImpl::new(
        Arc::clone(&train_schedule_repository_impl),
        Arc::clone(&train_seat_service_impl),
        Arc::clone(&transaction_service_impl),
        Arc::clone(&transaction_repository_impl),
        Arc::clone(&order_repository_impl),
    ));

    let dish_booking_service_impl = Arc::new(DishBookingServiceImpl::new(Arc::clone(
        &order_repository_impl,
    )));

    let takeaway_booking_service_impl = Arc::new(TakeawayBookingServiceImpl::new(Arc::clone(
        &order_repository_impl,
    )));

    let message_listener_service_impl = Arc::new(MessageListenerServiceImpl::new(
        MAX_CONCURRENT_WEBSOCKET_SESSION_PER_USER,
    ));

    let message_service_impl = Arc::new(MessageServiceImpl::new(
        Arc::clone(&message_listener_service_impl),
        Arc::clone(&notify_repository_impl),
        Arc::clone(&order_service_impl),
    ));

    let message_application_service_impl = Arc::new(MessageApplicationServiceImpl::new(
        Arc::clone(&message_service_impl),
        Arc::clone(&session_manager_service_impl),
    ));

    let train_type_configuration_service_impl = Arc::new(TrainTypeConfigurationServiceImpl::new(
        Arc::clone(&train_repository_impl),
    ));

    let train_schedule_service_impl = Arc::new(TrainScheduleServiceImpl::new(
        Arc::clone(&route_service_impl),
        Arc::clone(&train_repository_impl),
        Arc::clone(&train_schedule_repository_impl),
        Arc::clone(&route_repository_impl),
        tz_offset_hour,
    ));

    let dish_query_service_impl = Arc::new(DishQueryServiceImpl::new(
        Arc::clone(&dish_repository_impl),
        Arc::clone(&takeaway_repository_impl),
        Arc::clone(&train_repository_impl),
        Arc::clone(&session_manager_service_impl),
        Arc::clone(&train_schedule_service_impl),
        Arc::clone(&train_type_configuration_service_impl),
        Arc::clone(&station_service_impl),
    ));

    let train_dish_application_service_impl =
        Arc::new(base::infrastructure::application::service::train_dish::TrainDishApplicationServiceImpl::new(
            Arc::clone(&train_type_configuration_service_impl),
            Arc::clone(&dish_repository_impl),
            Arc::clone(&takeaway_repository_impl),
            Arc::clone(&train_schedule_repository_impl),
            Arc::clone(&train_repository_impl),
            Arc::clone(&personal_info_repository_impl),
            Arc::clone(&session_manager_service_impl),
            Arc::clone(&station_repository_impl),
            Arc::clone(&transaction_repository_impl),
            tz_offset_hour as u32,
        ));

    let train_query_service_impl = Arc::new(TrainQueryServiceImpl::new(
        Arc::clone(&train_schedule_service_impl),
        Arc::clone(&station_service_impl),
        Arc::clone(&route_service_impl),
        Arc::clone(&session_manager_service_impl),
        Arc::clone(&route_repository_impl),
        Arc::clone(&train_repository_impl),
        tz_offset_hour,
    ));

    let train_order_service_impl = Arc::new(TrainOrderServiceImpl::new(
        Arc::clone(&train_schedule_repository_impl),
        Arc::clone(&train_booking_service_impl),
        Arc::clone(&train_repository_impl),
        Arc::clone(&route_repository_impl),
        Arc::clone(&station_repository_impl),
        Arc::clone(&order_repository_impl),
        Arc::clone(&transaction_service_impl),
        Arc::clone(&session_manager_service_impl),
        Arc::clone(&personal_info_repository_impl),
        Arc::clone(&train_schedule_service_impl),
    ));

    let hotel_order_service_impl = Arc::new(HotelOrderServiceImpl::new(
        Arc::clone(&hotel_repository_impl),
        Arc::clone(&hotel_booking_service_impl),
        Arc::clone(&order_repository_impl),
        Arc::clone(&transaction_service_impl),
        Arc::clone(&session_manager_service_impl),
        Arc::clone(&personal_info_repository_impl),
    ));

    let user_repository: web::Data<dyn UserRepository> =
        web::Data::from(user_repository_impl as Arc<dyn UserRepository>);

    let user_service: web::Data<dyn UserService> =
        web::Data::from(user_service_impl as Arc<dyn UserService>);

    let session_manager_service: web::Data<dyn SessionManagerService> =
        web::Data::from(session_manager_service_impl as Arc<dyn SessionManagerService>);

    let user_manager_service: web::Data<dyn UserManagerService> =
        web::Data::from(user_manager_service_impl as Arc<dyn UserManagerService>);

    let user_profile_service: web::Data<dyn UserProfileService> =
        web::Data::from(user_profile_service_impl as Arc<dyn UserProfileService>);

    let train_data_service: web::Data<dyn TrainDataService> =
        web::Data::from(train_data_service_impl as Arc<dyn TrainDataService>);

    let route_service: web::Data<dyn RouteService> =
        web::Data::from(Arc::clone(&route_service_impl) as Arc<dyn RouteService>);

    let train_type_service: web::Data<dyn TrainTypeConfigurationService> = web::Data::from(
        Arc::clone(&train_type_service_impl) as Arc<dyn TrainTypeConfigurationService>,
    );

    let geo_application_service: web::Data<dyn GeoApplicationService> =
        web::Data::from(geo_application_service_impl as Arc<dyn GeoApplicationService>);

    let transaction_application_service: web::Data<dyn TransactionApplicationService> =
        web::Data::from(
            transaction_application_service_impl as Arc<dyn TransactionApplicationService>,
        );

    let personal_info_service: web::Data<dyn PersonalInfoService> =
        web::Data::from(personal_info_service_impl as Arc<dyn PersonalInfoService>);

    let object_storage_service: web::Data<dyn ObjectStorageService> =
        web::Data::from(s3_object_storage_service_impl as Arc<dyn ObjectStorageService>);

    let hotel_data_service: web::Data<dyn HotelDataService> =
        web::Data::from(hotel_data_service_impl as Arc<dyn HotelDataService>);

    let hotel_service: web::Data<dyn HotelService> =
        web::Data::from(hotel_service_impl as Arc<dyn HotelService>);

    let message_application_service: web::Data<dyn MessageApplicationService> =
        web::Data::from(message_application_service_impl as Arc<dyn MessageApplicationService>);

    let app_config_data = web::Data::new(app_config);

    let dish_order_status_consumer = Box::new(DishOrderStatusConsumer::new(
        Arc::clone(&dish_booking_service_impl),
        Arc::clone(&transaction_service_impl),
    )) as Box<dyn RabbitMQOrderStatusConsumer>;

    let takeaway_order_status_consumer = Box::new(TakeawayOrderStatusConsumer::new(
        Arc::clone(&takeaway_booking_service_impl),
        Arc::clone(&transaction_service_impl),
    )) as Box<dyn RabbitMQOrderStatusConsumer>;

    let train_order_status_consumer = Box::new(TrainOrderStatusConsumer::new(
        Arc::clone(&train_booking_service_impl),
        Arc::clone(&transaction_service_impl),
    ));

    let hotel_order_status_consumer = Box::new(HotelOrderStatusConsumer::new(
        Arc::clone(&hotel_booking_service_impl),
        Arc::clone(&transaction_service_impl),
    )) as Box<dyn RabbitMQOrderStatusConsumer>;

    let order_status_consumer = vec![
        train_order_status_consumer,
        dish_order_status_consumer,
        takeaway_order_status_consumer,
        hotel_order_status_consumer,
    ];

    let dish_query_service: web::Data<dyn DishQueryService> =
        web::Data::from(dish_query_service_impl as Arc<dyn DishQueryService>);

    let train_dish_application_service: web::Data<dyn TrainDishApplicationService> =
        web::Data::from(
            train_dish_application_service_impl as Arc<dyn TrainDishApplicationService>,
        );

    let train_order_service: web::Data<dyn TrainOrderService> =
        web::Data::from(train_order_service_impl as Arc<dyn TrainOrderService>);

    let hotel_order_service: web::Data<dyn HotelOrderService> =
        web::Data::from(hotel_order_service_impl as Arc<dyn HotelOrderService>);

    let _ = OrderStatusConsumerService::start(&rabbitmq_url, order_status_consumer)
        .await
        .expect("Failed to start order status consumer service");

    {
        let train_schedule_service_impl = Arc::clone(&train_schedule_service_impl);

        actix_web::rt::spawn(async move {
            train_schedule_service_impl
                .auto_plan_schedule_daemon(auto_schedule_days)
                .await;
        });
    }

    let train_query_service: web::Data<dyn TrainQueryService> =
        web::Data::from(train_query_service_impl as Arc<dyn TrainQueryService>);

    let message_listener_service: web::Data<dyn MessageListenerService> =
        web::Data::from(message_listener_service_impl as Arc<dyn MessageListenerService>);

    HttpServer::new(move || {
        App::new()
            .app_data(session_manager_service.clone())
            .app_data(user_repository.clone())
            .app_data(user_service.clone())
            .app_data(user_manager_service.clone())
            .app_data(user_profile_service.clone())
            .app_data(train_data_service.clone())
            .app_data(geo_application_service.clone())
            .app_data(personal_info_service.clone())
            .app_data(transaction_application_service.clone())
            .app_data(object_storage_service.clone())
            .app_data(hotel_data_service.clone())
            .app_data(route_service.clone())
            .app_data(train_type_service.clone())
            .app_data(hotel_service.clone())
            .app_data(message_listener_service.clone())
            .app_data(message_application_service.clone())
            // Step 3: Register your application service using `.app_data` function
            // Exercise 1.2.1D - 6: Your code here. (2 / 2)
            .app_data(train_query_service.clone())
            .app_data(dish_query_service.clone())
            .app_data(train_dish_application_service.clone())
            .app_data(train_order_service.clone())
            .app_data(hotel_order_service.clone())
            // Thinking 1.2.1D - 8: `App::new().app_data(...).app_data(...)`是什么设计模式的体现？
            // Good! Next, build your API endpoint in `api::train::schedule`
            .app_data(app_config_data.clone())
            .app_data(web::PayloadConfig::default().limit(MAX_BODY_LENGTH))
            .wrap(TracingLogger::default())
            .service(web::scope("/resource").configure(resource::scoped_config))
            .service(
                web::scope("/api")
                    .service(web::scope("/user").configure(api::user::scoped_config))
                    .service(web::scope("/general").configure(api::general::scoped_config))
                    .service(web::scope("/data").configure(api::data::scoped_config))
                    .service(web::scope("/payment").configure(api::payment::scoped_config))
                    .service(web::scope("/order").configure(api::order::scoped_config))
                    .service(web::scope("/notify").configure(api::notify::scoped_config))
                    // Step 6: Register your endpoint using `.service()` function
                    // Exercise 1.2.1D - 7: Your code here. (5 / 5)
                    .service(web::scope("/train").configure(api::train::scoped_config))
                    .service(web::scope("/hotel").configure(api::hotel::scoped_config)) // Congratulations! You have finished Task 1.2.1D!
                    .service(web::scope("/dish").configure(api::dish::scoped_config)),
            )
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

    if result.is_none() {
        if let Ok(env_str) = env::var(target_env) {
            result = Some(env_str);
        }
    }

    result
}
