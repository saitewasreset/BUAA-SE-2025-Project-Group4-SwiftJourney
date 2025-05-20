use crate::m20250411_010614_create_station::Station;
use crate::m20250411_010701_create_route::Route;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Route::Table)
                    .drop_foreign_key(Alias::new("route_station_id_fkey"))
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Route::Table)
                    .modify_column(ColumnDef::new(Route::StationId).integer().not_null())
                    .add_foreign_key(
                        TableForeignKey::new()
                            .from_tbl(Route::Table)
                            .from_col(Route::StationId)
                            .to_tbl(Station::Table)
                            .to_col(Station::Id),
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
                    .table(Route::Table)
                    .drop_foreign_key(Alias::new("route_station_id_fkey"))
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Route::Table)
                    .modify_column(ColumnDef::new(Route::StationId).big_integer().not_null())
                    .add_foreign_key(
                        TableForeignKey::new()
                            .from_tbl(Route::Table)
                            .from_col(Route::StationId)
                            .to_tbl(Station::Table)
                            .to_col(Station::Id),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
