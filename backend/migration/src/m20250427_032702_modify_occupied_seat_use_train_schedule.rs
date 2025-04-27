use crate::m20250411_010617_create_train::Train;
use crate::m20250427_031607_create_train_schedule::TrainSchedule;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum OccupiedSeat {
    Table,
    Id,
    TrainScheduleId,
    SeatTypeId,
    SeatId,
    BeginStationId,
    EndStationId,
    PersonInfoId,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(OccupiedSeat::Table)
                    .drop_column(
                        crate::m20250411_010735_create_occupied_seat::OccupiedSeat::TrainId,
                    )
                    .drop_column(
                        crate::m20250411_010735_create_occupied_seat::OccupiedSeat::DepartureDate,
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(OccupiedSeat::Table)
                    .add_column(integer(OccupiedSeat::TrainScheduleId).not_null())
                    .add_foreign_key(
                        TableForeignKey::new()
                            .from_tbl(OccupiedSeat::Table)
                            .from_col(OccupiedSeat::TrainScheduleId)
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
                    .drop_column(OccupiedSeat::TrainScheduleId)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .add_column(integer(
                        crate::m20250411_010735_create_occupied_seat::OccupiedSeat::TrainId,
                    ))
                    .add_column(date(
                        crate::m20250411_010735_create_occupied_seat::OccupiedSeat::DepartureDate,
                    ))
                    .add_foreign_key(
                        TableForeignKey::new()
                            .from_tbl(
                                crate::m20250411_010735_create_occupied_seat::OccupiedSeat::Table,
                            )
                            .from_col(
                                crate::m20250411_010735_create_occupied_seat::OccupiedSeat::TrainId,
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
