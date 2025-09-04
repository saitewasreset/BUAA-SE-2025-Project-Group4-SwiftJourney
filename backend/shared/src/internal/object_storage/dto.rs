use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct ObjectInfo {
    pub content_type: String,
    pub data: Vec<u8>,
}
