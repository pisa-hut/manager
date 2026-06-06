use axum::{Json, extract::State};
use sea_orm::{
    ConnectionTrait, DatabaseBackend, FromQueryResult, Statement, TransactionTrait, Value,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::app_state::AppState;
use crate::http::AppError;

#[derive(Debug, Serialize, FromQueryResult)]
pub struct TagPriorityRow {
    pub tag: String,
    pub position: i32,
}

#[derive(Debug, Deserialize)]
pub struct SetTagPriorityRequest {
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct TagPriorityMutationResponse {
    pub count: usize,
}

pub async fn list_tag_priority(
    State(state): State<AppState>,
) -> Result<Json<Vec<TagPriorityRow>>, AppError> {
    let stmt = Statement::from_string(
        DatabaseBackend::Postgres,
        "SELECT tag, position FROM tag_priority ORDER BY position ASC",
    );
    let rows = TagPriorityRow::find_by_statement(stmt)
        .all(&state.db)
        .await?;
    Ok(Json(rows))
}

/// Replace the entire ranking. `tags[i]` gets position `i` (0 = highest).
/// DELETE + single INSERT keep the per-statement recompute trigger firings
/// to two, regardless of list length.
pub async fn set_tag_priority(
    State(state): State<AppState>,
    Json(payload): Json<SetTagPriorityRequest>,
) -> Result<Json<TagPriorityMutationResponse>, AppError> {
    let mut seen = HashSet::new();
    for tag in &payload.tags {
        if tag.trim().is_empty() {
            return Err(AppError::bad_request("tag names must be non-empty"));
        }
        if !seen.insert(tag.as_str()) {
            return Err(AppError::bad_request(format!("duplicate tag: {tag}")));
        }
    }

    let tags_json =
        serde_json::to_string(&payload.tags).map_err(|e| AppError::bad_request(e.to_string()))?;

    let txn = state.db.begin().await?;
    txn.execute(Statement::from_string(
        DatabaseBackend::Postgres,
        "DELETE FROM tag_priority",
    ))
    .await?;
    txn.execute(Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        r#"
        INSERT INTO tag_priority (tag, position)
        SELECT elem, (ord - 1)::int
        FROM jsonb_array_elements_text($1::jsonb) WITH ORDINALITY AS e(elem, ord)
        "#,
        [Value::String(Some(Box::new(tags_json)))],
    ))
    .await?;
    txn.commit().await?;

    Ok(Json(TagPriorityMutationResponse {
        count: payload.tags.len(),
    }))
}
