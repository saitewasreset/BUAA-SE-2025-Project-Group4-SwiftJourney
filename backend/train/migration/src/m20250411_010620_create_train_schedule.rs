use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum TrainSchedule {
    Table,
    Id,
    TrainId,
    DepartureDate,
    OriginDepartureTime,
    LineId,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TrainSchedule::Table)
                    .col(pk_auto(TrainSchedule::Id))
                    .col(integer(TrainSchedule::TrainId))
                    .col(date(TrainSchedule::DepartureDate))
                    .col(integer(TrainSchedule::OriginDepartureTime))
                    .col(big_integer(TrainSchedule::LineId))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TrainSchedule::Table).to_owned())
            .await
    }
}
