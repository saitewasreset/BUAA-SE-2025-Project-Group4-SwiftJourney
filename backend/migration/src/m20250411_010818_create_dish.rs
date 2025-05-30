use crate::m20250411_010617_create_train::Train;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum Dish {
    Table,
    Id,
    TrainId,
    Type,
    Time,
    Name,
    Price,
    Images,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Dish::Table)
                    .if_not_exists()
                    .col(pk_auto(Dish::Id))
                    .col(integer(Dish::TrainId).not_null())
                    .col(string(Dish::Type).not_null())
                    .col(string(Dish::Time).not_null())
                    .col(string(Dish::Name).not_null())
                    .col(
                        decimal_len(Dish::Price, 10, 2)
                            .not_null()
                            .check(Expr::col(Dish::Price).gte(0)),
                    )
                    .col(json(Dish::Images))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Dish::Table, Dish::TrainId)
                            .to(Train::Table, Train::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Dish::Table).to_owned())
            .await
    }
}
