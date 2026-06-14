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
mod m20260516_000000_task_last_run_at;
mod m20260521_000000_av_sim_resources;
mod m20260601_000000_task_run_concrete_run_split;
mod m20260602_000000_drop_sampler_monitor_module_path;
mod m20260603_000000_concrete_run;
mod m20260604_000000_task_queue_priority;
mod m20260605_000000_tag_priority;
mod m20260606_000000_tag_priority_grant;
mod m20260616_000000_task_run_expected;
mod m20260617_000000_concrete_run_unique;
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
            Box::new(m20260516_000000_task_last_run_at::Migration),
            Box::new(m20260521_000000_av_sim_resources::Migration),
            Box::new(m20260601_000000_task_run_concrete_run_split::Migration),
            Box::new(m20260602_000000_drop_sampler_monitor_module_path::Migration),
            Box::new(m20260603_000000_concrete_run::Migration),
            Box::new(m20260604_000000_task_queue_priority::Migration),
            Box::new(m20260605_000000_tag_priority::Migration),
            Box::new(m20260606_000000_tag_priority_grant::Migration),
            Box::new(m20260616_000000_task_run_expected::Migration),
            Box::new(m20260617_000000_concrete_run_unique::Migration),
        ]
    }
}
