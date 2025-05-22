use crate::m20250411_010610_create_train_type::TrainType;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_index(
                Index::create()
                    .table(TrainType::Table)
                    .name("idx_train_type_type_name_unique")
                    .col(TrainType::TypeName)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .table(TrainType::Table)
                    .name("idx_train_type_type_name_unique")
                    .to_owned(),
            )
            .await
    }
}
