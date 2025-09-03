use actix_web::web::Bytes;
use actix_web::{post, web};
use object_storage_base::application::service::internal::ObjectStorageInternalService;
use shared::api::{ApiResponse, ApplicationErrorBox, parse_request_body};
use shared::application_error::ApplicationError;
use shared::internal::object_storage::command::{
    DeleteObjectCommand, ObjectQuery, PutObjectCommand,
};
use shared::internal::object_storage::dto::ObjectInfo;
use uuid::Uuid;

#[post("/put_object")]
pub async fn put_object(
    body: Bytes,
    object_storage_internal_service: web::Data<dyn ObjectStorageInternalService>,
) -> Result<ApiResponse<Uuid>, ApplicationErrorBox> {
    let command: PutObjectCommand = parse_request_body(body)?;

    let object_id = object_storage_internal_service
        .put_object(command)
        .await
        .map_err(|e| Box::new(e) as Box<dyn ApplicationError>)?;

    ApiResponse::ok(object_id)
}

#[post("/get_object")]
pub async fn get_object(
    body: Bytes,
    object_storage_internal_service: web::Data<dyn ObjectStorageInternalService>,
) -> Result<ApiResponse<ObjectInfo>, ApplicationErrorBox> {
    let query: ObjectQuery = parse_request_body(body)?;

    let object = object_storage_internal_service
        .get_object(query)
        .await
        .map_err(|e| Box::new(e) as Box<dyn ApplicationError>)?;

    ApiResponse::ok(object)
}

#[post("/delete_object")]
pub async fn delete_object(
    body: Bytes,
    object_storage_internal_service: web::Data<dyn ObjectStorageInternalService>,
) -> Result<ApiResponse<()>, ApplicationErrorBox> {
    let command: DeleteObjectCommand = parse_request_body(body)?;

    object_storage_internal_service
        .delete_object(command)
        .await
        .map_err(|e| Box::new(e) as Box<dyn ApplicationError>)?;

    ApiResponse::ok(())
}

pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(put_object)
        .service(get_object)
        .service(delete_object);
}
