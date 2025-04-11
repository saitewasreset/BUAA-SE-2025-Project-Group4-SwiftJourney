use crate::m20250411_010603_create_city::City;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum Station {
    Table,
    Id,
    Name,
    CityId,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Station::Table)
                    .if_not_exists()
                    .col(pk_auto(Station::Id))
                    .col(string(Station::Name).not_null())
                    .col(big_integer(Station::CityId).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Station::Table, Station::CityId)
                            .to(City::Table, City::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Station::Table).to_owned())
            .await
    }
}
