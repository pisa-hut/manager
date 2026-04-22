use crate::entity::sea_orm_active_enums::{TaskRunStatus, TaskStatus};
use crate::entity::{task, task_run};
use chrono::{Duration, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseConnection,
    DbBackend, DbErr, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Statement,
    TransactionError, TransactionTrait,
};
use sea_orm_migration::prelude::{LockBehavior, LockType};

/// Append a chunk to `task_run.log` without rewriting the entire column,
/// and bump `last_heartbeat_at` so the reaper treats this run as alive.
pub async fn append_log(
    db: &DatabaseConnection,
    run_id: i32,
    chunk: &str,
) -> Result<(), DbErr> {
    db.execute(Statement::from_sql_and_values(
        DbBackend::Postgres,
        r#"UPDATE task_run
           SET log = COALESCE(log, '') || $1,
               last_heartbeat_at = now()
           WHERE id = $2"#,
        [chunk.into(), run_id.into()],
    ))
    .await?;
    Ok(())
}

/// Mark task_runs that are still in `running` but haven't sent a log chunk
/// or heartbeat in `stale_after` seconds as `aborted`, and flip their
/// parent task back to `pending` so the scheduler can retry. Returns the
/// list of rows that were reaped so callers can log them.
///
/// Uses started_at as a fallback when a run has never sent a heartbeat
/// (brand-new run from an executor on an old build).
pub async fn reap_stale_runs(
    db: &DatabaseConnection,
    stale_after_secs: i64,
) -> Result<Vec<i32>, DbErr> {
    let cutoff = Utc::now().fixed_offset() - Duration::seconds(stale_after_secs);

    let stale = task_run::Entity::find()
        .filter(task_run::Column::TaskRunStatus.eq(TaskRunStatus::Running))
        .filter(
            task_run::Column::LastHeartbeatAt
                .lt(cutoff)
                .or(task_run::Column::LastHeartbeatAt
                    .is_null()
                    .and(task_run::Column::StartedAt.lt(cutoff))),
        )
        .all(db)
        .await?;

    if stale.is_empty() {
        return Ok(Vec::new());
    }

    let mut reaped = Vec::with_capacity(stale.len());
    for run in stale {
        let task_id = run.task_id;
        let run_id = run.id;

        let mut active_run: task_run::ActiveModel = run.into();
        active_run.task_run_status = Set(TaskRunStatus::Aborted);
        active_run.finished_at = Set(Some(Utc::now().fixed_offset()));
        active_run.error_message = Set(Some(format!(
            "no heartbeat from executor for {stale_after_secs}s (reaped)"
        )));
        active_run.update(db).await?;

        // Only pull the parent task back to `pending` if it's still
        // marked `running`; any other transition (succeeded/failed/aborted)
        // has already moved the task out of the running lane and we
        // don't want to undo it.
        if let Some(parent) = task::Entity::find_by_id(task_id).one(db).await?
            && parent.task_status == TaskStatus::Running
        {
            let mut active: task::ActiveModel = parent.into();
            active.task_status = Set(TaskStatus::Pending);
            active.update(db).await?;
        }

        reaped.push(run_id);
    }

    Ok(reaped)
}

/// Mark a running task_run as aborted (by SLURM scancel or user stop).
/// Parent task goes to `created` — the run is done with no retry.
pub async fn abort_task(
    db: &DatabaseConnection,
    task_id: i32,
    reason: String,
    log: Option<String>,
) -> Result<Option<task::Model>, DbErr> {
    let result = db
        .transaction(|txn| {
            Box::pin(async move {
                let task = task::Entity::find_by_id(task_id)
                    .lock_with_behavior(LockType::Update, LockBehavior::SkipLocked)
                    .one(txn)
                    .await?;
                let Some(task_model) = task else {
                    return Ok(None);
                };
                // Idempotent: if the task already left `running` (stopped
                // from the UI, etc.), don't rewrite it.
                if task_model.task_status != TaskStatus::Running {
                    return Ok(Some(task_model));
                }

                let mut active_task: task::ActiveModel = task_model.into();
                active_task.task_status = Set(TaskStatus::Created);
                let updated_task = active_task.update(txn).await?;

                if let Some(run) = task_run::Entity::find()
                    .filter(task_run::Column::TaskId.eq(task_id))
                    .filter(task_run::Column::TaskRunStatus.eq(TaskRunStatus::Running))
                    .order_by_desc(task_run::Column::Attempt)
                    .lock_with_behavior(LockType::Update, LockBehavior::SkipLocked)
                    .one(txn)
                    .await?
                {
                    let mut active_run: task_run::ActiveModel = run.into();
                    active_run.task_run_status = Set(TaskRunStatus::Aborted);
                    active_run.finished_at = Set(Some(Utc::now().fixed_offset()));
                    active_run.error_message = Set(Some(reason));
                    if log.is_some() {
                        active_run.log = Set(log);
                    }
                    active_run.update(txn).await?;
                }

                Ok(Some(updated_task))
            })
        })
        .await;

    match result {
        Ok(v) => Ok(v),
        Err(TransactionError::Connection(e)) => Err(e),
        Err(TransactionError::Transaction(e)) => Err(e),
    }
}
