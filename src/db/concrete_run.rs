use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection, DbErr};
use serde_json::Value;

use crate::entity::concrete_run;

pub struct NewConcreteRun {
    pub concrete_key: String,
    pub status: String,
    pub test_outcome: String,
    pub reason: Option<String>,
    pub stop_condition: Option<String>,
    pub params: Option<Value>,
    pub final_sim_time_ms: Option<f64>,
    pub wall_time_ms: Option<f64>,
    pub total_steps: Option<i32>,
}

pub async fn insert_many(
    db: &DatabaseConnection,
    task_id: i32,
    task_run_id: i32,
    rows: Vec<NewConcreteRun>,
) -> Result<Vec<concrete_run::Model>, DbErr> {
    let mut inserted = Vec::with_capacity(rows.len());
    for row in rows {
        let active = concrete_run::ActiveModel {
            task_id: Set(task_id),
            task_run_id: Set(task_run_id),
            concrete_key: Set(row.concrete_key),
            status: Set(row.status),
            test_outcome: Set(row.test_outcome),
            reason: Set(row.reason),
            stop_condition: Set(row.stop_condition),
            params: Set(row.params),
            final_sim_time_ms: Set(row.final_sim_time_ms),
            wall_time_ms: Set(row.wall_time_ms),
            total_steps: Set(row.total_steps),
            ..Default::default()
        };
        inserted.push(active.insert(db).await?);
    }
    Ok(inserted)
}
