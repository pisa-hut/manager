use axum::{Json, extract::State};
use sea_orm::{ConnectionTrait, DatabaseBackend, FromQueryResult, Statement, Value};
use serde::{Deserialize, Serialize};

use crate::app_state::AppState;
use crate::db;
use crate::http::AppError;
use crate::http::dto::plan::{CreatePlanRequest, PlanResponse};

pub async fn list_plans(
    State(state): State<AppState>,
) -> Result<Json<Vec<PlanResponse>>, AppError> {
    let plans = db::plan::find_all(&state.db).await?;
    Ok(Json(plans.into_iter().map(PlanResponse::from).collect()))
}

pub async fn create_plan(
    State(state): State<AppState>,
    Json(payload): Json<CreatePlanRequest>,
) -> Result<Json<PlanResponse>, AppError> {
    if !db::map::map_exists(&state.db, payload.map_id).await? {
        return Err(AppError::bad_request(format!(
            "map {} does not exist",
            payload.map_id
        )));
    }
    if !db::scenario::scenario_exists(&state.db, payload.scenario_id).await? {
        return Err(AppError::bad_request(format!(
            "scenario {} does not exist",
            payload.scenario_id
        )));
    }
    let plan = db::plan::create(
        &state.db,
        payload.name,
        payload.map_id,
        payload.scenario_id,
        payload.tags,
    )
    .await?;
    Ok(Json(PlanResponse::from(plan)))
}

// ---------------------------------------------------------------------------
// Tag management
//
// Tags live inline on `plan.tags text[]`; there is no separate `tag` table.
// "Manage" here means: aggregate distinct names + counts, bulk-remove a name
// from every plan that has it, or bulk-rename across the whole table. These
// are admin-style operations driven by the frontend's tag manager modal.
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, FromQueryResult)]
pub struct TagCount {
    pub name: String,
    pub count: i64,
}

#[derive(Debug, Deserialize)]
pub struct RemoveTagRequest {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct RenameTagRequest {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Serialize)]
pub struct TagMutationResponse {
    pub plans_updated: u64,
}

pub async fn list_plan_tags(
    State(state): State<AppState>,
) -> Result<Json<Vec<TagCount>>, AppError> {
    let stmt = Statement::from_string(
        DatabaseBackend::Postgres,
        r#"
        SELECT name, COUNT(*)::bigint AS count
        FROM (SELECT unnest(tags) AS name FROM plan) t
        GROUP BY name
        ORDER BY count DESC, name ASC
        "#,
    );
    let rows = TagCount::find_by_statement(stmt).all(&state.db).await?;
    Ok(Json(rows))
}

pub async fn remove_plan_tag(
    State(state): State<AppState>,
    Json(payload): Json<RemoveTagRequest>,
) -> Result<Json<TagMutationResponse>, AppError> {
    if payload.name.is_empty() {
        return Err(AppError::bad_request("name is required"));
    }
    let stmt = Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        "UPDATE plan SET tags = array_remove(tags, $1) WHERE $1 = ANY(tags)",
        [Value::String(Some(Box::new(payload.name)))],
    );
    let res = state.db.execute(stmt).await?;
    Ok(Json(TagMutationResponse {
        plans_updated: res.rows_affected(),
    }))
}

pub async fn rename_plan_tag(
    State(state): State<AppState>,
    Json(payload): Json<RenameTagRequest>,
) -> Result<Json<TagMutationResponse>, AppError> {
    if payload.from.is_empty() || payload.to.is_empty() {
        return Err(AppError::bad_request("from and to are required"));
    }
    if payload.from == payload.to {
        return Err(AppError::bad_request("from and to are identical"));
    }
    // ARRAY(SELECT DISTINCT …) collapses any duplicate that would result
    // when a plan already carries both the old and new tag names, so the
    // rename can't create `{foo, foo}`.
    let stmt = Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        r#"
        UPDATE plan
        SET tags = ARRAY(
            SELECT DISTINCT unnest(array_replace(tags, $1, $2))
            ORDER BY 1
        )
        WHERE $1 = ANY(tags)
        "#,
        [
            Value::String(Some(Box::new(payload.from))),
            Value::String(Some(Box::new(payload.to))),
        ],
    );
    let res = state.db.execute(stmt).await?;
    Ok(Json(TagMutationResponse {
        plans_updated: res.rows_affected(),
    }))
}
