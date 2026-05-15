use sea_orm_migration::prelude::*;

/// Add `task.last_run_at` so PostgREST can `?order=last_run_at.desc`.
///
/// Background: the Tasks page wants to sort rows by "most recent
/// run" but that value lives on `task_run`, not `task`. PostgREST
/// `order=` only sees parent columns and can't aggregate over an
/// embedded child resource. Without this column the frontend had to
/// either pull the full table to sort client-side (slow) or sort by
/// `id.desc` only (loses the "what just ran" surface).
///
/// Implementation: a denormalised column on `task`, kept current by
/// a trigger on `task_run` insert/update. Read-side cost is one
/// indexed column lookup per row; write-side cost is a single UPDATE
/// per task_run start, which already happens infrequently relative
/// to dashboard reads.
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260516_000000_task_last_run_at"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        conn.execute_unprepared(
            r#"
            ALTER TABLE task
                ADD COLUMN last_run_at timestamptz NULL;

            -- Backfill from existing task_run rows.
            UPDATE task t
               SET last_run_at = sub.max_started
              FROM (
                SELECT task_id, MAX(started_at) AS max_started
                  FROM task_run
                 GROUP BY task_id
              ) sub
             WHERE sub.task_id = t.id;

            -- Index for the common "newest-run-first" sort.
            CREATE INDEX IF NOT EXISTS task_last_run_at_idx
                ON task (last_run_at DESC NULLS LAST);

            -- Trigger: on any task_run insert/update of started_at,
            -- recompute the parent task.last_run_at as MAX(started_at)
            -- of all its runs. MAX is read from the index in O(log n).
            CREATE OR REPLACE FUNCTION task_run_touch_last_run_at()
            RETURNS trigger AS $$
            BEGIN
                UPDATE task
                   SET last_run_at = (
                       SELECT MAX(started_at)
                         FROM task_run
                        WHERE task_id = NEW.task_id
                   )
                 WHERE id = NEW.task_id;
                RETURN NEW;
            END;
            $$ LANGUAGE plpgsql;

            DROP TRIGGER IF EXISTS task_run_last_run_at_trg ON task_run;
            CREATE TRIGGER task_run_last_run_at_trg
                AFTER INSERT OR UPDATE OF started_at ON task_run
                FOR EACH ROW
                EXECUTE FUNCTION task_run_touch_last_run_at();

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
            DROP TRIGGER IF EXISTS task_run_last_run_at_trg ON task_run;
            DROP FUNCTION IF EXISTS task_run_touch_last_run_at();
            DROP INDEX IF EXISTS task_last_run_at_idx;
            ALTER TABLE task DROP COLUMN IF EXISTS last_run_at;
            NOTIFY pgrst, 'reload schema';
            "#,
        )
        .await?;
        Ok(())
    }
}
