use sea_orm_migration::prelude::*;

/// Make `task.monitor_id` required.
///
/// The previous migration (m20260507) added the column as nullable
/// so existing tasks survived the schema change and the executor
/// could fall back to a bundled DEFAULT_MONITOR_YAML. Now that
/// monitors are first-class and editable on the web, every task
/// must point at one — there's no implicit "default" any more.
///
/// Steps (in order, single transaction):
///   1. Insert a `default` monitor row if the table is empty,
///      so a fresh install can complete the migration without
///      the operator having to seed by hand. Idempotent: skipped
///      if any monitor already exists.
///   2. Backfill any task with NULL monitor_id to the lowest
///      monitor id (the seeded `default` for fresh installs, or
///      whatever monitor the operator created first).
///   3. ALTER COLUMN monitor_id SET NOT NULL.
///   4. Replace the FK action: ON DELETE SET NULL would now
///      violate the NOT NULL constraint, so swap to NO ACTION
///      (RESTRICT). Deleting a monitor that any task references
///      will fail loudly instead of corrupting rows.
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260513_000000_monitor_required"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // 1. Seed a default monitor only if none exists.
        conn.execute_unprepared(
            r#"
            INSERT INTO monitor (name, module_path)
            SELECT 'default', 'simcore.monitor.base:Monitor'
            WHERE NOT EXISTS (SELECT 1 FROM monitor);
            "#,
        )
        .await?;

        // 2. Backfill NULLs to the smallest monitor id.
        conn.execute_unprepared(
            r#"
            UPDATE task
               SET monitor_id = (SELECT MIN(id) FROM monitor)
             WHERE monitor_id IS NULL;
            "#,
        )
        .await?;

        // 3. Tighten the column.
        manager
            .alter_table(
                Table::alter()
                    .table(Task::Table)
                    .modify_column(ColumnDef::new(Task::MonitorId).integer().not_null())
                    .to_owned(),
            )
            .await?;

        // 4. Swap the FK action: SET NULL → NO ACTION (RESTRICT).
        //    Drop+recreate is the portable form across sea-orm
        //    backends; ALTER CONSTRAINT isn't supported on every
        //    db.
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .table(Task::Table)
                    .name("fk_task_monitor")
                    .to_owned(),
            )
            .await?;
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_task_monitor")
                    .from(Task::Table, Task::MonitorId)
                    .to(Monitor::Table, Monitor::Id)
                    .on_delete(ForeignKeyAction::NoAction)
                    .on_update(ForeignKeyAction::NoAction)
                    .to_owned(),
            )
            .await?;

        // PostgREST schema cache must catch the changed column
        // metadata so existing clients see the new constraint.
        conn.execute_unprepared("NOTIFY pgrst, 'reload schema';")
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Reverse only what we tightened — leave the seeded
        // monitor row + backfilled monitor_ids in place so a
        // re-up doesn't double-insert and existing tasks keep
        // working.
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .table(Task::Table)
                    .name("fk_task_monitor")
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
        manager
            .alter_table(
                Table::alter()
                    .table(Task::Table)
                    .modify_column(ColumnDef::new(Task::MonitorId).integer().null())
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Task {
    Table,
    MonitorId,
}

#[derive(DeriveIden)]
enum Monitor {
    Table,
    Id,
}
