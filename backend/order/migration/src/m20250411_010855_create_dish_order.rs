use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum DishOrder {
    Table,
    Id,
    Uuid,
    TrainOrderId,
    DishId,
    PersonInfoId,
    PayTransactionId,
    RefundTransactionId,
    Price,
    Amount,
    CreateTime,
    ActiveTime,
    CompleteTime,
    Status,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(DishOrder::Table)
                    .if_not_exists()
                    .col(pk_auto(DishOrder::Id))
                    .col(uuid(DishOrder::Uuid).not_null())
                    .col(integer(DishOrder::TrainOrderId).not_null())
                    .col(integer(DishOrder::DishId).not_null())
                    .col(integer(DishOrder::PersonInfoId).not_null())
                    .col(ColumnDef::new(DishOrder::PayTransactionId).integer().null())
                    .col(
                        ColumnDef::new(DishOrder::RefundTransactionId)
                            .integer()
                            .null(),
                    )
                    .col(
                        decimal_len(DishOrder::Price, 10, 2)
                            .not_null()
                            .check(Expr::col(DishOrder::Price).gte(0)),
                    )
                    .col(
                        integer(DishOrder::Amount)
                            .not_null()
                            .check(Expr::col(DishOrder::Amount).gt(0)),
                    )
                    .col(timestamp_with_time_zone(DishOrder::CreateTime).not_null())
                    .col(timestamp_with_time_zone(DishOrder::ActiveTime))
                    .col(timestamp_with_time_zone(DishOrder::CompleteTime))
                    .col(string(DishOrder::Status).not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(DishOrder::Table).to_owned())
            .await
    }
}
