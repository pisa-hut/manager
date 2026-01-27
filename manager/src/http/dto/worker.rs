use crate::entity::worker;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateWorkerRequest {
    pub job_id: i32,
    pub node_list: String,
    pub hostname: String,
}

#[derive(Debug, Serialize)]
pub struct WorkerResponse {
    pub id: i32,
    pub job_id: i32,
    pub node_list: String,
    pub hostname: String,
}

impl From<worker::Model> for WorkerResponse {
    fn from(m: worker::Model) -> Self {
        Self {
            id: m.id,
            job_id: m.job_id,
            node_list: m.node_list,
            hostname: m.hostname,
        }
    }
}
