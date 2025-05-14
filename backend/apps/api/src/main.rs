// Step 1: Read the sentences below.
// Thinking 1.2.1D - 6: 你认为下面的句子来自哪个游戏？其用意是什么？
// Thinking 1.2.1D - 7: 什么是“管理式民主”（Managed Democracy）？你认为它是真实的民主吗？
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
use actix_web::web::resource;
use actix_web::{App, HttpServer, web};
use api::{AppConfig, MAX_BODY_LENGTH, resource};
use base::application::service::geo::GeoApplicationService;
use base::application::service::personal_info::PersonalInfoService;
use base::application::service::train_data::TrainDataService;
use base::application::service::transaction::TransactionApplicationService;
use base::application::service::user_manager::UserManagerService;
use base::application::service::user_profile::UserProfileService;
use base::domain::model::session_config::SessionConfig;
use base::domain::repository::session::SessionRepositoryConfig;
use base::domain::repository::user::UserRepository;
use base::domain::service::object_storage::ObjectStorageService;
use base::domain::service::session::SessionManagerService;
use base::domain::service::user::UserService;
use base::infrastructure::application::service::geo::GeoApplicationServiceImpl;
use base::infrastructure::application::service::personal_info::PersonalInfoServiceImpl;
use base::infrastructure::application::service::train_data::TrainDataServiceImpl;
use base::infrastructure::application::service::transaction::TransactionApplicationServiceImpl;
use base::infrastructure::application::service::user_manager::UserManagerServiceImpl;
use base::infrastructure::application::service::user_profile::UserProfileServiceImpl;
use base::infrastructure::repository::city::CityRepositoryImpl;
use base::infrastructure::repository::order::OrderRepositoryImpl;
use base::infrastructure::repository::personal_info::PersonalInfoRepositoryImpl;
use base::infrastructure::repository::route::RouteRepositoryImpl;
use base::infrastructure::repository::session::SessionRepositoryImpl;
use base::infrastructure::repository::station::StationRepositoryImpl;
use base::infrastructure::repository::train::TrainRepositoryImpl;
use base::infrastructure::repository::transaction::TransactionRepositoryImpl;
use base::infrastructure::repository::user::UserRepositoryImpl;
use base::infrastructure::service::geo::GeoServiceImpl;
use base::infrastructure::service::object_storage::S3ObjectStorageServiceImpl;
use base::infrastructure::service::order::OrderServiceImpl;
use base::infrastructure::service::order_status::OrderStatusManagerServiceImpl;
use base::infrastructure::service::password::Argon2PasswordServiceImpl;
use base::infrastructure::service::session::SessionManagerServiceImpl;
use base::infrastructure::service::station::StationServiceImpl;
use base::infrastructure::service::transaction::TransactionServiceImpl;
use base::infrastructure::service::user::UserServiceImpl;
use migration::MigratorTrait;
use sea_orm::Database;
use std::env::VarError;
use std::sync::Arc;
use std::{env, fs};
use tracing::{error, instrument, warn};
use tracing_actix_web::TracingLogger;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // env_logger::init_from_env(Env::default().default_filter_or("info"));

    tracing_subscriber::fmt::init();

    let database_url = read_file_env("DATABASE_URL").expect("cannot get database url");
    let tz_offset_hour_str = read_file_env("TZ_OFFSET_HOUR");
    let mini_io_endpoint = read_file_env("MINIO_ENDPOINT").expect("cannot get minio endpoint");
    let mini_io_access_key =
        read_file_env("MINIO_ACCESS_KEY").expect("cannot get minio access key");
    let mini_io_secret_key =
        read_file_env("MINIO_SECRET_KEY").expect("cannot get minio secret key");

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

    migration::Migrator::up(&conn, None)
        .await
        .unwrap_or_else(|_| panic!("Error applying migration to {}", database_url));

    let app_config = AppConfig { debug: debug_mode };

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

    let order_status_manager_service_impl = Arc::new(OrderStatusManagerServiceImpl::new());

    let user_profile_service_impl = Arc::new(UserProfileServiceImpl::new(
        Arc::clone(&session_manager_service_impl),
        Arc::clone(&user_repository_impl),
    ));

    let train_data_service_impl = Arc::new(TrainDataServiceImpl::new(
        debug_mode,
        Arc::clone(&city_repository_impl),
        Arc::clone(&station_repository_impl),
        Arc::clone(&train_repository_impl),
        Arc::clone(&route_repository_impl),
    ));

    let geo_service_impl = Arc::new(GeoServiceImpl::new(Arc::clone(&city_repository_impl)));

    let station_service_impl = Arc::new(StationServiceImpl::new(
        Arc::clone(&station_repository_impl),
        Arc::clone(&geo_service_impl),
    ));

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

    let app_config_data = web::Data::new(app_config);

    // Step 2: Create instance of your application service,
    // and wrap it with `web::Data::new`
    // HINT: You can borrow web::Data<T> as &Arc<T>
    // that means you can pass a &web::Data<T> to `Arc::clone`
    // Exercise 1.2.1D - 6: Your code here. (1 / 2)

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
            // Step 3: Register your application service using `.app_data` function
            // Exercise 1.2.1D - 6: Your code here. (2 / 2)
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
                    .service(web::scope("/order").configure(api::order::scoped_config)),
            )
        // Step 6: Register your endpoint using `.service()` function
        // Exercise 1.2.1D - 7: Your code here. (5 / 5)
        // Congratulations! You have finished Task 1.2.1D!
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
