use crate::domain::DbId;
use crate::domain::model::route::RouteId;

impl_db_id_from_u64!(RouteId, i64, "route");
