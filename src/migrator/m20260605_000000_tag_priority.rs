use sea_orm_migration::prelude::*;

/// Global tag priority ranking + derived task priority.
///
/// `tag_priority(tag, position)` holds an operator-ordered ranking
/// (position 0 = highest). A task's effective priority is the priority
/// of its plan's highest-ranked tag, stored in `task.queue_priority`
/// (repurposed from the now-removed "Run next" lever) so `/task/claim`'s
/// existing ORDER BY needs no change.
///
/// Maintained entirely in the DB so it stays correct even though plan
/// tags are edited through PostgREST:
///   * BEFORE INSERT on task         -> stamp queue_priority from plan
///   * AFTER UPDATE on plan (tags)   -> recompute that plan's tasks
///   * AFTER STATEMENT on tag_priority -> recompute all idle/queued tasks
///
/// Also decouples `queued_at` from `queue_priority` changes: a tag-driven
/// recompute must not reset FIFO position within a priority tier.
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260605_000000_tag_priority"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();
        conn.execute_unprepared(
            r#"
            CREATE TABLE tag_priority (
                tag      text PRIMARY KEY,
                position int  NOT NULL CHECK (position >= 0 AND position < 1000000)
            );

            CREATE OR REPLACE FUNCTION task_effective_priority(p_plan_id int)
            RETURNS int AS $$
                SELECT COALESCE(MAX(1000000 - tp.position), 0)
                FROM plan p
                CROSS JOIN LATERAL unnest(p.tags) AS pt(tag)
                JOIN tag_priority tp ON tp.tag = pt.tag
                WHERE p.id = p_plan_id;
            $$ LANGUAGE sql STABLE;

            CREATE OR REPLACE FUNCTION task_set_initial_priority()
            RETURNS trigger AS $$
            BEGIN
                NEW.queue_priority := task_effective_priority(NEW.plan_id);
                RETURN NEW;
            END;
            $$ LANGUAGE plpgsql;

            DROP TRIGGER IF EXISTS task_set_initial_priority_trg ON task;
            CREATE TRIGGER task_set_initial_priority_trg
                BEFORE INSERT ON task
                FOR EACH ROW
                EXECUTE FUNCTION task_set_initial_priority();

            CREATE OR REPLACE FUNCTION plan_tags_recompute_priority()
            RETURNS trigger AS $$
            BEGIN
                IF OLD.tags IS DISTINCT FROM NEW.tags THEN
                    UPDATE task
                       SET queue_priority = task_effective_priority(NEW.id)
                     WHERE plan_id = NEW.id
                       AND task_status IN ('idle', 'queued');
                END IF;
                RETURN NULL;
            END;
            $$ LANGUAGE plpgsql;

            DROP TRIGGER IF EXISTS plan_tags_recompute_priority_trg ON plan;
            CREATE TRIGGER plan_tags_recompute_priority_trg
                AFTER UPDATE ON plan
                FOR EACH ROW
                EXECUTE FUNCTION plan_tags_recompute_priority();

            CREATE OR REPLACE FUNCTION tag_priority_recompute()
            RETURNS trigger AS $$
            BEGIN
                UPDATE task t
                   SET queue_priority = task_effective_priority(t.plan_id)
                 WHERE t.task_status IN ('idle', 'queued');
                RETURN NULL;
            END;
            $$ LANGUAGE plpgsql;

            DROP TRIGGER IF EXISTS tag_priority_recompute_trg ON tag_priority;
            CREATE TRIGGER tag_priority_recompute_trg
                AFTER INSERT OR UPDATE OR DELETE ON tag_priority
                FOR EACH STATEMENT
                EXECUTE FUNCTION tag_priority_recompute();

            CREATE OR REPLACE FUNCTION task_set_queued_at()
            RETURNS trigger AS $$
            BEGIN
                IF NEW.task_status = 'queued' AND (
                    TG_OP = 'INSERT'
                    OR OLD.task_status IS DISTINCT FROM NEW.task_status
                ) THEN
                    NEW.queued_at := NOW();
                END IF;
                RETURN NEW;
            END;
            $$ LANGUAGE plpgsql;

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
            DROP TRIGGER IF EXISTS tag_priority_recompute_trg ON tag_priority;
            DROP FUNCTION IF EXISTS tag_priority_recompute();
            DROP TRIGGER IF EXISTS plan_tags_recompute_priority_trg ON plan;
            DROP FUNCTION IF EXISTS plan_tags_recompute_priority();
            DROP TRIGGER IF EXISTS task_set_initial_priority_trg ON task;
            DROP FUNCTION IF EXISTS task_set_initial_priority();
            DROP FUNCTION IF EXISTS task_effective_priority(int);
            DROP TABLE IF EXISTS tag_priority;

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

            NOTIFY pgrst, 'reload schema';
            "#,
        )
        .await?;
        Ok(())
    }
}
