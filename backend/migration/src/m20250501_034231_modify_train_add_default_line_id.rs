use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
#[allow(dead_code)]
pub enum Train {
    Table,
    Id,
    Number,
    TypeId,
    DefaultLineId,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Train::Table)
                    .add_column(ColumnDef::new(Train::DefaultLineId).integer().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Train::Table)
                    .drop_column(Train::DefaultLineId)
                    .to_owned(),
            )
            .await
    }
}
