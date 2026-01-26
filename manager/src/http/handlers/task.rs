use axum::{Json, extract::State};

use crate::{
    app_state::AppState,
    http::dto::task::{
        CompleteTaskRequest, CompleteTaskResponse, LeaseTaskRequest, LeaseTaskResponse,
    },
};

pub async fn lease_task(
    State(state): State<AppState>,
    Json(req): Json<LeaseTaskRequest>,
) -> Json<LeaseTaskResponse> {
    // TEMP: you will replace this with SQL leasing
    let _pool = &state.db;

    Json(LeaseTaskResponse {
        task_id: "t_123".to_string(),
        task_type: "example".to_string(),
        payload: serde_json::json!({
            "leased_to": req.worker_id
        }),
        lease_seconds: 60,
    })
}

pub async fn complete_task(
    State(state): State<AppState>,
    Json(req): Json<CompleteTaskRequest>,
) -> Json<CompleteTaskResponse> {
    let _pool = &state.db;

    // TODO: update task row
    println!("complete from worker {} => {:?}", req.worker_id, req.result);

    Json(CompleteTaskResponse { ok: true })
}
