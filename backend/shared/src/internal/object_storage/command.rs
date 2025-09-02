use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::internal::object_storage::dto::ObjectCategory;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PutObjectCommand {
    pub object_category: ObjectCategory,
    pub content_type: String,
    pub object: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjectQuery {
    pub object_category: ObjectCategory,
    pub object_id: Uuid,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeleteObjectCommand {
    pub object_category: ObjectCategory,
    pub object_id: Uuid,
}
