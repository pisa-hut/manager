use axum::{
    body::Bytes,
    extract::{Path, State},
    http::StatusCode,
};
use serde_json::json;

use crate::{app_state::AppState, db};

/// Append a stdout/stderr chunk to a task_run row. Called by the executor
/// every ~1 s (or whenever its log buffer has fresh bytes) while the run
/// is in progress.
///
/// Two side effects: update `task_run.log` via an append-only SQL (no
/// racing with lifecycle calls), and broadcast a `log` SSE envelope so
/// the Log Drawer can stream chunks into the UI.
pub async fn append_log(
    State(state): State<AppState>,
    Path(run_id): Path<i32>,
    body: Bytes,
) -> Result<StatusCode, (StatusCode, String)> {
    if body.is_empty() {
        return Ok(StatusCode::NO_CONTENT);
    }
    let chunk = std::str::from_utf8(&body)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("utf-8 required: {e}")))?;
    db::task_run::append_log(&state.db, run_id, chunk)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let envelope = json!({
        "kind": "log",
        "task_run_id": run_id,
        "chunk": chunk,
    });
    let _ = state.events_tx.send(envelope.to_string());

    Ok(StatusCode::NO_CONTENT)
}
