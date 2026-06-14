use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};

use crate::{
    app_state::AppState,
    db,
    http::{dto::task_run::TaskRunProgressRequest, error::AppError},
};

/// Apply a live mid-run progress snapshot to a task_run. Called by the
/// executor after each concrete completes so the UI can show ongoing
/// progress instead of a bar that only fills at task end.
///
/// Returns 410 Gone once the run has been finalised (same contract as
/// `/log/append`); the executor swallows that and stops reporting.
pub async fn update_progress(
    State(state): State<AppState>,
    Path(run_id): Path<i32>,
    Json(payload): Json<TaskRunProgressRequest>,
) -> Result<StatusCode, AppError> {
    let updated = db::task_run::update_progress(
        &state.db,
        run_id,
        payload.finished_concrete_runs,
        payload.aborted_concrete_runs,
        payload.skipped_concrete_runs,
        payload.expected_concrete_runs,
    )
    .await?;
    if !updated {
        return Err(AppError::gone(format!("task_run {run_id} is not running")));
    }
    Ok(StatusCode::NO_CONTENT)
}
