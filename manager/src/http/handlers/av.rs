use axum::{Json, extract::State, http::StatusCode};

use crate::{app_state::AppState, db, entity::av, http::dto::av::AvResponse};

pub async fn list_avs(State(state): State<AppState>) -> Result<Json<Vec<AvResponse>>, StatusCode> {
    let avs: Vec<av::Model> = db::av::find_all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(avs.into_iter().map(AvResponse::from).collect()))
}
