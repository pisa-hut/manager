use sea_orm_migration::prelude::*;

/// Replace the single `concrete_scenarios_executed` column with three
/// cumulative counters (`finished`, `aborted`, `skipped`). simcore's
/// `ExecResult` now carries all three end-to-end; the manager records the
/// cumulative snapshot at each task_run finalisation. The useless-streak
/// rule moves from `concrete_scenarios_executed == 0` to "current
/// cumulative sum equals the oldest in the streak window" (cumulative is
/// monotonic, so equality there ⇒ no growth in any window member).
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260601_000000_task_run_concrete_run_split"
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
                        ColumnDef::new(TaskRun::FinishedConcreteRuns)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .add_column(
                        ColumnDef::new(TaskRun::AbortedConcreteRuns)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .add_column(
                        ColumnDef::new(TaskRun::SkippedConcreteRuns)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .drop_column(TaskRun::ConcreteScenariosExecuted)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(TaskRun::Table)
                    .add_column(
                        ColumnDef::new(TaskRun::ConcreteScenariosExecuted)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .drop_column(TaskRun::FinishedConcreteRuns)
                    .drop_column(TaskRun::AbortedConcreteRuns)
                    .drop_column(TaskRun::SkippedConcreteRuns)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum TaskRun {
    Table,
    ConcreteScenariosExecuted,
    FinishedConcreteRuns,
    AbortedConcreteRuns,
    SkippedConcreteRuns,
}
