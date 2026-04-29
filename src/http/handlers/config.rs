//! HTTP handlers for AV / Simulator / Sampler config bytes.

use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
};
use sea_orm::DbErr;

use crate::app_state::AppState;
use crate::db;
use crate::http::handlers::bytes::{build_blob_response, sha256_hex};

pub async fn get_av_config(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(av_id): Path<i32>,
) -> Result<axum::response::Response, (StatusCode, String)> {
    let av = db::av::get_by_id(&state.db, av_id)
        .await
        .map_err(internal_error)?
        .ok_or((StatusCode::NOT_FOUND, "av not found".to_string()))?;
    let content = av
        .config
        .ok_or((StatusCode::NOT_FOUND, "config not set".to_string()))?;
    let sha = av.config_sha256.unwrap_or_else(|| sha256_hex(&content));
    Ok(build_blob_response(&headers, content, &sha))
}

pub async fn put_av_config(
    State(state): State<AppState>,
    Path(av_id): Path<i32>,
    body: Bytes,
) -> Result<StatusCode, (StatusCode, String)> {
    let content = body.to_vec();
    let sha = sha256_hex(&content);
    db::av::set_config(&state.db, av_id, content, sha)
        .await
        .map_err(internal_error)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_av_config(
    State(state): State<AppState>,
    Path(av_id): Path<i32>,
) -> Result<StatusCode, (StatusCode, String)> {
    db::av::clear_config(&state.db, av_id)
        .await
        .map_err(internal_error)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_simulator_config(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(sim_id): Path<i32>,
) -> Result<axum::response::Response, (StatusCode, String)> {
    let sim = db::simulator::get_by_id(&state.db, sim_id)
        .await
        .map_err(internal_error)?
        .ok_or((StatusCode::NOT_FOUND, "simulator not found".to_string()))?;
    let content = sim
        .config
        .ok_or((StatusCode::NOT_FOUND, "config not set".to_string()))?;
    let sha = sim.config_sha256.unwrap_or_else(|| sha256_hex(&content));
    Ok(build_blob_response(&headers, content, &sha))
}

pub async fn put_simulator_config(
    State(state): State<AppState>,
    Path(sim_id): Path<i32>,
    body: Bytes,
) -> Result<StatusCode, (StatusCode, String)> {
    let content = body.to_vec();
    let sha = sha256_hex(&content);
    db::simulator::set_config(&state.db, sim_id, content, sha)
        .await
        .map_err(internal_error)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_simulator_config(
    State(state): State<AppState>,
    Path(sim_id): Path<i32>,
) -> Result<StatusCode, (StatusCode, String)> {
    db::simulator::clear_config(&state.db, sim_id)
        .await
        .map_err(internal_error)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_sampler_config(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(sampler_id): Path<i32>,
) -> Result<axum::response::Response, (StatusCode, String)> {
    let sampler = db::sampler::get_by_id(&state.db, sampler_id)
        .await
        .map_err(internal_error)?
        .ok_or((StatusCode::NOT_FOUND, "sampler not found".to_string()))?;
    let content = sampler
        .config
        .ok_or((StatusCode::NOT_FOUND, "config not set".to_string()))?;
    let sha = sampler
        .config_sha256
        .unwrap_or_else(|| sha256_hex(&content));
    Ok(build_blob_response(&headers, content, &sha))
}

pub async fn put_sampler_config(
    State(state): State<AppState>,
    Path(sampler_id): Path<i32>,
    body: Bytes,
) -> Result<StatusCode, (StatusCode, String)> {
    let content = body.to_vec();
    let sha = sha256_hex(&content);
    db::sampler::set_config(&state.db, sampler_id, content, sha)
        .await
        .map_err(internal_error)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_sampler_config(
    State(state): State<AppState>,
    Path(sampler_id): Path<i32>,
) -> Result<StatusCode, (StatusCode, String)> {
    db::sampler::clear_config(&state.db, sampler_id)
        .await
        .map_err(internal_error)?;
    Ok(StatusCode::NO_CONTENT)
}

/// Map a SeaORM error into the right HTTP status. The `set_config` /
/// `clear_config` helpers return `RecordNotFound` when the parent
/// AV/Simulator/Sampler row doesn't exist; surface that as 404 so
/// clients can distinguish missing resources from real DB errors.
fn internal_error(e: DbErr) -> (StatusCode, String) {
    match e {
        DbErr::RecordNotFound(msg) => (StatusCode::NOT_FOUND, msg),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}
