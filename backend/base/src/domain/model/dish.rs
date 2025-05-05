use crate::domain::model::train::TrainId;
use crate::domain::{Aggregate, Entity, Identifiable, Identifier};
use id_macro::define_id_type;
use rust_decimal::Decimal;
use std::fmt::Formatter;
use thiserror::Error;

define_id_type!(Dish);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DishTime {
    Lunch,
    Dinner,
}

#[derive(Debug, Error)]
pub enum DishTimeError {
    #[error("Invalid dish time: {0}")]
    InvalidDishTime(String),
}

impl TryFrom<&str> for DishTime {
    type Error = DishTimeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "lunch" => Ok(DishTime::Lunch),
            "dinner" => Ok(DishTime::Dinner),
            x => Err(DishTimeError::InvalidDishTime(x.to_string())),
        }
    }
}

impl From<DishTime> for &str {
    fn from(value: DishTime) -> Self {
        match value {
            DishTime::Lunch => "lunch",
            DishTime::Dinner => "dinner",
        }
    }
}

impl From<&DishTime> for &str {
    fn from(value: &DishTime) -> Self {
        match &value {
            DishTime::Lunch => "lunch",
            DishTime::Dinner => "dinner",
        }
    }
}

impl std::fmt::Display for DishTime {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", <&DishTime as Into<&str>>::into(self))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Dish {
    id: Option<DishId>,
    train_id: TrainId,
    dish_type: String,
    dish_time: DishTime,
    name: String,
    unit_price: Decimal,
    images: Vec<String>,
}

impl Dish {
    pub fn new(
        id: Option<DishId>,
        train_id: TrainId,
        dish_type: String,
        dish_time: DishTime,
        name: String,
        unit_price: Decimal,
        images: Vec<String>,
    ) -> Self {
        Self {
            id,
            train_id,
            dish_type,
            dish_time,
            name,
            unit_price,
            images,
        }
    }

    pub fn train_id(&self) -> TrainId {
        self.train_id
    }

    pub fn dish_type(&self) -> &str {
        &self.dish_type
    }

    pub fn dish_time(&self) -> DishTime {
        self.dish_time
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn unit_price(&self) -> Decimal {
        self.unit_price
    }

    pub fn images(&self) -> &[String] {
        &self.images
    }
}

impl Identifiable for Dish {
    type ID = DishId;

    fn get_id(&self) -> Option<Self::ID> {
        self.id
    }

    fn set_id(&mut self, id: Self::ID) {
        self.id = Some(id)
    }
}

impl Entity for Dish {}
impl Aggregate for Dish {}
