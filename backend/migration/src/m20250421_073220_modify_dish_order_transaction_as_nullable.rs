use crate::m20250411_010855_create_dish_order::DishOrder;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(DishOrder::Table)
                    .modify_column(
                        ColumnDef::new(DishOrder::PayTransactionId)
                            .big_integer()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(DishOrder::Table)
                    .modify_column(
                        ColumnDef::new(DishOrder::RefundTransactionId)
                            .big_integer()
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
                    .table(DishOrder::Table)
                    .modify_column(
                        ColumnDef::new(DishOrder::PayTransactionId)
                            .big_integer()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(DishOrder::Table)
                    .modify_column(
                        ColumnDef::new(DishOrder::RefundTransactionId)
                            .big_integer()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
