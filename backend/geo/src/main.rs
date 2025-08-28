use actix_web::{App, HttpServer, web};
use migration::MigratorTrait;
use sea_orm::Database;
use std::env;
use std::sync::Arc;
use tracing_subscriber::fmt::init as tracing_init;

use geo::infrastructure::repository::{city::CityRepositoryImpl, station::StationRepositoryImpl};
use geo::web as geo_web;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let _ = dotenvy::dotenv();
    tracing_init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is required");
    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8082);
    let debug = env::var("DEBUG").is_ok();

    let conn = Database::connect(&database_url)
        .await
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));

    migration::Migrator::up(&conn, None)
        .await
        .unwrap_or_else(|_| panic!("Error applying migration to {}", database_url));

    let city_repo = Arc::new(CityRepositoryImpl::new(conn.clone()))
        as Arc<dyn geo::domain::repository::city::CityRepository>;
    let station_repo = Arc::new(StationRepositoryImpl::new(conn.clone()))
        as Arc<dyn geo::domain::repository::station::StationRepository>;
    let state = web::Data::new(geo_web::AppState {
        city_repo: Arc::clone(&city_repo),
        station_repo: Arc::clone(&station_repo),
        debug,
    });
    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .service(web::scope("/api").service(geo_web::scope()))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
