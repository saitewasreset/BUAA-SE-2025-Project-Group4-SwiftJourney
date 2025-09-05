use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum Transaction {
    Table,
    Id,
    Uuid,
    CreateTime,
    FinishTime,
    Amount,
    Status,
    UserId,
    Atomic,
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
                    .col(uuid(Transaction::Uuid).not_null())
                    .col(timestamp_with_time_zone(Transaction::CreateTime).not_null())
                    .col(
                        ColumnDef::new(Transaction::FinishTime)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(decimal_len(Transaction::Amount, 10, 2).not_null())
                    .col(string(Transaction::Status).not_null())
                    .col(integer(Transaction::UserId).not_null())
                    .col(boolean(Transaction::Atomic).default(false))
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
