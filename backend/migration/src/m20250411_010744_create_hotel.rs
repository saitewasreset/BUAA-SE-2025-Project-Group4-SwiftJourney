use crate::m20250411_010603_create_city::City;
use crate::m20250411_010614_create_station::Station;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum Hotel {
    Table,
    Id,
    Name,
    CityId,
    StationId,
    Address,
    Phone,
    Images,
    TotalRatingCount,
    TotalBookingCount,
    Info,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Hotel::Table)
                    .if_not_exists()
                    .col(pk_auto(Hotel::Id))
                    .col(string(Hotel::Name).not_null())
                    .col(big_integer(Hotel::CityId).not_null())
                    .col(big_integer(Hotel::StationId))
                    .col(string(Hotel::Address))
                    .col(json(Hotel::Phone))
                    .col(json(Hotel::Images))
                    .col(
                        integer(Hotel::TotalRatingCount)
                            .default(0)
                            .check(Expr::col(Hotel::TotalRatingCount).gte(0)),
                    )
                    .col(
                        integer(Hotel::TotalBookingCount)
                            .default(0)
                            .check(Expr::col(Hotel::TotalBookingCount).gte(0)),
                    )
                    .col(string(Hotel::Info))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Hotel::Table, Hotel::CityId)
                            .to(City::Table, City::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Hotel::Table, Hotel::StationId)
                            .to(Station::Table, Station::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Hotel::Table).to_owned())
            .await
    }
}
