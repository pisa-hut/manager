use crate::entity::sea_orm_active_enums::TaskRunStatus;
use crate::entity::task_run;
use chrono::Utc;
use sea_orm::*;

pub async fn find_all(db: &DatabaseConnection) -> Result<Vec<task_run::Model>, DbErr> {
    task_run::Entity::find().all(db).await
}

pub async fn create(
    db: &DatabaseConnection,
    task_id: i32,
    executor_id: i32,
    attempt: i32,
) -> Result<task_run::Model, DbErr> {
    let active = task_run::ActiveModel {
        task_id: Set(task_id),
        executor_id: Set(executor_id),
        attempt: Set(attempt),
        task_run_status: Set(TaskRunStatus::Running),
        started_at: Set(Some(Utc::now().fixed_offset())),
        ..Default::default()
    };
    active.insert(db).await
}
