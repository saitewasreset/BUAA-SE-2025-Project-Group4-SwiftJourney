use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
#[allow(dead_code)]
pub enum Transaction {
    Table,
    Id,
    Uuid,
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
            .alter_table(
                Table::alter()
                    .table(Transaction::Table)
                    .add_column(ColumnDef::new(Transaction::Uuid).uuid().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Transaction::Table)
                    .col(Transaction::Uuid)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Transaction::Table).to_owned())
            .await?;

        Ok(())
    }
}
