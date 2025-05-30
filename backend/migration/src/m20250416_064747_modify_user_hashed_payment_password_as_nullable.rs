use crate::m20250411_010715_create_user::User;
use crate::sea_orm::{DatabaseBackend, Statement};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(User::Table)
                    .modify_column(ColumnDef::new(User::HashedPaymentPassword).binary().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute(Statement::from_string(
                DatabaseBackend::Postgres,
                "DELETE FROM \"user\" WHERE hashed_payment_password IS NULL",
            ))
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(User::Table)
                    .modify_column(
                        ColumnDef::new(User::HashedPaymentPassword)
                            .binary()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }
}
