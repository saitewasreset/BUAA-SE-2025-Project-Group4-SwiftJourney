//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.3

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "dish")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub train_id: i32,
    pub r#type: String,
    pub time: String,
    pub name: String,
    #[sea_orm(column_type = "Decimal(Some((10, 2)))")]
    pub price: Decimal,
    pub images: Json,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::dish_order::Entity")]
    DishOrder,
    #[sea_orm(
        belongs_to = "super::train::Entity",
        from = "Column::TrainId",
        to = "super::train::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Train,
}

impl Related<super::dish_order::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DishOrder.def()
    }
}

impl Related<super::train::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Train.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
