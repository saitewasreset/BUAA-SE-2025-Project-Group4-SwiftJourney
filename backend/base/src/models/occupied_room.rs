//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.3

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "occupied_room")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub hotel_id: i32,
    pub room_type_id: i32,
    pub begin_date: Date,
    pub end_date: Date,
    pub person_info_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::hotel::Entity",
        from = "Column::HotelId",
        to = "super::hotel::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Hotel,
    #[sea_orm(
        belongs_to = "super::hotel_room_type::Entity",
        from = "Column::RoomTypeId",
        to = "super::hotel_room_type::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    HotelRoomType,
    #[sea_orm(
        belongs_to = "super::person_info::Entity",
        from = "Column::PersonInfoId",
        to = "super::person_info::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    PersonInfo,
}

impl Related<super::hotel::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Hotel.def()
    }
}

impl Related<super::hotel_room_type::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::HotelRoomType.def()
    }
}

impl Related<super::person_info::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PersonInfo.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
