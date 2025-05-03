use crate::m20250411_010603_create_city::City;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(City::Table)
                    .add_column(ColumnDef::new(Alias::new("province")).string().not_null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(City::Table)
                    .drop_column(Alias::new("province"))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
