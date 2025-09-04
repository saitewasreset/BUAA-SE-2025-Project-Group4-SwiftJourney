pub use crate::application_error::ApplicationError;
use crate::{
    API_BAD_REQUEST_MESSAGE_TEMPLATE, API_FORBIDDEN_CODE, API_FORBIDDEN_MESSAGE_TEMPLATE,
    API_SUCCESS_CODE, API_SUCCESS_MESSAGE, InternalApi,
};
use actix_web::body::BoxBody;
use actix_web::http::header::ContentType;
use actix_web::web::Bytes;
use actix_web::{HttpRequest, HttpResponse, Responder, ResponseError};
use dyn_fmt::AsStrFormatExt;
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};
use std::{env, fs};
use thiserror::Error;
use tracing::{instrument, warn};

pub const MAX_BODY_LENGTH: usize = 5 * 1024 * 1024 * 1024;

pub struct ApplicationErrorBox(pub Box<dyn ApplicationError>);

impl Display for ApplicationErrorBox {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl Debug for ApplicationErrorBox {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, f)
    }
}

impl From<Box<dyn ApplicationError>> for ApplicationErrorBox {
    fn from(value: Box<dyn ApplicationError>) -> Self {
        Self(value)
    }
}

impl From<ApplicationErrorBox> for Box<dyn ApplicationError> {
    fn from(value: ApplicationErrorBox) -> Self {
        value.0
    }
}

impl ResponseError for ApplicationErrorBox {
    fn error_response(&self) -> HttpResponse<BoxBody> {
        let api_response: ApiResponse<()> = ApiResponse {
            code: self.0.error_code(),
            message: self.0.error_message(),
            data: None,
        };

        let body = serde_json::to_string(&api_response).unwrap();

        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(body)
    }
}

#[derive(Debug)]
pub struct ModeError;

impl Display for ModeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ModeError")
    }
}

impl std::error::Error for ModeError {}

impl ApplicationError for ModeError {
    fn error_code(&self) -> u32 {
        API_FORBIDDEN_CODE
    }

