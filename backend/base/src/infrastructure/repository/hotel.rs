use crate::domain::DbId;
use crate::domain::model::hotel::{HotelId, HotelRoomTypeId};

impl_db_id_from_u64!(HotelId, i32, "hotel id");
impl_db_id_from_u64!(HotelRoomTypeId, i32, "hotel room type id");
