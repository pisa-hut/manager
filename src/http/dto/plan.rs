use crate::entity::plan;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreatePlanRequest {
    pub name: String,
    pub map_id: i32,
    pub scenario_id: i32,
    /// Optional free-form labels for grouping. Empty when omitted.
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct PlanResponse {
    pub id: i32,
    pub name: String,
    pub map_id: i32,
    pub scenario_id: i32,
    pub tags: Vec<String>,
}

impl From<plan::Model> for PlanResponse {
    fn from(m: plan::Model) -> Self {
        Self {
            id: m.id,
            name: m.name,
            map_id: m.map_id,
            scenario_id: m.scenario_id,
            tags: m.tags,
        }
    }
}
