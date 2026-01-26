use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct LeaseTaskRequest {
    pub worker_id: String,
}

#[derive(Debug, Serialize)]
pub struct LeaseTaskResponse {
    pub task_id: String,
    pub task_type: String,
    pub payload: Value,
    pub lease_seconds: u64,
}

#[derive(Debug, Deserialize)]
pub struct CompleteTaskRequest {
    pub worker_id: String,
    pub result: Value,
}

#[derive(Debug, Serialize)]
pub struct CompleteTaskResponse {
    pub ok: bool,
}
