use crate::m20250411_010617_create_train::Train;
use crate::m20250427_031607_create_train_schedule::TrainSchedule;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum TrainOrder {
    Table,
    Id,
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
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(TrainOrder::Table)
                    .drop_column(crate::m20250411_010837_create_train_order::TrainOrder::TrainId)
                    .drop_column(
                        crate::m20250411_010837_create_train_order::TrainOrder::DepartureDate,
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(TrainOrder::Table)
                    .add_column(integer(TrainOrder::TrainScheduleId).not_null())
                    .add_foreign_key(
                        TableForeignKey::new()
                            .from_tbl(TrainOrder::Table)
                            .from_col(TrainOrder::TrainScheduleId)
                            .to_tbl(TrainSchedule::Table)
                            .to_col(TrainSchedule::Id),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .drop_column(TrainOrder::TrainScheduleId)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .add_column(integer(
                        crate::m20250411_010837_create_train_order::TrainOrder::TrainId,
                    ))
                    .add_column(date(
                        crate::m20250411_010837_create_train_order::TrainOrder::DepartureDate,
                    ))
                    .add_foreign_key(
                        TableForeignKey::new()
                            .from_tbl(crate::m20250411_010837_create_train_order::TrainOrder::Table)
                            .from_col(
                                crate::m20250411_010837_create_train_order::TrainOrder::TrainId,
                            )
                            .to_tbl(Train::Table)
                            .to_col(Train::Id),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
