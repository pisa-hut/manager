use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::entity::sea_orm_active_enums::TaskStatus;
use crate::entity::task;
use crate::http::dto::{
    av::AvExecutionDto, map::MapExecutionDto, monitor::MonitorExecutionDto,
    sampler::SamplerExecutionDto, scenario::ScenarioExecutionDto, simulator::SimulatorExecutionDto,
};

#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    pub plan_id: i32,
    pub av_id: i32,
    pub sampler_id: i32,
    pub simulator_id: i32,
    /// Required: every task pins exactly one monitor. The seeded
    /// `default` monitor (id=1 on systems migrated through
    /// m20260513) is the safe choice when callers don't have a
    /// reason to pick a more specific one.
    pub monitor_id: i32,
}

#[derive(Debug, Serialize)]
pub struct TaskResponse {
    pub id: i32,
    pub plan_id: i32,
    pub av_id: i32,
    pub simulator_id: i32,
    pub sampler_id: i32,
    pub task_status: TaskStatus,
    pub created_at: DateTime<Utc>,
    pub attempt_count: i32,
    pub archived: bool,
    pub monitor_id: i32,
}

impl From<task::Model> for TaskResponse {
    fn from(m: task::Model) -> Self {
        Self {
            id: m.id,
            plan_id: m.plan_id,
            av_id: m.av_id,
            simulator_id: m.simulator_id,
            sampler_id: m.sampler_id,
            task_status: m.task_status,
            created_at: m.created_at.with_timezone(&Utc),
            attempt_count: m.attempt_count,
            archived: m.archived,
            monitor_id: m.monitor_id,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ClaimTaskRequest {
    pub executor_id: i32,
    pub task_id: Option<i32>,
    pub map_id: Option<i32>,
    pub scenario_id: Option<i32>,
    pub av_id: Option<i32>,
    pub simulator_id: Option<i32>,
    pub sampler_id: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct ClaimTaskResponse {
    pub task: TaskExecutionDto,
    /// Id of the task_run row created by this claim. Executors PUT log
    /// chunks to /task_run/{id}/log/append using this id.
    pub task_run_id: i32,
    pub av: AvExecutionDto,
    pub map: MapExecutionDto,
    pub scenario: ScenarioExecutionDto,
    pub simulator: SimulatorExecutionDto,
    pub sampler: SamplerExecutionDto,
    /// Always populated since the m20260513 migration; the
    /// executor reads its config bytes via /monitor/{id}/config.
    pub monitor: MonitorExecutionDto,
}

#[derive(Debug, Serialize)]
pub struct TaskExecutionDto {
    pub id: i32,
}

impl From<task::Model> for TaskExecutionDto {
    fn from(m: task::Model) -> Self {
        Self { id: m.id }
    }
}

#[derive(Debug, Deserialize)]
pub struct TaskRunUpdateRequest {
    pub task_id: i32,
    pub reason: Option<String>,
    #[serde(default)]
    pub log: Option<String>,
    /// Cumulative count of concrete scenarios that finished cleanly across
    /// this task (every attempt, including prior task_runs, contributes).
    /// Each task_run row records the snapshot at finalisation. Omitted by
    /// the SIGTERM / init-failure paths — the manager inherits the prior
    /// task_run's snapshot in that case so the cumulative never appears to
    /// decrease.
    #[serde(default)]
    pub finished_concrete_runs: Option<i32>,
    /// Same cumulative semantics for aborted concretes (wrapper raised an
    /// unrecognised error). See `finished_concrete_runs`.
    #[serde(default)]
    pub aborted_concrete_runs: Option<i32>,
    /// Same cumulative semantics for skipped concretes (precondition
    /// rejected the run, retry budget exhausted, etc.).
    #[serde(default)]
    pub skipped_concrete_runs: Option<i32>,
}
