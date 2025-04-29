use crate::domain::model::city::CityId;
use crate::domain::{Aggregate, Entity, Identifiable, Identifier};
use id_macro::define_id_type;

define_id_type!(Station);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Station {
    station_id: Option<StationId>,
    name: String,
    city_id: CityId,
}

impl Identifiable for Station {
    type ID = StationId;

    fn get_id(&self) -> Option<Self::ID> {
        self.station_id
    }

    fn set_id(&mut self, id: Self::ID) {
        self.station_id = Some(id);
    }
}

impl Entity for Station {}
impl Aggregate for Station {}

impl Station {
    pub fn new(station_id: Option<StationId>, name: String, city_id: CityId) -> Self {
        Station {
            station_id,
            name,
            city_id,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn city_id(&self) -> CityId {
        self.city_id
    }
}
