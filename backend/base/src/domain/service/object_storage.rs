use async_trait::async_trait;
use std::fmt::{Display, Formatter};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum ObjectStorageServiceError {
    #[error("storage service error: {0}")]
    StorageServiceError(anyhow::Error),
    #[error("object not found: {0} in bucket: {1}")]
    ObjectNotFound(Uuid, &'static str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObjectCategory {
    Hotel,
    Dish,
    Takeaway,
}

impl From<&ObjectCategory> for &'static str {
    fn from(category: &ObjectCategory) -> Self {
        match category {
            ObjectCategory::Hotel => "hotel",
            ObjectCategory::Dish => "dish",
            ObjectCategory::Takeaway => "takeaway",
        }
    }
}

impl Display for ObjectCategory {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", <&ObjectCategory as Into<&'static str>>::into(self))
    }
}

impl ObjectCategory {
    pub fn to_bucket_name(&self) -> &'static str {
        match self {
            ObjectCategory::Hotel => "super-hotel",
            ObjectCategory::Dish => "super-dish",
            ObjectCategory::Takeaway => "super-takeaway",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObjectInfo {
    pub content_type: String,
    pub data: Vec<u8>,
}

#[async_trait]
pub trait ObjectStorageService: 'static + Send + Sync {
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
