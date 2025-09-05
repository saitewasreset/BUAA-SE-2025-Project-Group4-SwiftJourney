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
                    .col(integer(Station::CityId).not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_station_name_city_unique")
                    .table(Station::Table)
                    .col(Station::Name)
                    .col(Station::CityId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Station::Table).to_owned())
            .await
    }
}
