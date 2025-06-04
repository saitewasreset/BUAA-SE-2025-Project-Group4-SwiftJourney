use crate::m20250411_010837_create_train_order::TrainOrder;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(TrainOrder::Table)
                    .modify_column(
                        ColumnDef::new(TrainOrder::PayTransactionId)
                            .integer()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(TrainOrder::Table)
                    .modify_column(
                        ColumnDef::new(TrainOrder::RefundTransactionId)
                            .integer()
                            .null(),
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
                    .table(TrainOrder::Table)
                    .modify_column(
                        ColumnDef::new(TrainOrder::PayTransactionId)
                            .integer()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(TrainOrder::Table)
                    .modify_column(
                        ColumnDef::new(TrainOrder::RefundTransactionId)
                            .integer()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
