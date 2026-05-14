use sea_orm_migration::prelude::*;

/// Add `plan.tags` for grouping plans without a separate `tag`
/// entity. The web UI lets the user attach free-form labels (e.g.
/// "sprint-2026-05", "carla-only") and filter by them in the
/// bulk-create-task modal so a batch of related plans is one
/// click away from a cartesian.
///
/// `text[]` matches Postgres convention; default `'{}'` keeps
/// existing rows valid without a backfill.
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260514_000000_plan_tags"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        conn.execute_unprepared(
            r#"
            ALTER TABLE plan
                ADD COLUMN tags text[] NOT NULL DEFAULT '{}';
            NOTIFY pgrst, 'reload schema';
            "#,
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Plan::Table)
                    .drop_column(Plan::Tags)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Plan {
    Table,
    Tags,
}
