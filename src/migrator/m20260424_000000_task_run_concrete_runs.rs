use sea_orm_migration::prelude::*;

/// Count of concrete-scenario executions that actually ran to completion
/// during this task_run. The fail-task path uses this to decide whether
/// a task should be permanently failed (10 consecutive useless runs, i.e.
/// count = 0) or requeued for another retry — replacing the earlier
/// "10 consecutive identical error messages" heuristic.
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260424_000000_task_run_concrete_runs"
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
                        ColumnDef::new(TaskRun::ConcreteScenariosExecuted)
                            .integer()
                            .not_null()
                            .default(0),
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
                    .table(TaskRun::Table)
                    .drop_column(TaskRun::ConcreteScenariosExecuted)
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
}
