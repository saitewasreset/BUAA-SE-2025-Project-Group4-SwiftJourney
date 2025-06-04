use crate::m20250411_010858_create_takeaway_order::TakeawayOrder;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(TakeawayOrder::Table)
                    .modify_column(
                        ColumnDef::new(TakeawayOrder::PayTransactionId)
                            .integer()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(TakeawayOrder::Table)
                    .modify_column(
                        ColumnDef::new(TakeawayOrder::RefundTransactionId)
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
                    .table(TakeawayOrder::Table)
                    .modify_column(
                        ColumnDef::new(TakeawayOrder::PayTransactionId)
                            .integer()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(TakeawayOrder::Table)
                    .modify_column(
                        ColumnDef::new(TakeawayOrder::RefundTransactionId)
                            .integer()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
