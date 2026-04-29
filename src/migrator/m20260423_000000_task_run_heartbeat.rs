use sea_orm_migration::prelude::*;

/// Add `last_heartbeat_at` so a reaper can mark runs aborted when the
/// executor dies without sending a terminal state (SIGKILL, node crash).
///
/// The column is touched on every `/task_run/{id}/log/append` call, which
/// the executor's LogStreamer fires every ~1 s while alive. A row that
/// hasn't heart-beat in several minutes is presumed dead.
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260423_000000_task_run_heartbeat"
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
                        ColumnDef::new(TaskRun::LastHeartbeatAt)
                            .timestamp_with_time_zone()
                            .null(),
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
                    .drop_column(TaskRun::LastHeartbeatAt)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum TaskRun {
    Table,
    LastHeartbeatAt,
}
