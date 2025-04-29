use crate::m20250411_010603_create_city::City;
use crate::m20250411_010614_create_station::Station;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Station::Table)
                    .drop_foreign_key(Alias::new("station_city_id_fkey"))
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Station::Table)
                    .modify_column(ColumnDef::new(Station::CityId).integer().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Station::Table)
                    .add_foreign_key(
                        TableForeignKey::new()
                            .from_tbl(Station::Table)
                            .from_col(Station::CityId)
                            .to_tbl(City::Table)
                            .to_col(City::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Station::Table)
                    .drop_foreign_key(Alias::new("station_city_id_fkey"))
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Station::Table)
                    .modify_column(ColumnDef::new(Station::CityId).big_integer().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Station::Table)
                    .add_foreign_key(
                        TableForeignKey::new()
                            .from_tbl(Station::Table)
                            .from_col(Station::CityId)
                            .to_tbl(City::Table)
                            .to_col(City::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
