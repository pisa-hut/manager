use sea_orm_migration::prelude::*;

/// Rebuild `task_status` with clearer names and split meanings:
///   created   -> idle        (not queued, awaits user action)
///   pending   -> queued      (unambiguous)
///   running   -> running     (unchanged)
///   completed -> completed   (unchanged)
///   failed    -> exhausted   (permanent task-level failure — per-run
///                              `failed` is still a transient state on
///                              task_run and means "crashed, retry")
///   invalid   -> invalid     (unchanged)
///                 aborted    (new — user stopped via web UI / scancel)
///
/// Also: add `invalid` to `task_run_status` so invalidate_task can stop
/// abusing `completed` for crashed-on-bad-config runs.
///
/// Also: rename `task.retry_count` -> `task.attempt_count` (same
/// semantics; the column has always held total attempts, not retries).
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260425_000000_task_status_rename"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Enum surgery has to be done in raw SQL — sea-query's alter-type
        // doesn't cover "rebuild with different variants" in one shot.
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                -- Rebuild task_status with renamed values.
                CREATE TYPE task_status_new AS ENUM (
                    'idle', 'queued', 'running',
                    'completed', 'exhausted', 'invalid', 'aborted'
                );

                ALTER TABLE task ADD COLUMN task_status_new task_status_new;
                UPDATE task SET task_status_new = CASE task_status
                    WHEN 'created'::task_status   THEN 'idle'::task_status_new
                    WHEN 'pending'::task_status   THEN 'queued'::task_status_new
                    WHEN 'running'::task_status   THEN 'running'::task_status_new
                    WHEN 'completed'::task_status THEN 'completed'::task_status_new
                    WHEN 'failed'::task_status    THEN 'exhausted'::task_status_new
                    WHEN 'invalid'::task_status   THEN 'invalid'::task_status_new
                END;

                ALTER TABLE task DROP COLUMN task_status;
                ALTER TABLE task RENAME COLUMN task_status_new TO task_status;
                ALTER TABLE task ALTER COLUMN task_status SET NOT NULL;

                DROP TYPE task_status;
                ALTER TYPE task_status_new RENAME TO task_status;

                -- task_run_status gains `invalid`.
                ALTER TYPE task_run_status ADD VALUE IF NOT EXISTS 'invalid';

                -- retry_count was always a count of attempts, off-by-one
                -- relative to "retries". Rename to match reality.
                ALTER TABLE task RENAME COLUMN retry_count TO attempt_count;
                "#,
            )
            .await?;
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        // Irreversible in practice: `aborted` on task_status and `invalid`
        // on task_run_status have no pre-migration equivalents, so a blind
        // down() would fail on any existing rows using them. If you really
        // need to roll back, do it manually once live data is drained.
        Err(DbErr::Migration(
            "m20260425_000000_task_status_rename cannot be rolled back \
             automatically; manually map aborted/invalid rows and revert."
                .to_string(),
        ))
    }
}
