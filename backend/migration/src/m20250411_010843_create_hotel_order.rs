use crate::m20250411_010719_create_person_info::PersonInfo;
use crate::m20250411_010725_create_transaction::Transaction;
use crate::m20250411_010744_create_hotel::Hotel;
use crate::m20250411_010751_create_hotel_room_type::HotelRoomType;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum HotelOrder {
    Table,
    Id,
    HotelId,
    BeginDate,
    EndDate,
    HotelRoomTypeId,
    PersonInfoId,
    PayTransactionId,
    RefundTransactionId,
    Price,
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
                    .col(big_integer(HotelOrder::HotelId).not_null())
                    .col(date(HotelOrder::BeginDate).not_null())
                    .col(date(HotelOrder::EndDate).not_null().check(
                        Expr::col(HotelOrder::EndDate).gte(Expr::col(HotelOrder::BeginDate)),
                    ))
                    .col(big_integer(HotelOrder::HotelRoomTypeId).not_null())
                    .col(big_integer(HotelOrder::PersonInfoId).not_null())
                    .col(big_integer(HotelOrder::PayTransactionId))
                    .col(big_integer(HotelOrder::RefundTransactionId))
                    .col(
                        decimal_len(HotelOrder::Price, 10, 2)
                            .not_null()
                            .check(Expr::col(HotelOrder::Price).gte(0)),
                    )
                    .col(timestamp_with_time_zone(HotelOrder::CreateTime).not_null())
                    .col(timestamp_with_time_zone(HotelOrder::ActiveTime))
                    .col(timestamp_with_time_zone(HotelOrder::CompleteTime))
                    .col(string(HotelOrder::Status).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(HotelOrder::Table, HotelOrder::HotelId)
                            .to(Hotel::Table, Hotel::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(HotelOrder::Table, HotelOrder::HotelRoomTypeId)
                            .to(HotelRoomType::Table, HotelRoomType::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(HotelOrder::Table, HotelOrder::PersonInfoId)
                            .to(PersonInfo::Table, PersonInfo::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(HotelOrder::Table, HotelOrder::PayTransactionId)
                            .to(Transaction::Table, Transaction::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(HotelOrder::Table, HotelOrder::RefundTransactionId)
                            .to(Transaction::Table, Transaction::Id),
                    )
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
