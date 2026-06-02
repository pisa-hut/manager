use sea_orm_migration::prelude::*;

/// Drop `sampler.module_path` and `monitor.module_path`.
///
/// The runner now resolves both sampler and monitor implementations from
/// their `name` alone (via simcore's built-in registries). `module_path`
/// only ever held the canonical class for the given name and was never
/// edited per-row, so the column was dead weight. Drop it and let
/// PostgREST re-read its schema cache so the field stops appearing on
/// GETs.
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260602_000000_drop_sampler_monitor_module_path"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Sampler::Table)
                    .drop_column(Sampler::ModulePath)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Monitor::Table)
                    .drop_column(Monitor::ModulePath)
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared("NOTIFY pgrst, 'reload schema'")
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Re-create as nullable text so rollback is non-lossy for any
        // rows inserted after the drop. The original schema had it
        // NOT NULL, but we can't synthesize a value for new rows.
        manager
            .alter_table(
                Table::alter()
                    .table(Sampler::Table)
                    .add_column(ColumnDef::new(Sampler::ModulePath).string().null())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Monitor::Table)
                    .add_column(ColumnDef::new(Monitor::ModulePath).string().null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Sampler {
    Table,
    ModulePath,
}

#[derive(DeriveIden)]
enum Monitor {
    Table,
    ModulePath,
}
