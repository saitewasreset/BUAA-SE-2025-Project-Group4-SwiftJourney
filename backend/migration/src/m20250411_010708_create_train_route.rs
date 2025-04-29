use crate::m20250411_010617_create_train::Train;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum TrainRoute {
    Table,
    TrainId,
    LineId,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TrainRoute::Table)
                    .if_not_exists()
                    .col(big_integer(TrainRoute::TrainId).not_null())
                    .col(big_integer(TrainRoute::LineId).not_null())
                    .primary_key(
                        Index::create()
                            .col(TrainRoute::TrainId)
                            .col(TrainRoute::LineId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(TrainRoute::Table, TrainRoute::TrainId)
                            .to(Train::Table, Train::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TrainRoute::Table).to_owned())
            .await
    }
}
