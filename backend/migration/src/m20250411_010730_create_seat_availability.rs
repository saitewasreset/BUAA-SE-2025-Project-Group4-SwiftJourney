use crate::m20250411_010614_create_station::Station;
use crate::m20250411_010620_create_train_schedule::TrainSchedule;
use crate::m20250411_010621_create_seat_type::SeatType;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum SeatAvailability {
    Table,
    Id,
    TrainScheduleId,
    SeatTypeId,
    BeginStationId,
    EndStationId,
    OccupiedSeats,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SeatAvailability::Table)
                    .col(pk_auto(SeatAvailability::Id))
                    .col(integer(SeatAvailability::TrainScheduleId))
                    .col(integer(SeatAvailability::SeatTypeId))
                    .col(integer(SeatAvailability::BeginStationId))
                    .col(integer(SeatAvailability::EndStationId))
                    .col(
                        integer(SeatAvailability::OccupiedSeats)
                            .check(Expr::col(SeatAvailability::OccupiedSeats).gte(0)),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(SeatAvailability::Table, SeatAvailability::TrainScheduleId)
                            .to(TrainSchedule::Table, TrainSchedule::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(SeatAvailability::Table, SeatAvailability::SeatTypeId)
                            .to(SeatType::Table, SeatType::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(SeatAvailability::Table, SeatAvailability::BeginStationId)
                            .to(Station::Table, Station::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(SeatAvailability::Table, SeatAvailability::EndStationId)
                            .to(Station::Table, Station::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                IndexCreateStatement::new()
                    .name("seat_availability_unique")
                    .table(SeatAvailability::Table)
                    .col(SeatAvailability::TrainScheduleId)
                    .col(SeatAvailability::SeatTypeId)
                    .col(SeatAvailability::BeginStationId)
                    .col(SeatAvailability::EndStationId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SeatAvailability::Table).to_owned())
            .await?;

        Ok(())
    }
}
