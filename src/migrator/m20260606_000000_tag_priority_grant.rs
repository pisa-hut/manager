use sea_orm_migration::prelude::*;

/// Grant `web_anon` access to `tag_priority`.
///
/// `tag_priority` was created in `m20260605_000000_tag_priority` *after*
/// `m20260312_163958_postgrest_permission` ran its one-shot
/// `GRANT ... ON ALL TABLES IN SCHEMA public TO web_anon`. That grant only
/// covers tables existing at the time, so the new table inherited nothing.
///
/// The `task_set_initial_priority` BEFORE INSERT trigger runs as the
/// invoking role (it is not SECURITY DEFINER), so a PostgREST insert as
/// `web_anon` calls `task_effective_priority`, which JOINs `tag_priority`
/// and fails with "permission denied for table tag_priority" — breaking
/// every task create. Grant the same CRUD set the other tables already have.
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260606_000000_tag_priority_grant"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                GRANT SELECT, INSERT, UPDATE, DELETE ON TABLE tag_priority TO web_anon;
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
                REVOKE SELECT, INSERT, UPDATE, DELETE ON TABLE tag_priority FROM web_anon;
                NOTIFY pgrst, 'reload schema';
                "#,
            )
            .await?;
        Ok(())
    }
}
