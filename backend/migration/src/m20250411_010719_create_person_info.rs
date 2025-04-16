use crate::m20250411_010715_create_user::User;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum PersonInfo {
    Table,
    Id,
    Name,
    IdentityCard,
    PreferredSeatLocation,
    UserId,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PersonInfo::Table)
                    .if_not_exists()
                    .col(pk_auto(PersonInfo::Id))
                    .col(string(PersonInfo::Name).not_null())
                    .col(string(PersonInfo::IdentityCard).not_null())
                    .col(char(PersonInfo::PreferredSeatLocation))
                    .col(big_integer(PersonInfo::UserId).not_null())
                    .index(
                        Index::create()
                            .col(PersonInfo::UserId)
                            .col(PersonInfo::IdentityCard)
                            .unique(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(PersonInfo::Table, PersonInfo::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PersonInfo::Table).to_owned())
            .await
    }
}
