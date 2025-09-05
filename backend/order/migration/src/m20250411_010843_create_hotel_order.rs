use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum HotelOrder {
    Table,
    Id,
    Uuid,
    HotelId,
    BeginDate,
    EndDate,
    HotelRoomTypeId,
    PersonInfoId,
    PayTransactionId,
    RefundTransactionId,
    Price,
    Amount,
    CreateTime,
    ActiveTime,
    CompleteTime,
    Status,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(HotelOrder::Table)
                    .if_not_exists()
                    .col(pk_auto(HotelOrder::Id))
                    .col(uuid(HotelOrder::Uuid).not_null())
                    .col(integer(HotelOrder::HotelId).not_null())
                    .col(date(HotelOrder::BeginDate).not_null())
                    .col(date(HotelOrder::EndDate).not_null().check(
                        Expr::col(HotelOrder::EndDate).gte(Expr::col(HotelOrder::BeginDate)),
                    ))
                    .col(integer(HotelOrder::HotelRoomTypeId).not_null())
                    .col(integer(HotelOrder::PersonInfoId).not_null())
                    .col(
                        ColumnDef::new(HotelOrder::PayTransactionId)
                            .integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(HotelOrder::RefundTransactionId)
                            .integer()
                            .null(),
                    )
                    .col(
                        decimal_len(HotelOrder::Price, 10, 2)
                            .not_null()
                            .check(Expr::col(HotelOrder::Price).gte(0)),
                    )
                    .col(
                        integer(HotelOrder::Amount)
                            .not_null()
                            .check(Expr::col(HotelOrder::Amount).gt(0)),
                    )
                    .col(timestamp_with_time_zone(HotelOrder::CreateTime).not_null())
                    .col(timestamp_with_time_zone(HotelOrder::ActiveTime))
                    .col(timestamp_with_time_zone(HotelOrder::CompleteTime))
                    .col(string(HotelOrder::Status).not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(HotelOrder::Table).to_owned())
            .await
    }
}
