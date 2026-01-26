use axum::Json;

use crate::http::dto::task::{
    CompleteTaskRequest, CompleteTaskResponse, LeaseTaskRequest, LeaseTaskResponse,
};

pub async fn lease_task(Json(req): Json<LeaseTaskRequest>) -> Json<LeaseTaskResponse> {
    // TODO: replace with real DB leasing logic
    Json(LeaseTaskResponse {
        task_id: "t_123".to_string(),
        task_type: "example".to_string(),
        payload: serde_json::json!({
            "message": "run this task",
            "leased_to": req.worker_id
        }),
        lease_seconds: 60,
    })
}

pub async fn complete_task(Json(req): Json<CompleteTaskRequest>) -> Json<CompleteTaskResponse> {
    // TODO: update task in DB
    println!(
        "worker {} completed task with result: {:?}",
        req.worker_id, req.result
    );

    Json(CompleteTaskResponse { ok: true })
}
