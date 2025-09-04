use actix_web::web::Data;
use actix_web::{get, post, web};
use geo_base::application::service::internal::GeoInternalService;
use shared::api::{ApiResponse, ApplicationErrorBox, parse_request_body};
use shared::application_error::ApplicationError;
use shared::internal::geo::command::{SaveCityProvinceMapCommand, SaveStationCityMapCommand};
use shared::internal::geo::dto::{
    CityDTO, CityInfoDTO, CityStationInfoDTO, DbCityDTO, DbStationDTO,
};

#[get("/get_cities")]
pub async fn get_cities(
    geo_internal_service: Data<dyn GeoInternalService>,
) -> Result<ApiResponse<Vec<CityDTO>>, ApplicationErrorBox> {
    let result = geo_internal_service
        .get_cities()
        .await
        .map_err(|e| Box::new(e) as Box<dyn ApplicationError>)?;

    ApiResponse::ok(result)
}

#[get("/get_city_info")]
pub async fn get_city_info(
    geo_internal_service: Data<dyn GeoInternalService>,
) -> Result<ApiResponse<CityInfoDTO>, ApplicationErrorBox> {
    let result = geo_internal_service
        .get_city_info_list()
        .await
        .map_err(|e| Box::new(e) as Box<dyn ApplicationError>)?;

    ApiResponse::ok(result)
}

#[get("/get_stations")]
pub async fn get_stations(
    geo_internal_service: Data<dyn GeoInternalService>,
) -> Result<ApiResponse<CityStationInfoDTO>, ApplicationErrorBox> {
    let result = geo_internal_service
        .get_city_station_info()
        .await
        .map_err(|e| Box::new(e) as Box<dyn ApplicationError>)?;

    ApiResponse::ok(result)
}

#[get("/db_get_cities")]
pub async fn db_get_cities(
    geo_internal_service: Data<dyn GeoInternalService>,
) -> Result<ApiResponse<Vec<DbCityDTO>>, ApplicationErrorBox> {
    let result = geo_internal_service
        .db_get_cities()
        .await
        .map_err(|e| Box::new(e) as Box<dyn ApplicationError>)?;

    ApiResponse::ok(result)
}

#[get("/db_get_stations")]
pub async fn db_get_stations(
    geo_internal_service: Data<dyn GeoInternalService>,
) -> Result<ApiResponse<Vec<DbStationDTO>>, ApplicationErrorBox> {
    let result = geo_internal_service
        .db_get_stations()
        .await
        .map_err(|e| Box::new(e) as Box<dyn ApplicationError>)?;

    ApiResponse::ok(result)
}

#[post("/save_city_province_map")]
pub async fn save_city_province_map(
    geo_internal_service: Data<dyn GeoInternalService>,
    body: web::Bytes,
) -> Result<ApiResponse<()>, ApplicationErrorBox> {
    let cmd: SaveCityProvinceMapCommand = parse_request_body(body)?;

    geo_internal_service
        .save_city_province_map(cmd)
        .await
        .map_err(|e| Box::new(e) as Box<dyn ApplicationError>)?;

    ApiResponse::ok(())
}

#[post("/save_station_city_map")]
pub async fn save_station_city_map(
    geo_internal_service: Data<dyn GeoInternalService>,
    body: web::Bytes,
) -> Result<ApiResponse<()>, ApplicationErrorBox> {
    let cmd: SaveStationCityMapCommand = parse_request_body(body)?;

    geo_internal_service
        .save_station_city_map(cmd)
        .await
        .map_err(|e| Box::new(e) as Box<dyn ApplicationError>)?;

    ApiResponse::ok(())
}

pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_cities)
        .service(get_city_info)
        .service(get_stations)
        .service(db_get_cities)
        .service(db_get_stations)
        .service(save_city_province_map)
        .service(save_station_city_map);
}
