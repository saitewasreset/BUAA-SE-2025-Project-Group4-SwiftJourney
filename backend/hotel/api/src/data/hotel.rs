use actix_web::post;
use actix_web::web::{Bytes, Data};
use hotel_base::application::commands::hotel_data::LoadHotelCommand;
use hotel_base::application::service::hotel_data::HotelDataService;
use sea_orm::DatabaseConnection;
use shared::api::{ApiResponse, ApplicationErrorBox, parse_request_body};

#[post("/hotel")]
async fn load_hotel_data(
    body: Bytes,
    hotel_data_service: Data<dyn HotelDataService>,
    db: Data<DatabaseConnection>,
) -> Result<ApiResponse<()>, ApplicationErrorBox> {
    let data: LoadHotelCommand = parse_request_body(body)?;

    hotel_data_service.load_hotel(data, db.get_ref()).await?;

    ApiResponse::ok(())
}
