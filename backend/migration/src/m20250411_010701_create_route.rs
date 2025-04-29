use crate::m20250411_010614_create_station::Station;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum Route {
    Table,
    Id,
    LineId,
    StationId,
    ArrivalTime,
    DepartureTime,
    Order,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Route::Table)
                    .if_not_exists()
                    .col(pk_auto(Route::Id))
                    .col(big_integer(Route::LineId).not_null())
                    .col(big_integer(Route::StationId).not_null())
                    .col(
                        integer(Route::ArrivalTime)
                            .not_null()
                            .check(Expr::col(Route::ArrivalTime).gte(0)),
                    )
                    .col(
                        integer(Route::DepartureTime)
                            .not_null()
                            .check(Expr::col(Route::DepartureTime).gte(0)),
                    )
                    .col(
                        integer(Route::Order)
                            .not_null()
                            .check(Expr::col(Route::Order).gte(0)),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Route::Table, Route::StationId)
                            .to(Station::Table, Station::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Route::Table).to_owned())
            .await
    }
}
