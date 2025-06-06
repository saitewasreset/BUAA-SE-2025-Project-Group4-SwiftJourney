use actix_web::web::Data;
use actix_web::{HttpResponse, get, web};
use base::domain::service::object_storage::{
    ObjectCategory, ObjectStorageService, ObjectStorageServiceError,
};
use tracing::{error, instrument};
use uuid::Uuid;

#[instrument(skip(object_storage_service))]
pub async fn get_object(
    object_storage_service: Data<dyn ObjectStorageService>,
    object_category: ObjectCategory,
    object_id_str: &str,
) -> HttpResponse {
    if let Ok(uuid) = Uuid::try_parse(object_id_str) {
        match object_storage_service
            .get_object(object_category, uuid)
            .await
        {
            Ok(object) => HttpResponse::Ok()
                .content_type(object.content_type)
                .body(object.data),
            Err(e) => match e {
                ObjectStorageServiceError::ObjectNotFound(_, _) => {
                    HttpResponse::NotFound().finish()
                }
                x => {
                    error!("Failed to get {}: {}", object_category, x);
                    HttpResponse::InternalServerError().finish()
                }
            },
        }
    } else {
        HttpResponse::BadRequest().finish()
    }
}

#[get("/hotel/images/{uuid}")]
async fn get_hotel_image(
    path: web::Path<String>,
    object_storage_service: Data<dyn ObjectStorageService>,
) -> HttpResponse {
    let uuid = path.into_inner();

    get_object(object_storage_service, ObjectCategory::Hotel, &uuid).await
}

#[get("/dish/images/{uuid}")]
async fn get_dish_image(
    path: web::Path<String>,
    object_storage_service: Data<dyn ObjectStorageService>,
) -> HttpResponse {
    let uuid = path.into_inner();

    get_object(object_storage_service, ObjectCategory::Dish, &uuid).await
}

#[get("/takeaway/images/{uuid}")]
async fn get_takeaway_image(
    path: web::Path<String>,
    object_storage_service: Data<dyn ObjectStorageService>,
) -> HttpResponse {
    let uuid = path.into_inner();

    get_object(object_storage_service, ObjectCategory::Takeaway, &uuid).await
}

pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_hotel_image)
        .service(get_dish_image)
        .service(get_takeaway_image);
}
