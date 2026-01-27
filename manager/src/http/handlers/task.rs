use axum::{Json, extract::State, http::StatusCode};

use crate::{
    app_state::AppState,
    db,
    http::dto::task::{CreateTaskRequest, TaskResponse},
};

pub async fn list_tasks(
    State(state): State<AppState>,
) -> Result<Json<Vec<TaskResponse>>, (StatusCode, &'static str)> {
    let tasks = db::task::find_all(&state.db)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "db error"))?;

    Ok(Json(tasks.into_iter().map(TaskResponse::from).collect()))
}

pub async fn create_task(
    State(state): State<AppState>,
    Json(payload): Json<CreateTaskRequest>,
) -> Result<Json<TaskResponse>, (StatusCode, &'static str)> {
    // Validate foreign keys first
    if !db::plan::plan_exists(&state.db, payload.plan_id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "db error"))?
    {
        return Err((StatusCode::BAD_REQUEST, "Plan does not exist"));
    }

    if !db::av::av_exists(&state.db, payload.av_id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "db error"))?
    {
        return Err((StatusCode::BAD_REQUEST, "AV does not exist"));
    }

    // Create task (unassigned)
    let task = db::task::create(&state.db, payload.plan_id, payload.av_id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "db error"))?;

    Ok(Json(TaskResponse::from(task)))
}
