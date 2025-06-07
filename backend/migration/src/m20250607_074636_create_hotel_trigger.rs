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
                r#"
                -- 创建评分更新触发器函数
CREATE OR REPLACE FUNCTION update_hotel_rating_count()
RETURNS TRIGGER AS $$
BEGIN
    -- 当有新评分插入时，增加对应酒店的评分计数
    UPDATE hotel 
    SET total_rating_count = total_rating_count + 1 
    WHERE id = NEW.hotel_id;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
"#,
            ))
            .await?;

        manager
            .get_connection()
            .execute(Statement::from_string(
                DatabaseBackend::Postgres,
                r#"
-- 创建在hotel_rating插入后触发的触发器
CREATE TRIGGER hotel_rating_insert_trigger
AFTER INSERT ON hotel_rating
FOR EACH ROW
EXECUTE FUNCTION update_hotel_rating_count()"#,
            ))
            .await?;

        manager
            .get_connection()
            .execute(Statement::from_string(
                DatabaseBackend::Postgres,
                r#"
-- 创建订单状态更新触发器函数
CREATE OR REPLACE FUNCTION update_hotel_booking_count()
RETURNS TRIGGER AS $$
BEGIN
    -- 当状态从未支付(unpaid)变为已支付(paid)时增加计数
    IF OLD.status = 'unpaid' AND NEW.status = 'paid' THEN
        UPDATE hotel
        SET total_booking_count = total_booking_count + NEW.amount
        WHERE id = NEW.hotel_id;

    -- 当状态变为取消(cancelled)或失败(failed)时减少计数
    ELSIF NEW.status IN ('cancelled', 'failed') THEN
        UPDATE hotel
        SET total_booking_count = total_booking_count - NEW.amount
        WHERE id = NEW.hotel_id;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;"#,
            ))
            .await?;

        manager
            .get_connection()
            .execute(Statement::from_string(
                DatabaseBackend::Postgres,
                r#"
-- 创建在hotel_order更新后触发的触发器
CREATE TRIGGER hotel_order_update_trigger
AFTER UPDATE ON hotel_order
FOR EACH ROW
WHEN (OLD.status IS DISTINCT FROM NEW.status)  -- 仅当状态实际变化时触发
EXECUTE FUNCTION update_hotel_booking_count();"#,
            ))
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute(Statement::from_string(
                DatabaseBackend::Postgres,
                "DROP TRIGGER IF EXISTS hotel_order_update_trigger ON hotel_order",
            ))
            .await?;

        manager
            .get_connection()
            .execute(Statement::from_string(
                DatabaseBackend::Postgres,
                "DROP TRIGGER IF EXISTS hotel_rating_insert_trigger ON hotel_rating",
            ))
            .await?;

        manager
            .get_connection()
            .execute(Statement::from_string(
                DatabaseBackend::Postgres,
                "DROP FUNCTION IF EXISTS update_hotel_booking_count",
            ))
            .await?;

        manager
            .get_connection()
            .execute(Statement::from_string(
                DatabaseBackend::Postgres,
                "DROP FUNCTION IF EXISTS update_hotel_rating_count",
            ))
            .await?;

        Ok(())
    }
}
