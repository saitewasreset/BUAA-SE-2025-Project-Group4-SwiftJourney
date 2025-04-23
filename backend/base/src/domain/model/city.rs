use crate::domain::{Entity, Identifiable, Identifier};
use id_macro::define_id_type;

define_id_type!(City);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct City {
    city_id: Option<CityId>,
    name: String,
    province: String,
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
    pub fn new(name: String, province: String) -> Self {
        City {
            city_id: None,
            name,
            province,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn province(&self) -> &str {
        &self.province
    }
}
