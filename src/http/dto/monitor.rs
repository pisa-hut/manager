use crate::entity::monitor;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct MonitorResponse {
    pub id: i32,
    pub name: String,
    pub config_sha256: Option<String>,
}

impl From<monitor::Model> for MonitorResponse {
    fn from(m: monitor::Model) -> Self {
        Self {
            id: m.id,
            name: m.name,
            config_sha256: m.config_sha256,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateMonitorRequest {
    pub name: String,
}

pub type MonitorExecutionDto = MonitorResponse;
