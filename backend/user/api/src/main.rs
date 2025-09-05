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
use sea_orm::Database;
use shared::api::{MAX_BODY_LENGTH, read_file_env};
use shared::event::queue::EventService;
use std::sync::Arc;
use tracing_actix_web::TracingLogger;
use user_base::application::service::internal::UserInternalService;
use user_base::application::service::personal_info::PersonalInfoService;
use user_base::application::service::user_manager::UserManagerService;
use user_base::application::service::user_profile::UserProfileService;
use user_base::domain::model::session_config::SessionConfig;
use user_base::domain::repository::session::SessionRepositoryConfig;
use user_base::domain::repository::user::UserRepository;
use user_base::domain::service::session::SessionManagerService;
use user_base::domain::service::user::UserService;
use user_base::infrastructure::application::service::personal_info::PersonalInfoServiceImpl;
use user_base::infrastructure::application::service::user_manager::UserManagerServiceImpl;
use user_base::infrastructure::application::service::user_profile::UserProfileServiceImpl;
use user_base::infrastructure::repository::personal_info::PersonalInfoRepositoryImpl;
use user_base::infrastructure::repository::session::SessionRepositoryImpl;
use user_base::infrastructure::repository::user::UserRepositoryImpl;
use user_base::infrastructure::service::event::UserEventServiceImpl;
use user_base::infrastructure::service::internal::UserInternalServiceImpl;
use user_base::infrastructure::service::password::Argon2PasswordServiceImpl;
use user_base::infrastructure::service::session::SessionManagerServiceImpl;
use user_base::infrastructure::service::user::UserServiceImpl;
use user_migration::MigratorTrait;

#[actix_web::main]

async fn main() -> std::io::Result<()> {
    // env_logger::init_from_env(Env::default().default_filter_or("info"));
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt::init();

    let database_url = read_file_env("DATABASE_URL").expect("cannot get database url");
    let rabbitmq_url = read_file_env("RABBITMQ_URL").expect("cannot get rabbitmq url");

    let conn = Database::connect(&database_url)
        .await
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));

    user_migration::Migrator::up(&conn, None)
        .await
        .unwrap_or_else(|_| panic!("Error applying migration to {}", database_url));

    let user_event_service_impl = UserEventServiceImpl::new(&rabbitmq_url)
        .await
        .expect("cannot connect to rabbitmq");

    user_event_service_impl
        .clone()
        .init_consumer()
        .await
        .expect("failed to init consumer");

    let session_manager_service_impl =
        Arc::new(SessionManagerServiceImpl::<SessionRepositoryImpl>::new(
            Arc::new(SessionRepositoryImpl::new(
                SessionRepositoryConfig::default(),
            )),
            SessionConfig::default(),
        ));

    let user_repository_impl = Arc::new(UserRepositoryImpl::new(conn.clone()));

    let personal_info_repository_impl = Arc::new(PersonalInfoRepositoryImpl::new(conn.clone()));

    let user_service_impl = Arc::new(UserServiceImpl::<_, Argon2PasswordServiceImpl, _>::new(
        Arc::clone(&user_repository_impl),
        Arc::clone(&user_event_service_impl),
    ));

    let user_manager_service_impl = Arc::new(UserManagerServiceImpl::new(
        Arc::clone(&user_service_impl),
        Arc::clone(&user_repository_impl),
        Arc::clone(&session_manager_service_impl),
    ));

    let user_profile_service_impl = Arc::new(UserProfileServiceImpl::new(
        Arc::clone(&session_manager_service_impl),
        Arc::clone(&user_repository_impl),
        Arc::clone(&user_event_service_impl),
    ));

    let personal_info_service_impl = Arc::new(PersonalInfoServiceImpl::new(
        Arc::clone(&session_manager_service_impl),
        Arc::clone(&personal_info_repository_impl),
        Arc::clone(&user_event_service_impl),
    ));

    let user_internal_service_impl = Arc::new(UserInternalServiceImpl::new(
        Arc::clone(&user_service_impl),
        Arc::clone(&session_manager_service_impl),
        Arc::clone(&user_repository_impl),
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

    let personal_info_service: web::Data<dyn PersonalInfoService> =
        web::Data::from(personal_info_service_impl as Arc<dyn PersonalInfoService>);

    let user_internal_service: web::Data<dyn UserInternalService> =
        web::Data::from(user_internal_service_impl as Arc<dyn UserInternalService>);

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
