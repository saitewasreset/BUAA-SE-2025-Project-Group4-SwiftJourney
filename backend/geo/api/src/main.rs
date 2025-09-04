use actix_web::{App, HttpServer, web};
use geo_base::application::service::geo::{GeoApplicationService, GeoApplicationServiceImpl};
use geo_base::application::service::internal::GeoInternalService;
use geo_base::infrastructure::repository::city::CityRepositoryImpl;
use geo_base::infrastructure::repository::station::StationRepositoryImpl;
use geo_base::infrastructure::service::event::GeoEventServiceImpl;
use geo_base::infrastructure::service::geo::GeoServiceImpl;
use geo_base::infrastructure::service::internal::GeoInternalServiceImpl;
use geo_base::infrastructure::service::station::StationServiceImpl;
use migration::MigratorTrait;
use sea_orm::Database;
use shared::api::{MAX_BODY_LENGTH, read_file_env};
use shared::event::queue::EventService;
use std::sync::Arc;
use tracing_actix_web::TracingLogger;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt::init();

    let database_url = read_file_env("DATABASE_URL").expect("cannot get database url");
    let rabbitmq_url = read_file_env("RABBITMQ_URL").expect("cannot get rabbitmq url");

    let conn = Database::connect(&database_url)
        .await
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));

    migration::Migrator::up(&conn, None)
        .await
        .unwrap_or_else(|_| panic!("Error applying migration to {}", database_url));

    let geo_event_service_impl = GeoEventServiceImpl::new(&rabbitmq_url)
        .await
        .expect("Cannot connect to rabbitmq");

    geo_event_service_impl
        .clone()
        .init_consumer()
        .await
        .expect("Failed to init consumer");

    let city_repository_impl = Arc::new(CityRepositoryImpl::new(conn.clone()));
    let station_repository_impl = Arc::new(StationRepositoryImpl::new(conn.clone()));

    let geo_service_impl = Arc::new(GeoServiceImpl::<CityRepositoryImpl>::new(Arc::clone(
        &city_repository_impl,
    )));

    let station_service_impl = Arc::new(StationServiceImpl::<
        StationRepositoryImpl,
        GeoServiceImpl<CityRepositoryImpl>,
    >::new(
        Arc::clone(&station_repository_impl),
        Arc::clone(&geo_service_impl),
    ));

    let geo_app_service_impl = Arc::new(GeoApplicationServiceImpl::<
        GeoServiceImpl<CityRepositoryImpl>,
        StationServiceImpl<StationRepositoryImpl, GeoServiceImpl<CityRepositoryImpl>>,
    >::new(
        Arc::clone(&geo_service_impl),
        Arc::clone(&station_service_impl),
    ));

    let geo_internal_service_impl = Arc::new(GeoInternalServiceImpl::new(
        Arc::clone(&city_repository_impl),
        Arc::clone(&station_repository_impl),
        Arc::clone(&geo_event_service_impl),
    ));

    let geo_app_service: web::Data<dyn GeoApplicationService> =
        web::Data::from(geo_app_service_impl as Arc<dyn GeoApplicationService>);

    let geo_internal_service: web::Data<dyn GeoInternalService> =
        web::Data::from(geo_internal_service_impl as Arc<dyn GeoInternalService>);

    tokio::task::spawn(async move {
        HttpServer::new(move || {
            App::new()
                .app_data(geo_internal_service.clone())
                .app_data(web::PayloadConfig::default().limit(MAX_BODY_LENGTH))
                .wrap(TracingLogger::default())
                .service(web::scope("/internal").configure(geo_api::internal::scoped_config))
        })
        .bind(("0.0.0.0", 23333))
        .unwrap()
        .run()
        .await
        .unwrap();
    });

    HttpServer::new(move || {
        App::new()
            .app_data(geo_app_service.clone())
            .app_data(web::PayloadConfig::default().limit(MAX_BODY_LENGTH))
            .wrap(TracingLogger::default())
            .service(
                web::scope("/api")
                    .service(web::scope("/general").configure(geo_api::general::scoped_config)),
            )
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await?;

    Ok(())
}
