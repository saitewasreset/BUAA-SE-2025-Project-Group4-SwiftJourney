use crate::m20250411_010617_create_train::Train;
use crate::m20250411_010708_create_train_route::TrainRoute;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(TrainRoute::Table)
                    .drop_foreign_key(Alias::new("train_route_train_id_fkey"))
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(TrainRoute::Table)
                    .modify_column(ColumnDef::new(TrainRoute::TrainId).integer().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(TrainRoute::Table)
                    .add_foreign_key(
                        TableForeignKey::new()
                            .from_tbl(TrainRoute::Table)
                            .from_col(TrainRoute::TrainId)
                            .to_tbl(Train::Table)
                            .to_col(Train::Id)
                            .on_delete(ForeignKeyAction::Cascade),
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
                    .table(TrainRoute::Table)
                    .drop_foreign_key(Alias::new("train_route_train_id_fkey"))
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(TrainRoute::Table)
                    .modify_column(ColumnDef::new(TrainRoute::TrainId).big_integer().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(TrainRoute::Table)
                    .add_foreign_key(
                        TableForeignKey::new()
                            .from_tbl(TrainRoute::Table)
                            .from_col(TrainRoute::TrainId)
                            .to_tbl(Train::Table)
                            .to_col(Train::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
