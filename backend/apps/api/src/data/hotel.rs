use crate::{ApiResponse, ApplicationErrorBox, parse_request_body};
use actix_web::post;
use actix_web::web::{Bytes, Data};
use base::application::commands::hotel_data::LoadHotelCommand;
use base::application::service::hotel_data::HotelDataService;

#[post("/hotel")]
async fn load_hotel_data(
    body: Bytes,
    hotel_data_service: Data<dyn HotelDataService>,
) -> Result<ApiResponse<()>, ApplicationErrorBox> {
    let data: LoadHotelCommand = parse_request_body(body)?;

    hotel_data_service.load_hotel(data).await?;

    ApiResponse::ok(())
}
