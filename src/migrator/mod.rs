// src/migrator/mod.rs (create new file)

use sea_orm_migration::prelude::*;

mod m20260305_155925_new_db_schema;
mod m20260312_163958_postgrest_permission;
mod m20260422_120000_file_bytes;
mod m20260422_180000_drop_path_columns;
mod m20260422_200000_task_run_log;
mod m20260422_220000_normalize_goal_config;
mod m20260422_230000_drop_goal_config;
mod m20260422_234500_pg_notify_triggers;
mod m20260423_000000_task_run_heartbeat;
mod m20260424_000000_task_run_concrete_runs;
mod m20260425_000000_task_status_rename;
mod m20260425_010000_drop_dead_statuses;
mod m20260425_020000_task_archived;
mod m20260503_000000_drop_executor_slurm_array_id;
mod m20260507_000000_monitor;
mod m20260513_000000_monitor_required;
mod m20260514_000000_plan_tags;
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260305_155925_new_db_schema::Migration),
            Box::new(m20260312_163958_postgrest_permission::Migration),
            Box::new(m20260422_120000_file_bytes::Migration),
            Box::new(m20260422_180000_drop_path_columns::Migration),
            Box::new(m20260422_200000_task_run_log::Migration),
            Box::new(m20260422_220000_normalize_goal_config::Migration),
            Box::new(m20260422_230000_drop_goal_config::Migration),
            Box::new(m20260422_234500_pg_notify_triggers::Migration),
            Box::new(m20260423_000000_task_run_heartbeat::Migration),
            Box::new(m20260424_000000_task_run_concrete_runs::Migration),
            Box::new(m20260425_000000_task_status_rename::Migration),
            Box::new(m20260425_010000_drop_dead_statuses::Migration),
            Box::new(m20260425_020000_task_archived::Migration),
            Box::new(m20260503_000000_drop_executor_slurm_array_id::Migration),
            Box::new(m20260507_000000_monitor::Migration),
            Box::new(m20260513_000000_monitor_required::Migration),
            Box::new(m20260514_000000_plan_tags::Migration),
        ]
    }
}
