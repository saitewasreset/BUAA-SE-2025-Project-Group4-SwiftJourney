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
                    .col(integer(SeatTypeInTrainType::SeatTypeId).not_null())
                    .col(integer(SeatTypeInTrainType::TrainTypeId).not_null())
                    .primary_key(
                        Index::create()
                            .col(SeatTypeInTrainType::SeatTypeId)
                            .col(SeatTypeInTrainType::TrainTypeId),
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
