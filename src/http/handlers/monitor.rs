use axum::{Json, extract::State};

use crate::app_state::AppState;
use crate::db;
use crate::http::AppError;
use crate::http::dto::monitor::{CreateMonitorRequest, MonitorResponse};

pub async fn list_monitors(
    State(state): State<AppState>,
) -> Result<Json<Vec<MonitorResponse>>, AppError> {
    let monitors = db::monitor::find_all(&state.db).await?;
    Ok(Json(
        monitors.into_iter().map(MonitorResponse::from).collect(),
    ))
}

pub async fn create_monitor(
    State(state): State<AppState>,
    Json(payload): Json<CreateMonitorRequest>,
) -> Result<Json<MonitorResponse>, AppError> {
    let monitor_model = db::monitor::create(&state.db, payload.name, payload.module_path).await?;
    Ok(Json(MonitorResponse::from(monitor_model)))
}
