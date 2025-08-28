#![cfg(test)]

use async_trait::async_trait;
use mockall::mock;
use uuid::Uuid;
use crate::domain::service::object_storage::{ObjectCategory, ObjectInfo, ObjectStorageService, ObjectStorageServiceError};

mock! {
    pub ObjectStorageService {}

    #[async_trait]
    impl ObjectStorageService for ObjectStorageService {
        async fn init_buckets(&self) -> Result<(), ObjectStorageServiceError>;

        async fn put_object(
            &self,
            object_category: ObjectCategory,
            content_type: &str,
            object: Vec<u8>,
        ) -> Result<Uuid, ObjectStorageServiceError>;

        async fn get_object(
            &self,
            object_category: ObjectCategory,
            object_id: Uuid,
        ) -> Result<ObjectInfo, ObjectStorageServiceError>;

        async fn delete_object(
            &self,
            object_category: ObjectCategory,
            object_id: Uuid,
        ) -> Result<(), ObjectStorageServiceError>;
    }
}

pub fn mock_object_storage_service() -> MockObjectStorageService {
    MockObjectStorageService::new()
}