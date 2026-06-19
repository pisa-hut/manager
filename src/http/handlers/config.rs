//! HTTP handlers for AV / Simulator / Sampler / Monitor config bytes.
//!
//! All four entities expose the same `(GET | PUT | DELETE)
//! /{kind}/{id}/config` shape. The router wires these generic handlers
//! per entity with a turbofish (e.g. `get_config::<av::Model>`), so
//! Swagger still shows discrete endpoint groups while the bodies all
//! dispatch through the `db::ConfigBearing` trait.

use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
};

use crate::app_state::AppState;
use crate::db::ConfigBearing;
use crate::http::AppError;
use crate::http::handlers::bytes::{build_blob_response, sha256_hex};

pub async fn get_config<E: ConfigBearing>(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<axum::response::Response, AppError> {
    let model = E::get_by_id(&state.db, id)
        .await?
        .ok_or_else(|| AppError::not_found(format!("{} not found", E::kind())))?;
    let content = model
        .config_bytes()
        .ok_or_else(|| AppError::not_found("config not set"))?
        .to_vec();
    let sha = model
        .config_sha256()
        .map(str::to_owned)
        .unwrap_or_else(|| sha256_hex(&content));
    Ok(build_blob_response(&headers, content, &sha))
}

pub async fn put_config<E: ConfigBearing>(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    body: Bytes,
) -> Result<StatusCode, AppError> {
    let content = body.to_vec();
    let sha = sha256_hex(&content);
    E::set_config(&state.db, id, content, sha).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_config<E: ConfigBearing>(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<StatusCode, AppError> {
    E::clear_config(&state.db, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
