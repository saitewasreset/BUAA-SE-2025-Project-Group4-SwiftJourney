use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    api::InternalApiError,
    internal::object_storage::{
        command::{DeleteObjectCommand, ObjectQuery, PutObjectCommand},
        dto::ObjectInfo,
    },
};

#[async_trait]
pub trait ObjectStoragePort: 'static + Send + Sync {
    async fn put_object(&self, command: PutObjectCommand) -> Result<Uuid, InternalApiError>;
    async fn get_object(&self, query: ObjectQuery) -> Result<ObjectInfo, InternalApiError>;
    async fn delete_object(&self, command: DeleteObjectCommand) -> Result<(), InternalApiError>;
}
