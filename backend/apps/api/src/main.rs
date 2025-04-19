use actix_web::{App, HttpServer, web};
use api::MAX_BODY_LENGTH;
use base::domain::model::session_config::SessionConfig;
use base::domain::repository::session::SessionRepositoryConfig;
use base::infrastructure::repository::session::SessionRepositoryImpl;
use base::infrastructure::repository::user::UserRepositoryImpl;
use base::infrastructure::service::password::Argon2PasswordServiceImpl;
use base::infrastructure::service::session::SessionManagerServiceImpl;
use base::infrastructure::service::user::UserServiceImpl;
use migration::MigratorTrait;
use sea_orm::Database;
use std::sync::Arc;
use std::{env, fs};
use tracing::{instrument, warn};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let database_url = read_file_env("DATABASE_URL").expect("cannot get database url");

    let conn = Database::connect(&database_url)
        .await
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));

    migration::Migrator::up(&conn, None)
        .await
        .unwrap_or_else(|_| panic!("Error applying migration to {}", database_url));

    let session_manager_service =
        web::Data::new(SessionManagerServiceImpl::<SessionRepositoryImpl>::new(
            Arc::new(SessionRepositoryImpl::new(
                SessionRepositoryConfig::default(),
            )),
            SessionConfig::default(),
        ));

    let user_repository = web::Data::new(UserRepositoryImpl::new(conn.clone()));

    let user_service = web::Data::new(UserServiceImpl::<_, Argon2PasswordServiceImpl>::new(
        Arc::clone(&user_repository),
    ));

    HttpServer::new(move || {
        App::new()
            .app_data(session_manager_service.clone())
            .app_data(user_repository.clone())
            .app_data(user_service.clone())
            .app_data(web::PayloadConfig::default().limit(MAX_BODY_LENGTH))
            .service(
                web::scope("/api").service(web::scope("/user").configure(api::user::scoped_config)),
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
