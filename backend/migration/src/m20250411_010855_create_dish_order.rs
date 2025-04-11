use crate::m20250411_010719_create_person_info::PersonInfo;
use crate::m20250411_010725_create_transaction::Transaction;
use crate::m20250411_010818_create_dish::Dish;
use crate::m20250411_010837_create_train_order::TrainOrder;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum DishOrder {
    Table,
    Id,
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
                    .col(big_integer(DishOrder::TrainOrderId).not_null())
                    .col(big_integer(DishOrder::DishId).not_null())
                    .col(big_integer(DishOrder::PersonInfoId).not_null())
                    .col(big_integer(DishOrder::PayTransactionId))
                    .col(big_integer(DishOrder::RefundTransactionId))
                    .col(
                        decimal_len(DishOrder::Price, 10, 2)
                            .not_null()
                            .check(Expr::col(DishOrder::Price).gte(0)),
                    )
                    .col(
                        integer(DishOrder::Amount)
                            .not_null()
                            .check(Expr::col(DishOrder::Price).gt(0)),
                    )
                    .col(timestamp_with_time_zone(DishOrder::CreateTime).not_null())
                    .col(timestamp_with_time_zone(DishOrder::ActiveTime))
                    .col(timestamp_with_time_zone(DishOrder::CompleteTime))
                    .col(string(DishOrder::Status).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(DishOrder::Table, DishOrder::TrainOrderId)
                            .to(TrainOrder::Table, TrainOrder::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(DishOrder::Table, DishOrder::DishId)
                            .to(Dish::Table, Dish::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(DishOrder::Table, DishOrder::PersonInfoId)
                            .to(PersonInfo::Table, PersonInfo::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(DishOrder::Table, DishOrder::PayTransactionId)
                            .to(Transaction::Table, Transaction::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(DishOrder::Table, DishOrder::RefundTransactionId)
                            .to(Transaction::Table, Transaction::Id),
                    )
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
