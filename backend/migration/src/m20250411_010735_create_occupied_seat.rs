use crate::m20250411_010719_create_person_info::PersonInfo;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum OccupiedSeat {
    Table,
    SeatAvailabilityId,
    SeatId,
    PersonInfoId,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(OccupiedSeat::Table)
                    .if_not_exists()
                    .col(integer(OccupiedSeat::SeatAvailabilityId).not_null())
                    .col(big_integer(OccupiedSeat::SeatId).not_null())
                    .col(integer(OccupiedSeat::PersonInfoId).not_null())
                    .primary_key(
                        Index::create()
                            .col(OccupiedSeat::SeatAvailabilityId)
                            .col(OccupiedSeat::SeatId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(OccupiedSeat::Table, OccupiedSeat::PersonInfoId)
                            .to(PersonInfo::Table, PersonInfo::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(OccupiedSeat::Table).to_owned())
            .await
    }
}
