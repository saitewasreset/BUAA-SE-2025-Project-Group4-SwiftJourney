use crate::m20250411_010610_create_train_type::TrainType;
use crate::m20250411_010617_create_train::Train;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Train::Table)
                    .drop_foreign_key(Alias::new("train_type_id_fkey"))
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Train::Table)
                    .modify_column(ColumnDef::new(Train::TypeId).integer().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Train::Table)
                    .add_foreign_key(
                        TableForeignKey::new()
                            .from_tbl(Train::Table)
                            .from_col(Train::TypeId)
                            .to_tbl(TrainType::Table)
                            .to_col(TrainType::Id)
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
                    .table(Train::Table)
                    .drop_foreign_key(Alias::new("train_type_id_fkey"))
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Train::Table)
                    .modify_column(ColumnDef::new(Train::TypeId).big_integer().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Train::Table)
                    .add_foreign_key(
                        TableForeignKey::new()
                            .from_tbl(Train::Table)
                            .from_col(Train::TypeId)
                            .to_tbl(TrainType::Table)
                            .to_col(TrainType::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
