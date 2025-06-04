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
                    .col(integer(HotelOrder::PayTransactionId))
                    .col(integer(HotelOrder::RefundTransactionId))
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
                    .foreign_key(
                        ForeignKey::create()
                            .from(HotelOrder::Table, HotelOrder::HotelId)
                            .to(Hotel::Table, Hotel::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(HotelOrder::Table, HotelOrder::HotelRoomTypeId)
                            .to(HotelRoomType::Table, HotelRoomType::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(HotelOrder::Table, HotelOrder::PersonInfoId)
                            .to(PersonInfo::Table, PersonInfo::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(HotelOrder::Table, HotelOrder::PayTransactionId)
                            .to(Transaction::Table, Transaction::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(HotelOrder::Table, HotelOrder::RefundTransactionId)
                            .to(Transaction::Table, Transaction::Id)
                            .on_delete(ForeignKeyAction::Cascade),
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
