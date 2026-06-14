use sea_orm_migration::prelude::*;

/// Total concrete runs the sampler expects for this task_run, reported
/// live mid-run by the executor. Nullable: open-ended (continuous /
/// adaptive) samplers don't know a total, so the UI falls back to a
/// composition bar when this is NULL and a fill-toward-N bar when set.
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260616_000000_task_run_expected"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(TaskRun::Table)
                    .add_column(
                        ColumnDef::new(TaskRun::ExpectedConcreteRuns)
                            .integer()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .get_connection()
            .execute_unprepared("NOTIFY pgrst, 'reload schema';")
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(TaskRun::Table)
                    .drop_column(TaskRun::ExpectedConcreteRuns)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum TaskRun {
    Table,
    ExpectedConcreteRuns,
}
