use crate::m20250411_010744_create_hotel::Hotel;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum HotelRoomType {
    Table,
    Id,
    TypeName,
    Capacity,
    Price,
    HotelId,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(HotelRoomType::Table)
                    .if_not_exists()
                    .col(pk_auto(HotelRoomType::Id))
                    .col(string(HotelRoomType::TypeName).not_null())
                    .col(
                        integer(HotelRoomType::Capacity)
                            .not_null()
                            .check(Expr::col(HotelRoomType::Capacity).gt(0)),
                    )
                    .col(
                        decimal_len(HotelRoomType::Price, 10, 2)
                            .not_null()
                            .check(Expr::col(HotelRoomType::Price).gte(0)),
                    )
                    .col(big_integer(HotelRoomType::HotelId).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(HotelRoomType::Table, HotelRoomType::HotelId)
                            .to(Hotel::Table, Hotel::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(HotelRoomType::Table).to_owned())
            .await
    }
}
