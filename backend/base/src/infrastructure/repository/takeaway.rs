use crate::domain::DbId;
use crate::domain::model::takeaway::TakeawayDishId;

impl_db_id_from_u64!(TakeawayDishId, i32, "takeaway dish id");
