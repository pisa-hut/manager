use crate::entity::simulator;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[derive(Debug, Deserialize)]
pub struct CreateSimulatorRequest {
    pub name: String,
    pub image_path: JsonValue,
    pub nv_runtime: bool,
    pub carla_runtime: bool,
    pub ros_runtime: bool,
    #[serde(default)]
    pub cpu_count: i32,
    #[serde(default)]
    pub memory_gb: i32,
    #[serde(default)]
    pub gpu_count: i32,
    #[serde(default)]
    pub gpu_vram_mb: i32,
}

#[derive(Debug, Serialize)]
pub struct SimulatorResponse {
    pub id: i32,
    pub name: String,
    pub image_path: JsonValue,
    pub nv_runtime: bool,
    pub carla_runtime: bool,
    pub ros_runtime: bool,
    pub extra_ports: Option<serde_json::Value>,
    pub config_sha256: Option<String>,
    pub cpu_count: i32,
    pub memory_gb: i32,
    pub gpu_count: i32,
    pub gpu_vram_mb: i32,
}

impl From<simulator::Model> for SimulatorResponse {
    fn from(m: simulator::Model) -> Self {
        Self {
            id: m.id,
            name: m.name,
            image_path: m.image_path,
            nv_runtime: m.nv_runtime,
            carla_runtime: m.carla_runtime,
            ros_runtime: m.ros_runtime,
            extra_ports: None,
            config_sha256: m.config_sha256,
            cpu_count: m.cpu_count,
            memory_gb: m.memory_gb,
            gpu_count: m.gpu_count,
            gpu_vram_mb: m.gpu_vram_mb,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SimulatorExecutionDto {
    pub id: i32,
    pub name: String,
    pub image_path: JsonValue,
    pub nv_runtime: bool,
    pub carla_runtime: bool,
    pub ros_runtime: bool,
    pub extra_ports: Option<serde_json::Value>,
    pub config_sha256: Option<String>,
    pub cpu_count: i32,
    pub memory_gb: i32,
    pub gpu_count: i32,
    pub gpu_vram_mb: i32,
}

impl From<simulator::Model> for SimulatorExecutionDto {
    fn from(m: simulator::Model) -> Self {
        Self {
            id: m.id,
            name: m.name,
            image_path: m.image_path,
            nv_runtime: m.nv_runtime,
            carla_runtime: m.carla_runtime,
            ros_runtime: m.ros_runtime,
            extra_ports: None,
            config_sha256: m.config_sha256,
            cpu_count: m.cpu_count,
            memory_gb: m.memory_gb,
            gpu_count: m.gpu_count,
            gpu_vram_mb: m.gpu_vram_mb,
        }
    }
}
