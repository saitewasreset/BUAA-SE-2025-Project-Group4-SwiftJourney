use crate::domain::model::station::StationId;
use crate::domain::{Aggregate, Entity, Identifiable, Identifier};
use id_macro::define_id_type;
use rust_decimal::Decimal;

define_id_type!(TakeawayShop);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TakeawayShop {
    id: Option<TakeawayShopId>,
    name: String,
    station_id: StationId,
    images: Vec<String>,
}

impl TakeawayShop {
    pub fn new(
        id: Option<TakeawayShopId>,
        name: String,
        station_id: StationId,
        images: Vec<String>,
    ) -> Self {
        Self {
            id,
            name,
            station_id,
            images,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn station_id(&self) -> StationId {
        self.station_id
    }

    pub fn images(&self) -> &[String] {
        &self.images
    }
}

impl Identifiable for TakeawayShop {
    type ID = TakeawayShopId;

    fn get_id(&self) -> Option<Self::ID> {
        self.id
    }

    fn set_id(&mut self, id: Self::ID) {
        self.id = Some(id);
    }
}

impl Entity for TakeawayShop {}
impl Aggregate for TakeawayShop {}

define_id_type!(TakeawayDish);
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TakeawayDish {
    id: Option<TakeawayDishId>,
    shop_id: TakeawayShopId,
    name: String,
    dish_type: String,
    unit_price: Decimal,
    images: Vec<String>,
}

impl TakeawayDish {
    pub fn new(
        id: Option<TakeawayDishId>,
        shop_id: TakeawayShopId,
        name: String,
        dish_type: String,
        unit_price: Decimal,
        images: Vec<String>,
    ) -> Self {
        Self {
            id,
            shop_id,
            name,
            dish_type,
            unit_price,
            images,
        }
    }

    pub fn shop_id(&self) -> TakeawayShopId {
        self.shop_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn dish_type(&self) -> &str {
        &self.dish_type
    }

    pub fn unit_price(&self) -> Decimal {
        self.unit_price
    }

    pub fn images(&self) -> &[String] {
        &self.images
    }
}

impl Identifiable for TakeawayDish {
    type ID = TakeawayDishId;

    fn get_id(&self) -> Option<Self::ID> {
        self.id
    }

    fn set_id(&mut self, id: Self::ID) {
        self.id = Some(id);
    }
}

impl Entity for TakeawayDish {}
impl Aggregate for TakeawayDish {}
