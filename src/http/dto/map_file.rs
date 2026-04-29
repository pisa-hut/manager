use crate::entity::map_file;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct MapFileMetaResponse {
    pub id: i32,
    pub map_id: i32,
    pub relative_path: String,
    pub content_sha256: String,
    pub size: usize,
}

impl From<map_file::Model> for MapFileMetaResponse {
    fn from(m: map_file::Model) -> Self {
        let size = m.content.len();
        Self {
            id: m.id,
            map_id: m.map_id,
            relative_path: m.relative_path,
            content_sha256: m.content_sha256,
            size,
        }
    }
}
