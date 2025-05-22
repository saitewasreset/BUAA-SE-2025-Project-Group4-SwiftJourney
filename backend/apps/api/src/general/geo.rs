use crate::general::web;
use crate::{ApiResponse, ApplicationErrorBox};
use actix_web::get;
use base::application::service::geo::{CityStationInfoDTO, GeoApplicationService};

#[get("/city_stations")]
pub async fn get_city_station_info(
    geo_service: web::Data<dyn GeoApplicationService>,
) -> Result<ApiResponse<CityStationInfoDTO>, ApplicationErrorBox> {
    let city_station_info = geo_service.get_city_station_info().await?;
    ApiResponse::ok(city_station_info)
}

#[get("/city")]
pub async fn get_city_info(
    geo_service: web::Data<dyn GeoApplicationService>,
) -> Result<ApiResponse<CityStationInfoDTO>, ApplicationErrorBox> {
    let city_info = geo_service.get_city_info().await?;
    ApiResponse::ok(city_info)
}
