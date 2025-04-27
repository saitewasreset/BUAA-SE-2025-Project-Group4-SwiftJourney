use crate::m20250411_010614_create_station::Station;
use crate::m20250411_010620_create_train_schedule::TrainSchedule;
use crate::m20250411_010621_create_seat_type::SeatType;
use crate::m20250411_010719_create_person_info::PersonInfo;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum OccupiedSeat {
    Table,
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
            .create_table(
                Table::create()
                    .table(OccupiedSeat::Table)
                    .if_not_exists()
                    .col(integer(OccupiedSeat::TrainScheduleId).not_null())
                    .col(integer(OccupiedSeat::SeatTypeId).not_null())
                    .col(big_integer(OccupiedSeat::SeatId).not_null())
                    .col(integer(OccupiedSeat::BeginStationId).not_null())
                    .col(integer(OccupiedSeat::EndStationId).not_null())
                    .col(integer(OccupiedSeat::PersonInfoId).not_null())
                    .primary_key(
                        Index::create()
                            .col(OccupiedSeat::TrainScheduleId)
                            .col(OccupiedSeat::SeatTypeId)
                            .col(OccupiedSeat::SeatId),
                    )
                    .index(
                        Index::create()
                            .col(OccupiedSeat::TrainScheduleId)
                            .col(OccupiedSeat::SeatTypeId)
                            .col(OccupiedSeat::SeatId)
                            .col(OccupiedSeat::BeginStationId)
                            .col(OccupiedSeat::EndStationId)
                            .unique(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(OccupiedSeat::Table, OccupiedSeat::TrainScheduleId)
                            .to(TrainSchedule::Table, TrainSchedule::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(OccupiedSeat::Table, OccupiedSeat::SeatTypeId)
                            .to(SeatType::Table, SeatType::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(OccupiedSeat::Table, OccupiedSeat::BeginStationId)
                            .to(Station::Table, Station::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(OccupiedSeat::Table, OccupiedSeat::EndStationId)
                            .to(Station::Table, Station::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(OccupiedSeat::Table, OccupiedSeat::PersonInfoId)
                            .to(PersonInfo::Table, PersonInfo::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(OccupiedSeat::Table).to_owned())
            .await
    }
}
