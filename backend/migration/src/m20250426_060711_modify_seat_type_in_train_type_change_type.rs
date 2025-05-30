use crate::m20250411_010610_create_train_type::TrainType;
use crate::m20250411_010621_create_seat_type::SeatType;
use crate::m20250411_010655_create_seat_type_in_train_type::SeatTypeInTrainType;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(SeatTypeInTrainType::Table)
                    .drop_foreign_key(Alias::new("seat_type_in_train_type_seat_type_id_fkey"))
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(SeatTypeInTrainType::Table)
                    .drop_foreign_key(Alias::new("seat_type_in_train_type_train_type_id_fkey"))
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(SeatTypeInTrainType::Table)
                    .modify_column(
                        ColumnDef::new(SeatTypeInTrainType::SeatTypeId)
                            .integer()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(SeatTypeInTrainType::Table)
                    .modify_column(
                        ColumnDef::new(SeatTypeInTrainType::TrainTypeId)
                            .integer()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(SeatTypeInTrainType::Table)
                    .add_foreign_key(
                        TableForeignKey::new()
                            .from_tbl(SeatTypeInTrainType::Table)
                            .from_col(SeatTypeInTrainType::SeatTypeId)
                            .to_tbl(SeatType::Table)
                            .to_col(SeatType::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(SeatTypeInTrainType::Table)
                    .add_foreign_key(
                        TableForeignKey::new()
                            .from_tbl(SeatTypeInTrainType::Table)
                            .from_col(SeatTypeInTrainType::TrainTypeId)
                            .to_tbl(TrainType::Table)
                            .to_col(TrainType::Id)
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
                    .table(SeatTypeInTrainType::Table)
                    .drop_foreign_key(Alias::new("seat_type_in_train_type_seat_type_id_fkey"))
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(SeatTypeInTrainType::Table)
                    .drop_foreign_key(Alias::new("seat_type_in_train_type_train_type_id_fkey"))
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(SeatTypeInTrainType::Table)
                    .modify_column(
                        ColumnDef::new(SeatTypeInTrainType::SeatTypeId)
                            .big_integer()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(SeatTypeInTrainType::Table)
                    .modify_column(
                        ColumnDef::new(SeatTypeInTrainType::TrainTypeId)
                            .big_integer()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(SeatTypeInTrainType::Table)
                    .add_foreign_key(
                        TableForeignKey::new()
                            .from_tbl(SeatTypeInTrainType::Table)
                            .from_col(SeatTypeInTrainType::SeatTypeId)
                            .to_tbl(SeatType::Table)
                            .to_col(SeatType::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(SeatTypeInTrainType::Table)
                    .add_foreign_key(
                        TableForeignKey::new()
                            .from_tbl(SeatTypeInTrainType::Table)
                            .from_col(SeatTypeInTrainType::TrainTypeId)
                            .to_tbl(TrainType::Table)
                            .to_col(TrainType::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
