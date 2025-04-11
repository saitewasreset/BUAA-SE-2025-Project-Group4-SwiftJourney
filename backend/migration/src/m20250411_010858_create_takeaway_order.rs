use crate::m20250411_010719_create_person_info::PersonInfo;
use crate::m20250411_010725_create_transaction::Transaction;
use crate::m20250411_010827_create_takeaway_dish::TakeawayDish;
use crate::m20250411_010837_create_train_order::TrainOrder;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum TakeawayOrder {
    Table,
    Id,
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
                    .col(big_integer(TakeawayOrder::TrainOrderId).not_null())
                    .col(big_integer(TakeawayOrder::TakeawayDishId).not_null())
                    .col(big_integer(TakeawayOrder::PersonInfoId).not_null())
                    .col(big_integer(TakeawayOrder::PayTransactionId))
                    .col(big_integer(TakeawayOrder::RefundTransactionId))
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
                    .foreign_key(
                        ForeignKey::create()
                            .from(TakeawayOrder::Table, TakeawayOrder::TrainOrderId)
                            .to(TrainOrder::Table, TrainOrder::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(TakeawayOrder::Table, TakeawayOrder::TakeawayDishId)
                            .to(TakeawayDish::Table, TakeawayDish::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(TakeawayOrder::Table, TakeawayOrder::PersonInfoId)
                            .to(PersonInfo::Table, PersonInfo::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(TakeawayOrder::Table, TakeawayOrder::PayTransactionId)
                            .to(Transaction::Table, Transaction::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(TakeawayOrder::Table, TakeawayOrder::RefundTransactionId)
                            .to(Transaction::Table, Transaction::Id),
                    )
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
