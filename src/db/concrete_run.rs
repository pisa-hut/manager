use sea_orm::{ActiveValue::Set, DatabaseConnection, DbErr, EntityTrait, sea_query::OnConflict};
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
    // Idempotent on (task_run_id, concrete_key): the executor sends each
    // concrete incrementally as it finalises and again in the terminal
    // reconcile batch. DO NOTHING keeps the first (live) row — so the
    // reconcile only fills in any that failed to send — and a conflict
    // surfaces as RecordNotInserted, which we skip.
    let on_conflict = OnConflict::columns([
        concrete_run::Column::TaskRunId,
        concrete_run::Column::ConcreteKey,
    ])
    .do_nothing()
    .to_owned();

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
        // exec() (not exec_with_returning) gives clean DO NOTHING semantics:
        // a conflict returns RecordNotInserted; a real insert returns the new
        // id, which we read back for the response.
        match concrete_run::Entity::insert(active)
            .on_conflict(on_conflict.clone())
            .exec(db)
            .await
        {
            Ok(res) => {
                if let Some(model) = concrete_run::Entity::find_by_id(res.last_insert_id)
                    .one(db)
                    .await?
                {
                    inserted.push(model);
                }
            }
            Err(DbErr::RecordNotInserted) => {}
            Err(e) => return Err(e),
        }
    }
    Ok(inserted)
}
