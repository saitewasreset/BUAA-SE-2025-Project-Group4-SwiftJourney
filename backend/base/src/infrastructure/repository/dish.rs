use crate::domain::DbId;
use crate::domain::model::dish::DishId;

impl_db_id_from_u64!(DishId, i32, "dish id");
