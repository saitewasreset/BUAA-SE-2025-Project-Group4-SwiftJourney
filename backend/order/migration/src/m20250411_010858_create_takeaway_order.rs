use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum TakeawayOrder {
    Table,
    Id,
    Uuid,
    TrainOrderId,
    TakeawayDishId,
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
                    .table(TakeawayOrder::Table)
                    .if_not_exists()
                    .col(pk_auto(TakeawayOrder::Id))
                    .col(uuid(TakeawayOrder::Uuid).not_null())
                    .col(integer(TakeawayOrder::TrainOrderId).not_null())
                    .col(integer(TakeawayOrder::TakeawayDishId).not_null())
                    .col(integer(TakeawayOrder::PersonInfoId).not_null())
                    .col(
                        ColumnDef::new(TakeawayOrder::PayTransactionId)
                            .integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(TakeawayOrder::RefundTransactionId)
                            .integer()
                            .null(),
                    )
                    .col(
                        decimal_len(TakeawayOrder::Price, 10, 2)
                            .not_null()
                            .check(Expr::col(TakeawayOrder::Price).gte(0)),
                    )
                    .col(
                        integer(TakeawayOrder::Amount)
                            .not_null()
                            .check(Expr::col(TakeawayOrder::Amount).gt(0)),
                    )
                    .col(timestamp_with_time_zone(TakeawayOrder::CreateTime).not_null())
                    .col(timestamp_with_time_zone(TakeawayOrder::ActiveTime))
                    .col(timestamp_with_time_zone(TakeawayOrder::CompleteTime))
                    .col(string(TakeawayOrder::Status).not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TakeawayOrder::Table).to_owned())
            .await
    }
}
