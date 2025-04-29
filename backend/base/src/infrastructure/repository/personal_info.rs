use crate::domain::DbId;
use crate::domain::model::personal_info::PersonalInfoId;

impl_db_id_from_u64!(PersonalInfoId, i32, "personal info");
