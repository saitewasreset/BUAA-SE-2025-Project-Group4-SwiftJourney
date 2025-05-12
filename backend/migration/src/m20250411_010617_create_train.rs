use crate::m20250411_010610_create_train_type::TrainType;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum Train {
    Table,
    Id,
    Number,
    TypeId,
    DefaultOriginDepartureTime,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Train::Table)
                    .if_not_exists()
                    .col(pk_auto(Train::Id))
                    .col(string(Train::Number).not_null().unique_key())
                    .col(big_integer(Train::TypeId).not_null())
                    .col(integer(Train::DefaultOriginDepartureTime))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Train::Table, Train::TypeId)
                            .to(TrainType::Table, TrainType::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Train::Table).to_owned())
            .await
    }
}
