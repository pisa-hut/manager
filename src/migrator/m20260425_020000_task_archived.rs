use sea_orm_migration::prelude::*;

/// Add `task.archived` — a soft-hide flag orthogonal to `task_status`.
///
/// `task_status` answers "what's happening to this task?" (idle / queued
/// / running / completed / invalid / aborted). `archived` answers "do I
/// want to see this row in the default list?". Triage of an `invalid`
/// task that the user decides isn't ours to fix flips `archived = true`
/// instead of mutating `task_status`, which keeps the state machine pure
/// and lets us audit "tasks the user has acknowledged" without parsing
/// `aborted` rows for two distinct intents.
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260425_020000_task_archived"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Task::Table)
                    .add_column(
                        ColumnDef::new(Task::Archived)
                            .boolean()
                            .not_null()
                            .default(false),
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
                    .table(Task::Table)
                    .drop_column(Task::Archived)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Task {
    Table,
    Archived,
}
