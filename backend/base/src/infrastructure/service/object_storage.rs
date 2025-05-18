use crate::domain::service::object_storage::{
    ObjectCategory, ObjectInfo, ObjectStorageService, ObjectStorageServiceError,
};
use anyhow::anyhow;
use async_trait::async_trait;
use aws_sdk_s3::error::SdkError;
use aws_sdk_s3::operation::create_bucket::CreateBucketError;
use aws_sdk_s3::operation::get_object::GetObjectError;
use uuid::Uuid;

pub struct S3ObjectStorageServiceImpl {
    client: aws_sdk_s3::Client,
}

impl S3ObjectStorageServiceImpl {
    pub fn new(endpoint_url: &str, access_key: &str, secret_key: &str) -> Self {
        let cred = aws_sdk_s3::config::Credentials::new(
            access_key,
            secret_key,
            None,
            None,
            "loaded-from-custom-env",
        );
        let s3_config = aws_sdk_s3::config::Builder::new()
            .endpoint_url(endpoint_url)
            .credentials_provider(cred)
            .force_path_style(true)
            .region(aws_sdk_s3::config::Region::new("super-earth"))
            .behavior_version_latest()
            .build();

        let client = aws_sdk_s3::Client::from_conf(s3_config);

        S3ObjectStorageServiceImpl { client }
    }
}

impl S3ObjectStorageServiceImpl {
    async fn create_bucket_allow_exists(
        &self,
        bucket_name: &str,
    ) -> Result<(), ObjectStorageServiceError> {
        match self.client.create_bucket().bucket(bucket_name).send().await {
            Ok(_) => Ok(()),
            Err(err) => match err {
                SdkError::ServiceError(e) => match e.err() {
                    CreateBucketError::BucketAlreadyExists(_) => Ok(()),
                    CreateBucketError::BucketAlreadyOwnedByYou(_) => Ok(()),
                    x => Err(ObjectStorageServiceError::StorageServiceError(anyhow!(
                        "service error: {}",
                        x
                    ))),
                },
                x => Err(ObjectStorageServiceError::StorageServiceError(anyhow!(
                    "sdk error: {}",
                    x
                ))),
            },
        }
    }
}

#[async_trait]
impl ObjectStorageService for S3ObjectStorageServiceImpl {
    async fn init_buckets(&self) -> Result<(), ObjectStorageServiceError> {
        let object_category_list = [
            ObjectCategory::Hotel,
            ObjectCategory::Dish,
            ObjectCategory::Takeaway,
        ];

        for object_category in object_category_list {
            self.create_bucket_allow_exists(object_category.to_bucket_name())
                .await?;
        }

        Ok(())
    }

    async fn put_object(
        &self,
        object_category: ObjectCategory,
        content_type: &str,
        object: Vec<u8>,
    ) -> Result<Uuid, ObjectStorageServiceError> {
        let object_uuid = Uuid::new_v4();

        self.client
            .put_object()
            .bucket(object_category.to_bucket_name())
            .key(object_uuid.to_string())
            .content_type(content_type)
            .body(object.into())
            .send()
            .await
            .map_err(|e| {
                ObjectStorageServiceError::StorageServiceError(anyhow!(
                    "failed to put object: {}",
                    e
                ))
            })?;

        Ok(object_uuid)
    }

    async fn get_object(
        &self,
        object_category: ObjectCategory,
        object_id: Uuid,
    ) -> Result<ObjectInfo, ObjectStorageServiceError> {
        match self
            .client
            .get_object()
            .bucket(object_category.to_bucket_name())
            .key(object_id.to_string())
            .send()
            .await
        {
            Ok(output) => {
                let content_type = output
                    .content_type
                    .unwrap_or("application/octet-stream".to_string());

                let body = output.body.collect().await.map_err(|e| {
                    ObjectStorageServiceError::StorageServiceError(anyhow!(
                        "failed to collect object body: {} for object uuid: {}, category: {}",
                        e,
                        object_id,
                        object_category
                    ))
                })?;

                let data = body.into_bytes().to_vec();

                Ok(ObjectInfo { content_type, data })
            }
            Err(sdk_err) => match sdk_err {
                SdkError::ServiceError(service_err) => match service_err.err() {
                    GetObjectError::NoSuchKey(_) => Err(ObjectStorageServiceError::ObjectNotFound(
                        object_id,
                        object_category.to_bucket_name(),
                    )),
                    x => Err(ObjectStorageServiceError::StorageServiceError(anyhow!(
                        "service error: {}",
                        x
                    ))),
                },
                x => Err(ObjectStorageServiceError::StorageServiceError(anyhow!(
                    "sdk error: {}",
                    x
                ))),
            },
        }
    }

    async fn delete_object(
        &self,
        object_category: ObjectCategory,
        object_id: Uuid,
    ) -> Result<(), ObjectStorageServiceError> {
        match self
            .client
            .delete_object()
            .bucket(object_category.to_bucket_name())
            .key(object_id.to_string())
            .send()
            .await
        {
            Ok(_for_super_earth) => Ok(()),
            Err(sdk_err) => match sdk_err {
                SdkError::ServiceError(service_err) => {
                    Err(ObjectStorageServiceError::StorageServiceError(anyhow!(
                        "service error: {}",
                        service_err.err()
                    )))
                }
                x => Err(ObjectStorageServiceError::StorageServiceError(anyhow!(
                    "sdk error: {}",
                    x
                ))),
            },
        }
    }
}
