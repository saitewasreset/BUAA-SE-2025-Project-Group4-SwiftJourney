use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum SeatType {
    Table,
    Id,
    TypeName,
    Capacity,
    Price,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SeatType::Table)
                    .if_not_exists()
                    .col(pk_auto(SeatType::Id))
                    .col(string(SeatType::TypeName).not_null())
                    .col(
                        integer(SeatType::Capacity)
                            .not_null()
                            .check(Expr::col(SeatType::Capacity).gt(0)),
                    )
                    .col(
                        decimal_len(SeatType::Price, 10, 2)
                            .not_null()
                            .check(Expr::col(SeatType::Price).gte(0)),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SeatType::Table).to_owned())
            .await
    }
}
