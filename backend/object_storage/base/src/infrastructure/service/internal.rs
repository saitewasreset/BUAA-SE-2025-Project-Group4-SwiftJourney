use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::application::service::internal::{
    ObjectStorageInternalService, ObjectStorageInternalServiceError,
};
use crate::domain::service::object_storage as domain;
use crate::domain::service::object_storage::{ObjectStorageService, ObjectStorageServiceError};
use shared::internal::object_storage::command::{
    DeleteObjectCommand, ObjectQuery, PutObjectCommand,
};
use shared::internal::object_storage::dto::{ObjectCategory, ObjectInfo};

pub struct ObjectStorageInternalServiceImpl<S: ObjectStorageService> {
    inner: Arc<S>,
}

impl<S: ObjectStorageService> ObjectStorageInternalServiceImpl<S> {
    pub fn new(inner: Arc<S>) -> Self {
        Self { inner }
    }
}

#[async_trait]
impl<S: ObjectStorageService> ObjectStorageInternalService for ObjectStorageInternalServiceImpl<S> {
    async fn put_object(
        &self,
        command: PutObjectCommand,
    ) -> Result<Uuid, ObjectStorageInternalServiceError> {
        let category = match command.object_category {
            ObjectCategory::Hotel => domain::ObjectCategory::Hotel,
            ObjectCategory::Dish => domain::ObjectCategory::Dish,
            ObjectCategory::Takeaway => domain::ObjectCategory::Takeaway,
        };

        self.inner
            .put_object(category, &command.content_type, command.object)
            .await
            .map_err(|e| match e {
                ObjectStorageServiceError::ObjectNotFound(id, _bucket) => {
                    ObjectStorageInternalServiceError::NotFound(id, command.object_category)
                }
                ObjectStorageServiceError::StorageServiceError(err) => {
                    ObjectStorageInternalServiceError::RelatedServiceError(err)
                }
            })
    }

    async fn get_object(
        &self,
        query: ObjectQuery,
    ) -> Result<ObjectInfo, ObjectStorageInternalServiceError> {
        let category = match query.object_category {
            ObjectCategory::Hotel => domain::ObjectCategory::Hotel,
            ObjectCategory::Dish => domain::ObjectCategory::Dish,
            ObjectCategory::Takeaway => domain::ObjectCategory::Takeaway,
        };

        self.inner
            .get_object(category, query.object_id)
            .await
            .map(|obj| ObjectInfo {
                content_type: obj.content_type,
                data: obj.data,
            })
            .map_err(|e| match e {
                ObjectStorageServiceError::ObjectNotFound(id, _bucket) => {
                    ObjectStorageInternalServiceError::NotFound(id, query.object_category)
                }
                ObjectStorageServiceError::StorageServiceError(err) => {
                    ObjectStorageInternalServiceError::RelatedServiceError(err)
                }
            })
    }

    async fn delete_object(
        &self,
        command: DeleteObjectCommand,
    ) -> Result<(), ObjectStorageInternalServiceError> {
        let category = match command.object_category {
            ObjectCategory::Hotel => domain::ObjectCategory::Hotel,
            ObjectCategory::Dish => domain::ObjectCategory::Dish,
            ObjectCategory::Takeaway => domain::ObjectCategory::Takeaway,
        };

        self.inner
            .delete_object(category, command.object_id)
            .await
            .map_err(|e| match e {
                ObjectStorageServiceError::ObjectNotFound(id, _bucket) => {
                    ObjectStorageInternalServiceError::NotFound(id, command.object_category)
                }
                ObjectStorageServiceError::StorageServiceError(err) => {
                    ObjectStorageInternalServiceError::RelatedServiceError(err)
                }
            })
    }
}
