use axum::{Json, extract::Path, extract::State};
use sea_orm::EntityTrait;

use crate::{
    app_state::AppState,
    db,
    entity::{sea_orm_active_enums::TaskRunStatus, task_run},
    http::{
        dto::concrete_run::{ConcreteRunCreateRequest, ConcreteRunResponse},
        error::AppError,
    },
};

const VALID_STATUSES: [&str; 4] = ["finished", "failed", "aborted", "skipped"];
const VALID_TEST_OUTCOMES: [&str; 4] = ["success", "fail", "invalid", "unknown"];

pub async fn create_concrete_runs(
    State(state): State<AppState>,
    Path(task_run_id): Path<i32>,
    Json(payload): Json<Vec<ConcreteRunCreateRequest>>,
) -> Result<Json<Vec<ConcreteRunResponse>>, AppError> {
    let Some(run) = task_run::Entity::find_by_id(task_run_id)
        .one(&state.db)
        .await?
    else {
        return Err(AppError::not_found(format!("task_run {task_run_id}")));
    };
    if run.task_run_status != TaskRunStatus::Running {
        return Err(AppError::gone(format!(
            "task_run {task_run_id} is not running"
        )));
    }

    let mut rows = Vec::with_capacity(payload.len());
    for item in payload {
        validate_item(&item)?;
        rows.push(db::concrete_run::NewConcreteRun {
            concrete_key: item.concrete_key,
            status: item.status,
            test_outcome: item.test_outcome,
            reason: item.reason,
            stop_condition: item.stop_condition,
            params: item.params,
            final_sim_time_ms: item.final_sim_time_ms,
            wall_time_ms: item.wall_time_ms,
            total_steps: item.total_steps,
        });
    }

    let inserted = db::concrete_run::insert_many(&state.db, run.task_id, run.id, rows).await?;
    Ok(Json(
        inserted
            .into_iter()
            .map(ConcreteRunResponse::from)
            .collect(),
    ))
}

fn validate_item(item: &ConcreteRunCreateRequest) -> Result<(), AppError> {
    if item.concrete_key.trim().is_empty() {
        return Err(AppError::bad_request("concrete_key must not be empty"));
    }
    if !VALID_STATUSES.contains(&item.status.as_str()) {
        return Err(AppError::bad_request(format!(
            "invalid concrete status: {}",
            item.status
        )));
    }
    if !VALID_TEST_OUTCOMES.contains(&item.test_outcome.as_str()) {
        return Err(AppError::bad_request(format!(
            "invalid concrete test_outcome: {}",
            item.test_outcome
        )));
    }
    if matches!(item.final_sim_time_ms, Some(n) if !n.is_finite() || n < 0.0) {
        return Err(AppError::bad_request(
            "final_sim_time_ms must be finite and >= 0",
        ));
    }
    if matches!(item.wall_time_ms, Some(n) if !n.is_finite() || n < 0.0) {
        return Err(AppError::bad_request(
            "wall_time_ms must be finite and >= 0",
        ));
    }
    if matches!(item.total_steps, Some(n) if n < 0) {
        return Err(AppError::bad_request("total_steps must be >= 0"));
    }
    Ok(())
}
