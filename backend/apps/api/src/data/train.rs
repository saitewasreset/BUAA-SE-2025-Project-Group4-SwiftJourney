use crate::{ApiResponse, ApplicationErrorBox, parse_request_body};
use actix_web::post;
use actix_web::web::{Bytes, Data};
use base::application::commands::train_data::{
    LoadCityCommand, LoadStationCommand, LoadTrainNumberCommand, LoadTrainTypeCommand,
};
use base::application::service::train_data::TrainDataService;

#[post("/city")]
async fn load_city_data(
    body: Bytes,
    train_data_service: Data<dyn TrainDataService>,
) -> Result<ApiResponse<()>, ApplicationErrorBox> {
    let data: LoadCityCommand = parse_request_body(body)?;

    train_data_service.load_city(data).await?;

    ApiResponse::ok(())
}

#[post("/station")]
async fn load_station_data(
    body: Bytes,
    train_data_service: Data<dyn TrainDataService>,
) -> Result<ApiResponse<()>, ApplicationErrorBox> {
    let data: LoadStationCommand = parse_request_body(body)?;

    train_data_service.load_station(data).await?;

    ApiResponse::ok(())
}

#[post("/train_type")]
async fn load_train_type_data(
    body: Bytes,
    train_data_service: Data<dyn TrainDataService>,
) -> Result<ApiResponse<()>, ApplicationErrorBox> {
    let data: LoadTrainTypeCommand = parse_request_body(body)?;

    train_data_service.load_train_type(data).await?;

    ApiResponse::ok(())
}

#[post("/train_number")]
async fn load_train_number_data(
    body: Bytes,
    train_data_service: Data<dyn TrainDataService>,
) -> Result<ApiResponse<()>, ApplicationErrorBox> {
    let data: LoadTrainNumberCommand = parse_request_body(body)?;

    train_data_service.load_train_number(data).await?;

    ApiResponse::ok(())
}
