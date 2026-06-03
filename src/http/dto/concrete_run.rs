use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::entity::concrete_run;

#[derive(Debug, Deserialize)]
pub struct ConcreteRunCreateRequest {
    pub concrete_key: String,
    pub status: String,
    #[serde(default = "default_test_outcome")]
    pub test_outcome: String,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub stop_condition: Option<String>,
    #[serde(default)]
    pub params: Option<Value>,
    #[serde(default)]
    pub final_sim_time_ms: Option<f64>,
    #[serde(default)]
    pub wall_time_ms: Option<f64>,
    #[serde(default)]
    pub total_steps: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct ConcreteRunResponse {
    pub id: i32,
    pub task_id: i32,
    pub task_run_id: i32,
    pub concrete_key: String,
    pub status: String,
    pub test_outcome: String,
    pub reason: Option<String>,
    pub stop_condition: Option<String>,
    pub params: Option<Value>,
    pub final_sim_time_ms: Option<f64>,
    pub wall_time_ms: Option<f64>,
    pub total_steps: Option<i32>,
    pub created_at: String,
}

fn default_test_outcome() -> String {
    "unknown".to_string()
}

impl From<concrete_run::Model> for ConcreteRunResponse {
    fn from(m: concrete_run::Model) -> Self {
        Self {
            id: m.id,
            task_id: m.task_id,
            task_run_id: m.task_run_id,
            concrete_key: m.concrete_key,
            status: m.status,
            test_outcome: m.test_outcome,
            reason: m.reason,
            stop_condition: m.stop_condition,
            params: m.params,
            final_sim_time_ms: m.final_sim_time_ms,
            wall_time_ms: m.wall_time_ms,
            total_steps: m.total_steps,
            created_at: m.created_at.to_rfc3339(),
        }
    }
}
