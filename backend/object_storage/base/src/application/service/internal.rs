use async_trait::async_trait;
use shared::application_error::ApplicationError;
use shared::internal::object_storage::command::{
    DeleteObjectCommand, ObjectQuery, PutObjectCommand,
};
use shared::internal::object_storage::dto::{ObjectCategory, ObjectInfo};
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum ObjectStorageInternalServiceError {
    #[error("object not found: {0} in category: {1}")]
    NotFound(Uuid, ObjectCategory),
    #[error("storage unavailable: {0}")]
    StorageUnavailable(String),
    #[error(transparent)]
    RelatedServiceError(#[from] anyhow::Error),
}

impl ApplicationError for ObjectStorageInternalServiceError {
    fn error_code(&self) -> u32 {
        match self {
            Self::NotFound(_, _) => 96003,
            Self::StorageUnavailable(_) => 96004,
            Self::RelatedServiceError(_) => 96001,
        }
    }

    fn error_message(&self) -> String {
        self.to_string()
    }
}

#[async_trait]
pub trait ObjectStorageInternalService: 'static + Send + Sync {
    async fn put_object(
        &self,
        command: PutObjectCommand,
    ) -> Result<Uuid, ObjectStorageInternalServiceError>;

    async fn get_object(
        &self,
        query: ObjectQuery,
    ) -> Result<ObjectInfo, ObjectStorageInternalServiceError>;

    async fn delete_object(
        &self,
        command: DeleteObjectCommand,
    ) -> Result<(), ObjectStorageInternalServiceError>;
}
