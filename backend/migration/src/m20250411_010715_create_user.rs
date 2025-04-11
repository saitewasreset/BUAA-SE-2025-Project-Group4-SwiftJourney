use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum User {
    Table,
    Id,
    Username,
    HashedPassword,
    HashedPaymentPassword,
    Salt,
    WrongPaymentPasswordTried,
    Gender,
    Age,
    Phone,
    Email,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .if_not_exists()
                    .col(pk_auto(User::Id))
                    .col(string(User::Username).not_null())
                    .col(binary(User::HashedPassword).not_null())
                    .col(binary(User::HashedPaymentPassword).not_null())
                    .col(binary(User::Salt).not_null())
                    .col(
                        integer(User::WrongPaymentPasswordTried)
                            .not_null()
                            .default(0),
                    )
                    .col(string(User::Gender))
                    .col(integer(User::Age).check(Expr::col(User::Age).gte(0)))
                    .col(string(User::Phone).unique_key())
                    .col(string(User::Email))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await
    }
}
