use axum::{
    Json,
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
};

use crate::app_state::AppState;
use crate::db;
use crate::http::dto::scenario_file::ScenarioFileMetaResponse;
use crate::http::handlers::bytes::{build_blob_response, sha256_hex};

pub async fn list_files(
    State(state): State<AppState>,
    Path(scenario_id): Path<i32>,
) -> Result<Json<Vec<ScenarioFileMetaResponse>>, StatusCode> {
    let files = db::scenario_file::find_by_scenario(&state.db, scenario_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(files.into_iter().map(Into::into).collect()))
}

pub async fn get_file(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path((scenario_id, relative_path)): Path<(i32, String)>,
) -> Result<axum::response::Response, (StatusCode, String)> {
    reject_traversal(&relative_path)?;

    let file = db::scenario_file::get(&state.db, scenario_id, &relative_path)
        .await
        .map_err(internal_error)?
        .ok_or((StatusCode::NOT_FOUND, "file not found".to_string()))?;

    Ok(build_blob_response(
        &headers,
        file.content,
        &file.content_sha256,
    ))
}

pub async fn put_file(
    State(state): State<AppState>,
    Path((scenario_id, relative_path)): Path<(i32, String)>,
    body: Bytes,
) -> Result<Json<ScenarioFileMetaResponse>, (StatusCode, String)> {
    reject_traversal(&relative_path)?;

    let content = body.to_vec();
    let sha = sha256_hex(&content);
    let model = db::scenario_file::put(&state.db, scenario_id, relative_path, content, sha)
        .await
        .map_err(internal_error)?;
    Ok(Json(model.into()))
}

pub async fn delete_file(
    State(state): State<AppState>,
    Path((scenario_id, relative_path)): Path<(i32, String)>,
) -> Result<StatusCode, (StatusCode, String)> {
    reject_traversal(&relative_path)?;

    let deleted = db::scenario_file::delete(&state.db, scenario_id, &relative_path)
        .await
        .map_err(internal_error)?;
    if deleted == 0 {
        return Err((StatusCode::NOT_FOUND, "file not found".to_string()));
    }
    Ok(StatusCode::NO_CONTENT)
}

fn reject_traversal(p: &str) -> Result<(), (StatusCode, String)> {
    if p.contains("..") || p.starts_with('/') {
        return Err((StatusCode::BAD_REQUEST, "invalid path".to_string()));
    }
    Ok(())
}

fn internal_error<E: std::fmt::Display>(e: E) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}
