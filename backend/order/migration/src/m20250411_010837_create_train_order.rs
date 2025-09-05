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
                    .col(
                        ColumnDef::new(TrainOrder::PayTransactionId)
                            .integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(TrainOrder::RefundTransactionId)
                            .integer()
                            .null(),
                    )
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
