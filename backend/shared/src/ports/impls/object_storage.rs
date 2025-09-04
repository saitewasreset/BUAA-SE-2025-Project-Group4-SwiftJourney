use async_trait::async_trait;
use tracing::error;
use uuid::Uuid;

use crate::api::ObjectStorageServiceInternalServiceApi;
use crate::{
    api::{ApiEndpoint, InternalApiError, SuperClient},
    internal::object_storage::{
        command::{DeleteObjectCommand, ObjectQuery, PutObjectCommand},
        dto::ObjectInfo,
    },
    ports::object_storage::ObjectStoragePort,
};

pub struct HttpObjectStoragePortImpl {
    super_client: SuperClient,
}

impl HttpObjectStoragePortImpl {
    pub fn new(api_endpoint: ApiEndpoint) -> Self {
        let super_client = SuperClient::new(api_endpoint);

        Self { super_client }
    }
}

#[async_trait]
impl ObjectStoragePort for HttpObjectStoragePortImpl {
    async fn put_object(&self, command: PutObjectCommand) -> Result<Uuid, InternalApiError> {
        self.super_client
            .post(ObjectStorageServiceInternalServiceApi::PutObject, command)
            .await
            .inspect_err(|e| error!("Failed to get city station info: {:?}", e))
    }
    async fn get_object(&self, query: ObjectQuery) -> Result<ObjectInfo, InternalApiError> {
        self.super_client
            .post(ObjectStorageServiceInternalServiceApi::GetObject, query)
            .await
            .inspect_err(|e| error!("Failed to get city station info: {:?}", e))
    }
    async fn delete_object(&self, command: DeleteObjectCommand) -> Result<(), InternalApiError> {
        self.super_client
            .post(
                ObjectStorageServiceInternalServiceApi::DeleteObject,
                command,
            )
            .await
    }
}
