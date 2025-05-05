use crate::domain::DbId;
use crate::domain::model::hotel::{HotelId, HotelRoomId};

impl_db_id_from_u64!(HotelId, i32, "hotel id");
impl_db_id_from_u64!(HotelRoomId, i32, "hotel room id");
