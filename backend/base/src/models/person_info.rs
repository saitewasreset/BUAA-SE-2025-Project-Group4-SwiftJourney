//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.3

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "person_info")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub uuid: Uuid,
    pub name: String,
    pub identity_card: String,
    pub preferred_seat_location: Option<String>,
    pub user_id: i32,
    pub is_default: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::dish_order::Entity")]
    DishOrder,
    #[sea_orm(has_many = "super::hotel_order::Entity")]
    HotelOrder,
    #[sea_orm(has_many = "super::occupied_room::Entity")]
    OccupiedRoom,
    #[sea_orm(has_many = "super::occupied_seat::Entity")]
    OccupiedSeat,
    #[sea_orm(has_many = "super::takeaway_order::Entity")]
    TakeawayOrder,
    #[sea_orm(has_many = "super::train_order::Entity")]
    TrainOrder,
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    User,
}

impl Related<super::dish_order::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DishOrder.def()
    }
}

impl Related<super::hotel_order::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::HotelOrder.def()
    }
}

impl Related<super::occupied_room::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::OccupiedRoom.def()
    }
}

impl Related<super::occupied_seat::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::OccupiedSeat.def()
    }
}

impl Related<super::takeaway_order::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TakeawayOrder.def()
    }
}

impl Related<super::train_order::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TrainOrder.def()
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
