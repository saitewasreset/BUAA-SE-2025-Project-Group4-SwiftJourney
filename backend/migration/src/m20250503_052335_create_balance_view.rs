use crate::sea_orm::{DatabaseBackend, Statement};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute(Statement::from_string(
                DatabaseBackend::Postgres,
                "CREATE OR REPLACE VIEW balance AS
SELECT
    user_id,
    SUM(CASE
        WHEN amount < 0 THEN -amount
        WHEN amount > 0 THEN -amount
        ELSE 0
    END) AS balance
FROM
    transaction
WHERE
    status = 'Paid'
GROUP BY
    user_id",
            ))
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute(Statement::from_string(
                DatabaseBackend::Postgres,
                "DROP VIEW balance",
            ))
            .await?;

        Ok(())
    }
}
