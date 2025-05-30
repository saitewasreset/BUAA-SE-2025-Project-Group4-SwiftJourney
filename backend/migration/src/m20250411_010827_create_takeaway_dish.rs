use crate::m20250411_010825_create_takeaway_shop::TakeawayShop;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum TakeawayDish {
    Table,
    Id,
    Name,
    DishType,
    Price,
    TakeawayShopId,
    Images,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TakeawayDish::Table)
                    .if_not_exists()
                    .col(pk_auto(TakeawayDish::Id))
                    .col(string(TakeawayDish::Name).not_null())
                    .col(string(TakeawayDish::DishType).not_null())
                    .col(
                        decimal_len(TakeawayDish::Price, 10, 2)
                            .not_null()
                            .check(Expr::col(TakeawayDish::Price).gte(0)),
                    )
                    .col(integer(TakeawayDish::TakeawayShopId).not_null())
                    .col(json(TakeawayDish::Images).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(TakeawayDish::Table, TakeawayDish::TakeawayShopId)
                            .to(TakeawayShop::Table, TakeawayShop::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TakeawayDish::Table).to_owned())
            .await
    }
}
