use sea_orm_migration::prelude::*;

/// Drop `executor.slurm_array_id`.
///
/// The column captured the SLURM array task index for the worker that
/// claimed a task, but no consumer reads it: not the manager's claim
/// path, not the reaper, not the frontend (after this migration's
/// companion frontend PR removes its column), not the scheduler. Drop
/// it instead of letting it accumulate dead writes.
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260503_000000_drop_executor_slurm_array_id"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Executor::Table)
                    .drop_column(Executor::SlurmArrayId)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add the column back with a 0 default so existing rows are
        // valid; new inserts have to provide it explicitly to match
        // the original NOT NULL semantics.
        manager
            .alter_table(
                Table::alter()
                    .table(Executor::Table)
                    .add_column(
                        ColumnDef::new(Executor::SlurmArrayId)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Executor {
    Table,
    SlurmArrayId,
}
