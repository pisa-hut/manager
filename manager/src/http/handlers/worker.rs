use axum::{Json, extract::State, http::StatusCode};

use crate::{
    app_state::AppState,
    db,
    http::dto::worker::{CreateWorkerRequest, WorkerResponse},
};

pub async fn list_workers(
    State(state): State<AppState>,
) -> Result<Json<Vec<WorkerResponse>>, StatusCode> {
    let workers = db::worker::find_all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(
        workers.into_iter().map(WorkerResponse::from).collect(),
    ))
}

pub async fn create_worker(
    State(state): State<AppState>,
    Json(payload): Json<CreateWorkerRequest>,
) -> Result<Json<WorkerResponse>, StatusCode> {
    let worker = db::worker::create(
        &state.db,
        payload.job_id,
        payload.node_list,
        payload.hostname,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(WorkerResponse::from(worker)))
}
