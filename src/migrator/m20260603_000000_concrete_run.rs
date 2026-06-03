use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260603_000000_concrete_run"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();
        conn.execute_unprepared(
            r#"
            CREATE TABLE concrete_run (
                id serial PRIMARY KEY,
                task_id integer NOT NULL REFERENCES task(id) ON DELETE CASCADE,
                task_run_id integer NOT NULL REFERENCES task_run(id) ON DELETE CASCADE,
                concrete_key text NOT NULL,
                status text NOT NULL CHECK (status IN ('finished', 'failed', 'aborted', 'skipped')),
                test_outcome text NOT NULL DEFAULT 'unknown' CHECK (test_outcome IN ('success', 'fail', 'invalid', 'unknown')),
                reason text,
                stop_condition text,
                params jsonb,
                final_sim_time_ms double precision,
                wall_time_ms double precision,
                total_steps integer,
                created_at timestamptz NOT NULL DEFAULT now()
            );

            CREATE INDEX concrete_run_task_key_idx ON concrete_run(task_id, concrete_key);
            CREATE INDEX concrete_run_task_run_idx ON concrete_run(task_run_id);
            CREATE INDEX concrete_run_task_created_idx ON concrete_run(task_id, created_at DESC);

            GRANT SELECT, INSERT, UPDATE, DELETE ON TABLE concrete_run TO web_anon;
            GRANT USAGE, SELECT ON SEQUENCE concrete_run_id_seq TO web_anon;

            DROP TRIGGER IF EXISTS concrete_run_notify ON concrete_run;
            CREATE TRIGGER concrete_run_notify
                AFTER INSERT OR UPDATE OR DELETE ON concrete_run
                FOR EACH ROW EXECUTE FUNCTION pisa_notify_row();

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
            DROP TRIGGER IF EXISTS concrete_run_notify ON concrete_run;
            DROP TABLE IF EXISTS concrete_run;
            NOTIFY pgrst, 'reload schema';
            "#,
        )
        .await?;
        Ok(())
    }
}
