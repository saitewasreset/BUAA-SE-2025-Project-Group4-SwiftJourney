use crate::m20250411_010610_create_train_type::TrainType;
use crate::m20250411_010621_create_seat_type::SeatType;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum SeatTypeMapping {
    Table,
    TrainTypeId,
    SeatTypeId,
    SeatId,
    Carriage,
    Row,
    Location,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SeatTypeMapping::Table)
                    .if_not_exists()
                    .col(big_integer(SeatTypeMapping::TrainTypeId).not_null())
                    .col(big_integer(SeatTypeMapping::SeatTypeId).not_null())
                    .col(big_integer(SeatTypeMapping::SeatId).not_null())
                    .col(integer(SeatTypeMapping::Carriage).not_null())
                    .col(integer(SeatTypeMapping::Row).not_null())
                    .col(char(SeatTypeMapping::Location).not_null())
                    .primary_key(
                        Index::create()
                            .col(SeatTypeMapping::TrainTypeId)
                            .col(SeatTypeMapping::SeatTypeId)
                            .col(SeatTypeMapping::SeatId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(SeatTypeMapping::Table, SeatTypeMapping::TrainTypeId)
                            .to(TrainType::Table, TrainType::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(SeatTypeMapping::Table, SeatTypeMapping::SeatTypeId)
                            .to(SeatType::Table, SeatType::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SeatTypeMapping::Table).to_owned())
            .await
    }
}
