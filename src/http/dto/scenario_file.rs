use crate::entity::scenario_file;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ScenarioFileMetaResponse {
    pub id: i32,
    pub scenario_id: i32,
    pub relative_path: String,
    pub content_sha256: String,
    pub size: usize,
}

impl From<scenario_file::Model> for ScenarioFileMetaResponse {
    fn from(m: scenario_file::Model) -> Self {
        let size = m.content.len();
        Self {
            id: m.id,
            scenario_id: m.scenario_id,
            relative_path: m.relative_path,
            content_sha256: m.content_sha256,
            size,
        }
    }
}
