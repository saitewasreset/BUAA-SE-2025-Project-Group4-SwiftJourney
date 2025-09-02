use actix_web::{get, web};
use geo_base::application::service::geo::GeoApplicationService;
use shared::api::{ApiResponse, ApplicationErrorBox};
use shared::application_error::ApplicationError as SharedAppError;

#[get("/city_station_info")]
pub async fn get_city_station_info(
    service: web::Data<dyn GeoApplicationService>,
) -> Result<ApiResponse<geo_base::application::service::geo::CityStationInfoDTO>, ApplicationErrorBox> {
    let dto = service.get_city_station_info().await.map_err(
    |e: Box<dyn shared::application_error::ApplicationError>| {
            let msg = e.error_message();
            let code = e.error_code();
            struct Adapter(u32, String);
            impl std::fmt::Debug for Adapter {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{}", self.1)
                }
            }
            impl std::fmt::Display for Adapter {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{}", self.1)
                }
            }
            impl std::error::Error for Adapter {}
            impl SharedAppError for Adapter {
                fn error_code(&self) -> u32 {
                    self.0
                }
                fn error_message(&self) -> String {
                    self.1.clone()
                }
            }
            ApplicationErrorBox::from(Box::new(Adapter(code, msg)) as Box<dyn SharedAppError>)
        },
    )?;
    ApiResponse::ok(dto)
}
