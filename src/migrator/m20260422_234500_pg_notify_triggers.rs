use sea_orm_migration::prelude::*;

/// Emit `pg_notify('pisa_events', …)` on every insert/update/delete of
/// `task` and `task_run` so the manager can fan-out realtime events to
/// SSE subscribers instead of the frontend polling every 5 s.
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260422_234500_pg_notify_triggers"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        conn.execute_unprepared(
            r#"
            CREATE OR REPLACE FUNCTION pisa_notify_row() RETURNS trigger
            LANGUAGE plpgsql AS $$
            BEGIN
                PERFORM pg_notify(
                    'pisa_events',
                    json_build_object(
                        'table', TG_TABLE_NAME,
                        'op', lower(TG_OP),
                        'id', COALESCE(NEW.id, OLD.id)
                    )::text
                );
                RETURN COALESCE(NEW, OLD);
            END;
            $$;
            "#,
        )
        .await?;

        for table in ["task", "task_run"] {
            conn.execute_unprepared(&format!(
                r#"
                DROP TRIGGER IF EXISTS {table}_notify ON {table};
                CREATE TRIGGER {table}_notify
                    AFTER INSERT OR UPDATE OR DELETE ON {table}
                    FOR EACH ROW EXECUTE FUNCTION pisa_notify_row();
                "#
            ))
            .await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();
        for table in ["task", "task_run"] {
            conn.execute_unprepared(&format!(
                "DROP TRIGGER IF EXISTS {table}_notify ON {table};"
            ))
            .await?;
        }
        conn.execute_unprepared("DROP FUNCTION IF EXISTS pisa_notify_row();")
            .await?;
        Ok(())
    }
}
