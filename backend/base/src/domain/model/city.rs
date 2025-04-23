use crate::domain::{Entity, Identifiable, Identifier};
use id_macro::define_id_type;
use std::ops::Deref;

define_id_type!(City);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CityName(String);

impl From<String> for CityName {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl Deref for CityName {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProvinceName(String);

impl Deref for ProvinceName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<String> for ProvinceName {
    fn from(value: String) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct City {
    city_id: Option<CityId>,
    name: CityName,
    province: ProvinceName,
}

impl Identifiable for City {
    type ID = CityId;
    fn get_id(&self) -> Option<Self::ID> {
        self.city_id
    }

    fn set_id(&mut self, id: Self::ID) {
        self.city_id = Some(id)
    }
}

impl Entity for City {}

impl City {
    pub fn new(name: CityName, province: ProvinceName) -> Self {
        City {
            city_id: None,
            name,
            province,
        }
    }

    pub fn name(&self) -> &CityName {
        &self.name
    }

    pub fn province(&self) -> &ProvinceName {
        &self.province
    }
}