    fn error_message(&self) -> String {
        API_FORBIDDEN_MESSAGE_TEMPLATE.format(["debug mode is not enabled"])
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AppConfig {
    pub debug: bool,
    pub server_name: String,
}

#[derive(Serialize, Deserialize)]
pub struct ApiResponse<T>
where
    T: Serialize,
{
    pub code: u32,
    pub message: String,
    pub data: Option<T>,
}

impl<T> ApiResponse<T>
where
    T: Serialize,
{
    pub fn ok(data: T) -> Result<Self, ApplicationErrorBox> {
        Ok(ApiResponse {
            code: API_SUCCESS_CODE,
            message: API_SUCCESS_MESSAGE.to_string(),
            data: Some(data),
        })
    }
}

impl<T> Responder for ApiResponse<T>
where
    T: Serialize,
{
    type Body = BoxBody;
    fn respond_to(self, _for_super_earth: &HttpRequest) -> HttpResponse<Self::Body> {
        let body = serde_json::to_string(&self).unwrap();

        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(body)
    }
}

impl<T> From<Result<T, Box<dyn ApplicationError>>> for ApiResponse<T>
where
    T: Serialize,
{
    fn from(value: Result<T, Box<dyn ApplicationError>>) -> Self {
        match value {
            Ok(data) => ApiResponse {
                code: API_SUCCESS_CODE,
                message: API_SUCCESS_MESSAGE.to_string(),
                data: Some(data),
            },
            Err(err) => ApiResponse {
                code: err.error_code(),
                message: err.error_message(),
                data: None,
            },
        }
    }
}

#[derive(Error, Debug)]
pub enum SessionIdError {
    #[error("no session id provided")]
    NoSessionIdProvided,
}

impl ApplicationError for SessionIdError {
    fn error_code(&self) -> u32 {
        403
    }

    fn error_message(&self) -> String {
        API_FORBIDDEN_MESSAGE_TEMPLATE.format(&[self.to_string()])
    }
}

pub fn get_session_id(request: &HttpRequest) -> Result<String, Box<dyn ApplicationError>> {
    if let Some(provided_session_id) = request.cookie("session_id") {
        Ok(provided_session_id.value().to_string())
    } else {
        Err(SessionIdError::NoSessionIdProvided.into())
    }
}

#[derive(Error, Debug)]
pub enum ParseRequestBodyError {
    #[error("invalid request body")]
    InvalidBody(#[from] serde_json::Error),
}

impl ApplicationError for ParseRequestBodyError {
    fn error_code(&self) -> u32 {
        400
    }

    fn error_message(&self) -> String {
        API_BAD_REQUEST_MESSAGE_TEMPLATE.format(&["invalid json"])
    }
}

pub fn parse_request_body<T: DeserializeOwned>(
    raw_body: Bytes,
) -> Result<T, Box<dyn ApplicationError>> {
    serde_json::from_slice(&raw_body)
        .map_err(|e| Box::new(ParseRequestBodyError::InvalidBody(e)) as Box<dyn ApplicationError>)
}

#[instrument]
pub fn read_file_env(target_env: &str) -> Option<String> {
    let mut result: Option<String> = None;
    if let Ok(file_path) = env::var(format!("{}_FILE", target_env)) {
        match fs::read_to_string(&file_path) {
            Ok(val) => result = Some(val.trim().to_string()),
            Err(e) => {
                warn!("cannot read env file {}: {}", file_path, e)
            }
        }
    }

    if result.is_none()
        && let Ok(env_str) = env::var(target_env)
    {
        result = Some(env_str);
    }

    result
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UserInternalServiceApi {
    VerifyPassword,
    VerifyPaymentPassword,
    SetPaymentPassword,
    ClearWrongPaymentPasswordTried,
    GetSession,
    GetUserInfo,
    DbGetUserInfo,
    DbGetPersonalInfo,
}

impl InternalApi for UserInternalServiceApi {
    fn name(&self) -> &'static str {
        match self {
            UserInternalServiceApi::VerifyPassword => "verify_password",
            UserInternalServiceApi::VerifyPaymentPassword => "verify_payment_password",
            UserInternalServiceApi::SetPaymentPassword => "set_payment_password",
            UserInternalServiceApi::ClearWrongPaymentPasswordTried => {
                "clear_wrong_payment_password_tried"
            }
            UserInternalServiceApi::GetSession => "get_session",
            UserInternalServiceApi::GetUserInfo => "get_user_info",
            UserInternalServiceApi::DbGetUserInfo => "db_get_user_info",
            UserInternalServiceApi::DbGetPersonalInfo => "db_get_personal_info",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GeoInternalServiceApi {
    GetCities,
    GetStations,
    DbGetCities,
    DbGetStations,
    SaveCityProvinceMap,
    SaveStationCityMap,
}

impl InternalApi for GeoInternalServiceApi {
    fn name(&self) -> &'static str {
        match self {
            GeoInternalServiceApi::GetCities => "get_cities",
            GeoInternalServiceApi::GetStations => "get_stations",
            GeoInternalServiceApi::DbGetCities => "db_get_cities",
            GeoInternalServiceApi::DbGetStations => "db_get_stations",
            GeoInternalServiceApi::SaveCityProvinceMap => "save_city_province_map",
            GeoInternalServiceApi::SaveStationCityMap => "save_station_city_map",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TrainInternalServiceApi {
    GetTrainByNumber,
    GetTrainScheduleByTrainIdAndOriginDepartureTime,
    GetTerminalArrivalTime,
    GetTrains,
    VerifyTrainNumber,
    DbGetTrains,
    DbGetRoutes,
    DbGetTrainSchedules,
    DbGetSeatTypes,
    DbGetSeatTypeMappings,
}

impl InternalApi for TrainInternalServiceApi {
    fn name(&self) -> &'static str {
        match self {
            TrainInternalServiceApi::GetTrainByNumber => "get_train_by_number",
            TrainInternalServiceApi::GetTrainScheduleByTrainIdAndOriginDepartureTime => {
                "get_train_schedule_by_train_id_and_origin_departure_time"
            }
            TrainInternalServiceApi::GetTerminalArrivalTime => "get_terminal_arrival_time",
            TrainInternalServiceApi::GetTrains => "get_trains",
            TrainInternalServiceApi::VerifyTrainNumber => "verify_train_number",
            TrainInternalServiceApi::DbGetTrains => "db_get_trains",
            TrainInternalServiceApi::DbGetRoutes => "db_get_routes",
            TrainInternalServiceApi::DbGetTrainSchedules => "db_get_train_schedules",
            TrainInternalServiceApi::DbGetSeatTypes => "db_get_seat_types",
            TrainInternalServiceApi::DbGetSeatTypeMappings => "db_get_seat_type_mappings",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DishInternalServiceApi {
    SaveRawDish,
    SaveRawTakeaway,
}

impl InternalApi for DishInternalServiceApi {
    fn name(&self) -> &'static str {
        match self {
            DishInternalServiceApi::SaveRawDish => "save_raw_dish",
            DishInternalServiceApi::SaveRawTakeaway => "save_raw_takeaway",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OrderInternalServiceApi {
    NewTransaction,
    RefundTransaction,
    GetOrderByUuid,
    ConvertOrderToDto,
    VerifyTrainOrder,
    UpdateOrders,
    GetOrderListByUserId,
}

impl InternalApi for OrderInternalServiceApi {
    fn name(&self) -> &'static str {
        match self {
            OrderInternalServiceApi::NewTransaction => "new_transaction",
            OrderInternalServiceApi::RefundTransaction => "refund_transaction",
            OrderInternalServiceApi::GetOrderByUuid => "get_order_by_uuid",
            OrderInternalServiceApi::ConvertOrderToDto => "convert_order_to_dto",
            OrderInternalServiceApi::VerifyTrainOrder => "verify_train_order",
            OrderInternalServiceApi::UpdateOrders => "update_orders",
            OrderInternalServiceApi::GetOrderListByUserId => "get_order_list_by_user_id",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObjectStorageServiceInternalServiceApi {
    PutObject,
    GetObject,
    DeleteObject,
}

impl InternalApi for ObjectStorageServiceInternalServiceApi {
    fn name(&self) -> &'static str {
        match self {
            ObjectStorageServiceInternalServiceApi::PutObject => "put_object",
            ObjectStorageServiceInternalServiceApi::GetObject => "get_object",
            ObjectStorageServiceInternalServiceApi::DeleteObject => "delete_object",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ApiEndpoint {
    pub host: String,
    pub port: u16,
}

impl Display for ApiEndpoint {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.host, self.port)
    }
}

#[derive(Debug, Error)]
pub enum InternalApiError {
    #[error("network error: {0}")]
    NetworkError(reqwest::Error),
    #[error("invalid json: {0}")]
    InvalidJson(serde_json::Error),
    #[error("API error: {status}, {message}")]
    ApiError { status: u16, message: String },
}

impl From<reqwest::Error> for InternalApiError {
    fn from(value: reqwest::Error) -> Self {
        InternalApiError::NetworkError(value)
    }
}

impl From<serde_json::Error> for InternalApiError {
    fn from(value: serde_json::Error) -> Self {
        InternalApiError::InvalidJson(value)
    }
}

pub struct SuperClient {
    client: Client,
    api_endpoint: ApiEndpoint,
}
impl SuperClient {
    pub fn new(api_endpoint: ApiEndpoint) -> Self {
        let client = Client::default();

        Self {
            client,
            api_endpoint,
        }
    }
}

impl SuperClient {
    fn get_url_for_api(&self, api: impl InternalApi) -> String {
        format!("http://{}{}", self.api_endpoint, api.path())
    }

    pub async fn get<Return>(&self, api: impl InternalApi) -> Result<Return, InternalApiError>
    where
        Return: Serialize + DeserializeOwned + Default,
    {
        let response = self.client.get(self.get_url_for_api(api)).send().await?;

        let payload: ApiResponse<Return> = response.json().await?;

        if payload.code != API_SUCCESS_CODE {
            return Err(InternalApiError::ApiError {
                status: payload.code as u16,
                message: payload.message,
            });
        }

        Ok(payload.data.unwrap())
    }

    pub async fn post<Data, Return>(
        &self,
        api: impl InternalApi,
        data: Data,
    ) -> Result<Return, InternalApiError>
    where
        Data: Serialize + DeserializeOwned,
        Return: Serialize + DeserializeOwned,
    {
        let serialized = serde_json::to_vec(&data).unwrap();

        let response = self
            .client
            .post(self.get_url_for_api(api))
            .body(serialized)
            .send()
            .await?;

        let payload: ApiResponse<Return> = response.json().await?;

        if payload.code != API_SUCCESS_CODE {
            return Err(InternalApiError::ApiError {
                status: payload.code as u16,
                message: payload.message,
            });
        }

        Ok(payload.data.unwrap())
    }
}
