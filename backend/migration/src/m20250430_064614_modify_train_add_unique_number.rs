use crate::m20250411_010617_create_train::Train;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_index(
                Index::create()
                    .table(Train::Table)
                    .name("idx_train_number_unique")
                    .col(Train::Number)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .table(Train::Table)
                    .name("idx_train_number_unique")
                    .to_owned(),
            )
            .await
    }
}
