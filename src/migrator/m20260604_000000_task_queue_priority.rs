use sea_orm_migration::prelude::*;

/// Add `task.queue_priority` and `task.queued_at` to give operators
/// a "Run next" boost lever, and to let the claim ordering favour
/// previously-attempted tasks before fresh ones.
///
/// New claim ORDER BY:
///   queue_priority DESC,
///   LEAST(attempt_count, 3) DESC,
///   queued_at ASC NULLS LAST,
///   id ASC
///
/// `queued_at` is maintained by a trigger so every code path that
/// transitions a row into `queued` (manager create, manager re-queue
/// after fail, frontend Run/Run-next PostgREST PATCH) gets the
/// timestamp without per-call-site bookkeeping. The trigger also fires
/// when `queue_priority` changes while the row is already `queued`, so
/// boosting an existing queued task refreshes its FIFO position
/// relative to other boosted peers.
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260604_000000_task_queue_priority"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        conn.execute_unprepared(
            r#"
            ALTER TABLE task
                ADD COLUMN queue_priority int NOT NULL DEFAULT 0,
                ADD COLUMN queued_at      timestamptz NULL;

            -- Backfill: anything currently in the queue gets its
            -- create-time so the new ordering is well-defined the
            -- moment the manager comes up.
            UPDATE task
               SET queued_at = created_at
             WHERE task_status = 'queued';

            -- Index supports the new ORDER BY.
            CREATE INDEX IF NOT EXISTS task_queue_priority_queued_at_idx
                ON task (queue_priority DESC, queued_at ASC NULLS LAST)
             WHERE task_status = 'queued';

            -- Trigger: stamp queued_at whenever the row enters (or
            -- re-enters) the queued state, OR when queue_priority
            -- changes on an already-queued row. Random unrelated
            -- updates (e.g. archived flip) do NOT touch queued_at.
            CREATE OR REPLACE FUNCTION task_set_queued_at()
            RETURNS trigger AS $$
            BEGIN
                IF NEW.task_status = 'queued' AND (
                    TG_OP = 'INSERT'
                    OR OLD.task_status IS DISTINCT FROM NEW.task_status
                    OR OLD.queue_priority IS DISTINCT FROM NEW.queue_priority
                ) THEN
                    NEW.queued_at := NOW();
                END IF;
                RETURN NEW;
            END;
            $$ LANGUAGE plpgsql;

            DROP TRIGGER IF EXISTS task_set_queued_at_trg ON task;
            CREATE TRIGGER task_set_queued_at_trg
                BEFORE INSERT OR UPDATE ON task
                FOR EACH ROW
                EXECUTE FUNCTION task_set_queued_at();

            NOTIFY pgrst, 'reload schema';
            "#,
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();
        conn.execute_unprepared(
            r#"
            DROP TRIGGER IF EXISTS task_set_queued_at_trg ON task;
            DROP FUNCTION IF EXISTS task_set_queued_at();
            DROP INDEX IF EXISTS task_queue_priority_queued_at_idx;
            ALTER TABLE task
                DROP COLUMN IF EXISTS queued_at,
                DROP COLUMN IF EXISTS queue_priority;
            NOTIFY pgrst, 'reload schema';
            "#,
        )
        .await?;
        Ok(())
    }
}
