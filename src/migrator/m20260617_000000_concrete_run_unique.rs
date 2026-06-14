use sea_orm_migration::prelude::*;

/// Make `concrete_run` rows idempotent per `(task_run_id, concrete_key)` so the
/// executor can insert each concrete incrementally as it finalises (giving
/// `created_at` a real per-concrete finish time) while the terminal bulk call
/// safely reconciles any that failed to send — `ON CONFLICT DO NOTHING` keeps
/// the live row and skips the duplicate.
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260617_000000_concrete_run_unique"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                -- Defensive: current behaviour inserts one row per concrete per
                -- run, so this should delete nothing, but the unique index would
                -- fail on any stray duplicate. Keep the lowest id per pair.
                DELETE FROM concrete_run a
                USING concrete_run b
                WHERE a.task_run_id = b.task_run_id
                  AND a.concrete_key = b.concrete_key
                  AND a.id > b.id;

                ALTER TABLE concrete_run
                    ADD CONSTRAINT concrete_run_run_key_uniq
                    UNIQUE (task_run_id, concrete_key);

                NOTIFY pgrst, 'reload schema';
                "#,
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                ALTER TABLE concrete_run DROP CONSTRAINT IF EXISTS concrete_run_run_key_uniq;
                NOTIFY pgrst, 'reload schema';
                "#,
            )
            .await?;
        Ok(())
    }
}
