use crate::m20250411_010715_create_user::User;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum Transaction {
    Table,
    Id,
    CreateTime,
    FinishTime,
    Amount,
    Status,
    UserId,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Transaction::Table)
                    .if_not_exists()
                    .col(pk_auto(Transaction::Id))
                    .col(timestamp_with_time_zone(Transaction::CreateTime).not_null())
                    .col(timestamp_with_time_zone(Transaction::FinishTime))
                    .col(decimal_len(Transaction::Amount, 10, 2).not_null())
                    .col(string(Transaction::Status).not_null())
                    .col(big_integer(Transaction::UserId).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Transaction::Table, Transaction::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Transaction::Table).to_owned())
            .await
    }
}
