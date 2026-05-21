use axum::{Json, extract::State};
use sea_orm::{DatabaseBackend, FromQueryResult, Statement};
use serde::Serialize;

use crate::app_state::AppState;
use crate::http::AppError;

/// One row per (av_id, simulator_id) bucket with at least one queued
/// task. The scheduler reads this to decide how many SLURM jobs to
/// submit per bucket and how to size each one.
///
/// Resources are the **sum** of the AV and Simulator hints (both run
/// as concurrent containers inside the same SLURM allocation). The
/// scheduler clamps each value to 1 at submit time so a forgotten
/// zero can't queue a zero-CPU job.
#[derive(Debug, Serialize, FromQueryResult)]
pub struct DemandBucket {
    pub av_id: i32,
    pub simulator_id: i32,
    pub queued: i64,
    pub cpu_count: i64,
    pub memory_gb: i64,
    pub gpu_count: i64,
    /// Oldest queued task's id in the bucket — lets the scheduler
    /// prioritise older work when slots are tight.
    pub oldest_task_id: i32,
}

pub async fn queue_demand(
    State(state): State<AppState>,
) -> Result<Json<Vec<DemandBucket>>, AppError> {
    let stmt = Statement::from_string(
        DatabaseBackend::Postgres,
        r#"
        SELECT
            t.av_id,
            t.simulator_id,
            COUNT(*)::bigint                AS queued,
            (av.cpu_count + sim.cpu_count)::bigint AS cpu_count,
            (av.memory_gb + sim.memory_gb)::bigint AS memory_gb,
            (av.gpu_count + sim.gpu_count)::bigint AS gpu_count,
            MIN(t.id)                       AS oldest_task_id
        FROM task t
        JOIN av  ON av.id  = t.av_id
        JOIN simulator sim ON sim.id = t.simulator_id
        WHERE t.task_status = 'queued'
        GROUP BY t.av_id, t.simulator_id, av.cpu_count, av.memory_gb, av.gpu_count,
                 sim.cpu_count, sim.memory_gb, sim.gpu_count
        ORDER BY MIN(t.id) ASC
        "#,
    );
    let rows = DemandBucket::find_by_statement(stmt).all(&state.db).await?;
    Ok(Json(rows))
}
