use crate::{ApiResponse, AppConfig, ApplicationErrorBox};
use actix_web::{get, web};

#[get("/mode")]
async fn get_mode(
    app_config: web::Data<AppConfig>,
) -> Result<ApiResponse<String>, ApplicationErrorBox> {
    let mode = match app_config.debug {
        true => "debug",
        false => "release",
    };

    ApiResponse::ok(mode.to_string())
}
