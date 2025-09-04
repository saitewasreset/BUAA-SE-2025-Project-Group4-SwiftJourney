use std::{
    env::{self, VarError},
    sync::Arc,
};

use actix_web::{App, HttpServer, web};
use migration::MigratorTrait;
use object_storage_api::resource;
use object_storage_base::{
    application::service::internal::ObjectStorageInternalService,
    domain::service::object_storage::ObjectStorageService,
    infrastructure::service::{
        internal::ObjectStorageInternalServiceImpl, object_storage::S3ObjectStorageServiceImpl,
    },
};
use sea_orm::Database;
use shared::api::read_file_env;
use shared::api::{AppConfig, MAX_BODY_LENGTH};
use tracing::error;
use tracing_actix_web::TracingLogger;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // env_logger::init_from_env(Env::default().default_filter_or("info"));
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt::init();

    let server_name = read_file_env("SERVER_NAME").expect("cannot get server name");

    let database_url = read_file_env("DATABASE_URL").expect("cannot get database url");

    let mini_io_endpoint = read_file_env("MINIO_ENDPOINT").expect("cannot get minio endpoint");
    let mini_io_access_key =
        read_file_env("MINIO_ACCESS_KEY").expect("cannot get minio access key");
    let mini_io_secret_key =
        read_file_env("MINIO_SECRET_KEY").expect("cannot get minio secret key");

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

    let s3_object_storage_service_impl = Arc::new(S3ObjectStorageServiceImpl::new(
        &mini_io_endpoint,
        &mini_io_access_key,
        &mini_io_secret_key,
    ));

    if let Err(e) = s3_object_storage_service_impl.init_buckets().await {
        error!("failed to initialize storage buckets: {}", e);
    }

    let object_storage_internal_service_impl = Arc::new(ObjectStorageInternalServiceImpl::new(
        s3_object_storage_service_impl.clone(),
    ));

    let object_storage_service: web::Data<dyn ObjectStorageService> =
        web::Data::from(s3_object_storage_service_impl as Arc<dyn ObjectStorageService>);

    let app_config_data = web::Data::new(app_config);

    let object_storage_internal_service: web::Data<dyn ObjectStorageInternalService> =
        web::Data::from(
            object_storage_internal_service_impl as Arc<dyn ObjectStorageInternalService>,
        );

    tokio::task::spawn(async move {
        HttpServer::new(move || {
            App::new()
                .app_data(object_storage_internal_service.clone())
                .app_data(web::PayloadConfig::default().limit(MAX_BODY_LENGTH))
                .wrap(TracingLogger::default())
                .service(
                    web::scope("/internal").configure(object_storage_api::internal::scoped_config),
                )
        })
        .bind(("0.0.0.0", 23333))
        .unwrap()
        .run()
        .await
        .unwrap();
    });

    HttpServer::new(move || {
        App::new()
            .app_data(object_storage_service.clone())
            .app_data(app_config_data.clone())
            .app_data(web::PayloadConfig::default().limit(MAX_BODY_LENGTH))
            .wrap(TracingLogger::default())
            .service(web::scope("/resource").configure(resource::scoped_config))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await?;

    Ok(())
}
