use crate::m20250411_010614_create_station::Station;
use crate::m20250411_010620_create_train_schedule::TrainSchedule;
use crate::m20250411_010621_create_seat_type::SeatType;
use crate::m20250411_010719_create_person_info::PersonInfo;
use crate::m20250411_010725_create_transaction::Transaction;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum TrainOrder {
    Table,
    Id,
    Uuid,
    TrainScheduleId,
    SeatTypeId,
    SeatId,
    BeginStationId,
    EndStationId,
    PersonInfoId,
    PayTransactionId,
    RefundTransactionId,
    Price,
    CreateTime,
    ActiveTime,
    CompleteTime,
    Status,
    OrderSeatType,
    PreferredSeatLocation,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TrainOrder::Table)
                    .if_not_exists()
                    .col(pk_auto(TrainOrder::Id))
                    .col(uuid(TrainOrder::Uuid).not_null())
                    .col(integer(TrainOrder::TrainScheduleId).not_null())
                    .col(ColumnDef::new(TrainOrder::SeatTypeId).integer().null())
                    .col(ColumnDef::new(TrainOrder::SeatId).integer().null())
                    .col(integer(TrainOrder::BeginStationId).not_null())
                    .col(integer(TrainOrder::EndStationId).not_null())
                    .col(integer(TrainOrder::PersonInfoId).not_null())
                    .col(integer(TrainOrder::PayTransactionId))
                    .col(integer(TrainOrder::RefundTransactionId))
                    .col(
                        decimal_len(TrainOrder::Price, 10, 2)
                            .not_null()
                            .check(Expr::col(TrainOrder::Price).gte(0)),
                    )
                    .col(timestamp_with_time_zone(TrainOrder::CreateTime).not_null())
                    .col(timestamp_with_time_zone(TrainOrder::ActiveTime))
                    .col(timestamp_with_time_zone(TrainOrder::CompleteTime))
                    .col(string(TrainOrder::Status).not_null())
                    .col(string(TrainOrder::OrderSeatType).not_null())
                    .col(
                        ColumnDef::new(TrainOrder::PreferredSeatLocation)
                            .string()
                            .null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(TrainOrder::Table, TrainOrder::TrainScheduleId)
                            .to(TrainSchedule::Table, TrainSchedule::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(TrainOrder::Table, TrainOrder::SeatTypeId)
                            .to(SeatType::Table, SeatType::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(TrainOrder::Table, TrainOrder::BeginStationId)
                            .to(Station::Table, Station::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(TrainOrder::Table, TrainOrder::EndStationId)
                            .to(Station::Table, Station::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(TrainOrder::Table, TrainOrder::PersonInfoId)
                            .to(PersonInfo::Table, PersonInfo::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(TrainOrder::Table, TrainOrder::PayTransactionId)
                            .to(Transaction::Table, Transaction::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(TrainOrder::Table, TrainOrder::RefundTransactionId)
                            .to(Transaction::Table, Transaction::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TrainOrder::Table).to_owned())
            .await
    }
}
