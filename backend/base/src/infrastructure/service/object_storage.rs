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

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;
    use mockall::{automock, predicate::*};
    use uuid::Uuid;

    // --------- 模拟 S3 Client ---------
    #[automock]
    pub trait S3ClientTrait {
        fn create_bucket(&self, bucket: &str) -> Result<(), anyhow::Error>;
        fn put_object(
            &self,
            bucket: &str,
            key: &str,
            content_type: &str,
            body: Vec<u8>,
        ) -> Result<(), anyhow::Error>;
        fn get_object(&self, bucket: &str, key: &str) -> Result<(String, Vec<u8>), anyhow::Error>;
        fn delete_object(&self, bucket: &str, key: &str) -> Result<(), anyhow::Error>;
    }

    // 替代实现：用 MockS3Client 替换 aws_sdk_s3::Client
    pub struct TestS3ObjectStorageServiceImpl {
        client: MockS3ClientTrait,
    }

    impl TestS3ObjectStorageServiceImpl {
        pub fn new(client: MockS3ClientTrait) -> Self {
            Self { client }
        }

        async fn create_bucket_allow_exists(
            &self,
            bucket_name: &str,
        ) -> Result<(), ObjectStorageServiceError> {
            self.client
                .create_bucket(bucket_name)
                .map_err(ObjectStorageServiceError::StorageServiceError)?;
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
                .put_object(
                    object_category.to_bucket_name(),
                    &object_uuid.to_string(),
                    content_type,
                    object,
                )
                .map_err(ObjectStorageServiceError::StorageServiceError)?;
            Ok(object_uuid)
        }

        async fn get_object(
            &self,
            object_category: ObjectCategory,
            object_id: Uuid,
        ) -> Result<ObjectInfo, ObjectStorageServiceError> {
            match self
                .client
                .get_object(object_category.to_bucket_name(), &object_id.to_string())
            {
                Ok((content_type, data)) => Ok(ObjectInfo { content_type, data }),
                Err(_) => Err(ObjectStorageServiceError::ObjectNotFound(
                    object_id,
                    object_category.to_bucket_name(),
                )),
            }
        }

        async fn delete_object(
            &self,
            object_category: ObjectCategory,
            object_id: Uuid,
        ) -> Result<(), ObjectStorageServiceError> {
            self.client
                .delete_object(object_category.to_bucket_name(), &object_id.to_string())
                .map_err(ObjectStorageServiceError::StorageServiceError)?;
            Ok(())
        }
    }

    // ---------------- create_bucket_allow_exists ----------------
    #[tokio::test]
    async fn test_create_bucket_success() {
        let mut mock = MockS3ClientTrait::new();
        mock.expect_create_bucket()
            .with(eq("hotel"))
            .returning(|_| Ok(()));

        let service = TestS3ObjectStorageServiceImpl::new(mock);
        let res = service.create_bucket_allow_exists("hotel").await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_create_bucket_fail() {
        let mut mock = MockS3ClientTrait::new();
        mock.expect_create_bucket()
            .with(eq("hotel"))
            .returning(|_| Err(anyhow!("bucket error")));

        let service = TestS3ObjectStorageServiceImpl::new(mock);
        let res = service.create_bucket_allow_exists("hotel").await;
        assert!(matches!(
            res,
            Err(ObjectStorageServiceError::StorageServiceError(_))
        ));
    }

    // ---------------- put_object ----------------
    #[tokio::test]
    async fn test_put_object_success() {
        let mut mock = MockS3ClientTrait::new();
        mock.expect_put_object().returning(|_, _, _, _| Ok(()));

        let service = TestS3ObjectStorageServiceImpl::new(mock);
        let res = service
            .put_object(ObjectCategory::Hotel, "image/png", vec![1, 2, 3])
            .await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_put_object_fail() {
        let mut mock = MockS3ClientTrait::new();
        mock.expect_put_object()
            .returning(|_, _, _, _| Err(anyhow!("put error")));

        let service = TestS3ObjectStorageServiceImpl::new(mock);
        let res = service
            .put_object(ObjectCategory::Hotel, "image/png", vec![1, 2, 3])
            .await;
        assert!(matches!(
            res,
            Err(ObjectStorageServiceError::StorageServiceError(_))
        ));
    }

    // ---------------- get_object ----------------
    #[tokio::test]
    async fn test_get_object_success() {
        let mut mock = MockS3ClientTrait::new();
        mock.expect_get_object()
            .returning(|_, _| Ok(("image/png".to_string(), vec![1, 2, 3])));

        let service = TestS3ObjectStorageServiceImpl::new(mock);
        let object_id = Uuid::new_v4();
        let res = service
            .get_object(ObjectCategory::Hotel, object_id)
            .await
            .unwrap();
        assert_eq!(res.content_type, "image/png");
        assert_eq!(res.data, vec![1, 2, 3]);
    }

    #[tokio::test]
    async fn test_get_object_fail() {
        let mut mock = MockS3ClientTrait::new();
        mock.expect_get_object()
            .returning(|_, _| Err(anyhow!("no key")));

        let service = TestS3ObjectStorageServiceImpl::new(mock);
        let object_id = Uuid::new_v4();
        let res = service.get_object(ObjectCategory::Hotel, object_id).await;
        assert!(matches!(
            res,
            Err(ObjectStorageServiceError::ObjectNotFound(_, _))
        ));
    }

    // ---------------- delete_object ----------------
    #[tokio::test]
    async fn test_delete_object_success() {
        let mut mock = MockS3ClientTrait::new();
        mock.expect_delete_object().returning(|_, _| Ok(()));

        let service = TestS3ObjectStorageServiceImpl::new(mock);
        let res = service
            .delete_object(ObjectCategory::Hotel, Uuid::new_v4())
            .await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_delete_object_fail() {
        let mut mock = MockS3ClientTrait::new();
        mock.expect_delete_object()
            .returning(|_, _| Err(anyhow!("delete error")));

        let service = TestS3ObjectStorageServiceImpl::new(mock);
        let res = service
            .delete_object(ObjectCategory::Hotel, Uuid::new_v4())
            .await;
        assert!(matches!(
            res,
            Err(ObjectStorageServiceError::StorageServiceError(_))
        ));
    }
}
