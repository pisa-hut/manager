use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::entity::sea_orm_active_enums::TaskStatus;
use crate::entity::task;
use crate::http::dto::{
    av::AvExecutionDto, map::MapExecutionDto, sampler::SamplerExecutionDto,
    scenario::ScenarioExecutionDto, simulator::SimulatorExecutionDto,
};

#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    pub plan_id: i32,
    pub av_id: i32,
    pub sampler_id: i32,
    pub simulator_id: i32,
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
    /// Count of concrete-scenario executions the run actually finished.
    /// A run with 0 is "useless" — ten consecutive useless runs fail the
    /// parent task permanently. Older clients omit this; default to 0.
    #[serde(default)]
    pub concrete_scenarios_executed: i32,
}
