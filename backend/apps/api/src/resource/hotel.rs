use crate::resource::get_object;
use actix_web::web::Data;
use actix_web::{HttpResponse, get, web};
use base::domain::service::object_storage::{ObjectCategory, ObjectStorageService};

#[get("/images/{uuid}")]
async fn get_hotel_image(
    path: web::Path<String>,
    object_storage_service: Data<dyn ObjectStorageService>,
) -> HttpResponse {
    let uuid = path.into_inner();

    get_object(object_storage_service, ObjectCategory::Hotel, &uuid).await
}
