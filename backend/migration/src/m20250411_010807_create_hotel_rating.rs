use crate::m20250411_010715_create_user::User;
use crate::m20250411_010744_create_hotel::Hotel;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum HotelRating {
    Table,
    Id,
    UserId,
    HotelId,
    Time,
    Rating,
    Text,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(HotelRating::Table)
                    .if_not_exists()
                    .col(pk_auto(HotelRating::Id))
                    .col(big_integer(HotelRating::UserId).not_null())
                    .col(big_integer(HotelRating::HotelId).not_null())
                    .col(timestamp_with_time_zone(HotelRating::Time).not_null())
                    .col(
                        decimal_len(HotelRating::Rating, 2, 1)
                            .not_null()
                            .check(Expr::col(HotelRating::Rating).between(1.0, 5.0)),
                    )
                    .col(string(HotelRating::Text))
                    .foreign_key(
                        ForeignKey::create()
                            .from(HotelRating::Table, HotelRating::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(HotelRating::Table, HotelRating::HotelId)
                            .to(Hotel::Table, Hotel::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(HotelRating::Table).to_owned())
            .await
    }
}
