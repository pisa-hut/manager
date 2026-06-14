use serde::Deserialize;

/// Live mid-run progress snapshot pushed by the executor. `expected` is
/// the sampler's total when known, omitted/None for open-ended samplers.
#[derive(Debug, Deserialize)]
pub struct TaskRunProgressRequest {
    pub finished_concrete_runs: i32,
    pub aborted_concrete_runs: i32,
    pub skipped_concrete_runs: i32,
    #[serde(default)]
    pub expected_concrete_runs: Option<i32>,
}
