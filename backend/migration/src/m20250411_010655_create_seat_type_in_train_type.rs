use crate::m20250411_010610_create_train_type::TrainType;
use crate::m20250411_010621_create_seat_type::SeatType;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum SeatTypeInTrainType {
    Table,
    SeatTypeId,
    TrainTypeId,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SeatTypeInTrainType::Table)
                    .if_not_exists()
                    .col(big_integer(SeatTypeInTrainType::SeatTypeId).not_null())
                    .col(big_integer(SeatTypeInTrainType::TrainTypeId).not_null())
                    .primary_key(
                        Index::create()
                            .col(SeatTypeInTrainType::SeatTypeId)
                            .col(SeatTypeInTrainType::TrainTypeId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(SeatTypeInTrainType::Table, SeatTypeInTrainType::SeatTypeId)
                            .to(SeatType::Table, SeatType::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(SeatTypeInTrainType::Table, SeatTypeInTrainType::TrainTypeId)
                            .to(TrainType::Table, TrainType::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SeatTypeInTrainType::Table).to_owned())
            .await
    }
}
