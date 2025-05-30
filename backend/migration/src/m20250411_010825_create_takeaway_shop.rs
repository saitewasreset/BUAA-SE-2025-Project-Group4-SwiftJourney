use crate::m20250411_010614_create_station::Station;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum TakeawayShop {
    Table,
    Id,
    Uuid,
    Name,
    StationId,
    Images,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TakeawayShop::Table)
                    .if_not_exists()
                    .col(pk_auto(TakeawayShop::Id))
                    .col(uuid(TakeawayShop::Uuid))
                    .col(string(TakeawayShop::Name).not_null())
                    .col(integer(TakeawayShop::StationId).not_null())
                    .col(json(TakeawayShop::Images).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(TakeawayShop::Table, TakeawayShop::StationId)
                            .to(Station::Table, Station::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TakeawayShop::Table).to_owned())
            .await
    }
}
