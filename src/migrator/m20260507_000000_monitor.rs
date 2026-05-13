use sea_orm_migration::prelude::*;

/// Add a `monitor` entity (mirror of `sampler`'s shape — id, name,
/// module_path, nullable config bytes + sha256) and an optional
/// `task.monitor_id` FK pointing at it.
///
/// The monitor decides when to stop a scenario early (timeout,
/// custom condition tree). Until this migration it lived only as a
/// hardcoded YAML in the executor; making it a first-class entity
/// matches the AV / Simulator / Sampler pattern and lets the web UI
/// edit it. `task.monitor_id` is nullable so existing rows stay
/// valid; the executor falls back to its bundled default when
/// `monitor_id` is unset.
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260507_000000_monitor"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Monitor::Table)
                    .col(
                        ColumnDef::new(Monitor::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Monitor::Name).string().not_null())
                    .col(ColumnDef::new(Monitor::ModulePath).string().not_null())
                    .col(ColumnDef::new(Monitor::Config).binary().null())
                    .col(ColumnDef::new(Monitor::ConfigSha256).string().null())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Task::Table)
                    .add_column(ColumnDef::new(Task::MonitorId).integer().null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_task_monitor")
                    .from(Task::Table, Task::MonitorId)
                    .to(Monitor::Table, Monitor::Id)
                    .on_delete(ForeignKeyAction::SetNull)
                    .on_update(ForeignKeyAction::NoAction)
                    .to_owned(),
            )
            .await?;

        // PostgREST grant + sequence usage for the new table. The
        // m20260312 migration grants on ALL TABLES at install time,
        // but new tables created afterward need explicit grants
        // (and the sequence they use for SERIAL ids).
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                GRANT SELECT, INSERT, UPDATE, DELETE ON TABLE monitor TO web_anon;
                GRANT USAGE, SELECT ON SEQUENCE monitor_id_seq TO web_anon;
                NOTIFY pgrst, 'reload schema';
                "#,
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .table(Task::Table)
                    .name("fk_task_monitor")
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Task::Table)
                    .drop_column(Task::MonitorId)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(Monitor::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Monitor {
    Table,
    Id,
    Name,
    ModulePath,
    Config,
    ConfigSha256,
}

#[derive(DeriveIden)]
enum Task {
    Table,
    MonitorId,
}
