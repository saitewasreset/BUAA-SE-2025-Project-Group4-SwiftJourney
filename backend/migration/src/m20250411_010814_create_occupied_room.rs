use crate::m20250411_010719_create_person_info::PersonInfo;
use crate::m20250411_010744_create_hotel::Hotel;
use crate::m20250411_010751_create_hotel_room_type::HotelRoomType;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum OccupiedRoom {
    Table,
    Id,
    HotelId,
    RoomTypeId,
    BeginDate,
    EndDate,
    PersonInfoId,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(OccupiedRoom::Table)
                    .if_not_exists()
                    .col(pk_auto(OccupiedRoom::Id))
                    .col(integer(OccupiedRoom::HotelId).not_null())
                    .col(integer(OccupiedRoom::RoomTypeId).not_null())
                    .col(date(OccupiedRoom::BeginDate).not_null())
                    .col(date(OccupiedRoom::EndDate).not_null().check(
                        Expr::col(OccupiedRoom::EndDate).gte(Expr::col(OccupiedRoom::BeginDate)),
                    ))
                    .col(integer(OccupiedRoom::PersonInfoId).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(OccupiedRoom::Table, OccupiedRoom::HotelId)
                            .to(Hotel::Table, Hotel::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(OccupiedRoom::Table, OccupiedRoom::RoomTypeId)
                            .to(HotelRoomType::Table, HotelRoomType::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(OccupiedRoom::Table, OccupiedRoom::PersonInfoId)
                            .to(PersonInfo::Table, PersonInfo::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(OccupiedRoom::Table).to_owned())
            .await
    }
}
