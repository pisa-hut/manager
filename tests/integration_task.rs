//! HTTP-level integration tests for the task lifecycle endpoints.

mod common;

use axum::http::StatusCode;
use serde_json::json;

use crate::common::spawn_test_app;

async fn seed_running_task_run(app: &crate::common::TestApp) {
    use sea_orm::ConnectionTrait;
    app.db
        .execute_unprepared(
            r#"
            INSERT INTO executor (id, slurm_job_id, slurm_node_list, hostname)
            VALUES (100, 42, 'node-0', 'test-host');
            INSERT INTO map (id, name) VALUES (100, 'map');
            INSERT INTO scenario (id, scenario_format, title)
            VALUES (100, 'open_scenario1', 'scenario');
            INSERT INTO plan (id, name, map_id, scenario_id, tags)
            VALUES (100, 'plan', 100, 100, '{}');
            INSERT INTO av (id, name, image_path, nv_runtime, ros_runtime, carla_runtime, cpu_count, memory_gb, gpu_count)
            VALUES (100, 'av', '{}', false, false, false, 1, 1, 0);
            INSERT INTO simulator (id, name, image_path, nv_runtime, ros_runtime, carla_runtime, cpu_count, memory_gb, gpu_count)
            VALUES (100, 'sim', '{}', false, false, false, 1, 1, 0);
            INSERT INTO sampler (id, name) VALUES (100, 'sampler');
            INSERT INTO monitor (id, name) VALUES (100, 'monitor');
            INSERT INTO task (id, plan_id, av_id, simulator_id, sampler_id, monitor_id, task_status)
            VALUES (100, 100, 100, 100, 100, 100, 'running');
            INSERT INTO task_run (id, task_id, executor_id, attempt, task_run_status, started_at)
            VALUES (100, 100, 100, 1, 'running', now());
            "#,
        )
        .await
        .expect("seed running task_run");
}

#[tokio::test]
async fn task_failed_rejects_negative_finished_concrete_runs() {
    let app = spawn_test_app().await;

    // Validation should fire before any DB lookup, so we don't need
    // to seed a task for this test.
    let resp = app
        .server
        .post("/task/failed")
        .json(&json!({
            "task_id": 1,
            "reason": "smoke",
            "finished_concrete_runs": -1,
        }))
        .await;

    resp.assert_status(StatusCode::BAD_REQUEST);
    assert!(
        resp.text().contains("finished_concrete_runs"),
        "expected error body to mention the field name; got: {}",
        resp.text()
    );
}

#[tokio::test]
async fn task_failed_rejects_negative_aborted_concrete_runs() {
    let app = spawn_test_app().await;

    let resp = app
        .server
        .post("/task/failed")
        .json(&json!({
            "task_id": 1,
            "reason": "smoke",
            "aborted_concrete_runs": -1,
        }))
        .await;

    resp.assert_status(StatusCode::BAD_REQUEST);
    assert!(
        resp.text().contains("aborted_concrete_runs"),
        "expected error body to mention the field name; got: {}",
        resp.text()
    );
}

#[tokio::test]
async fn task_failed_rejects_negative_skipped_concrete_runs() {
    let app = spawn_test_app().await;

    let resp = app
        .server
        .post("/task/failed")
        .json(&json!({
            "task_id": 1,
            "reason": "smoke",
            "skipped_concrete_runs": -1,
        }))
        .await;

    resp.assert_status(StatusCode::BAD_REQUEST);
    assert!(
        resp.text().contains("skipped_concrete_runs"),
        "expected error body to mention the field name; got: {}",
        resp.text()
    );
}

#[tokio::test]
async fn task_claim_with_no_pending_returns_null() {
    let app = spawn_test_app().await;

    // First we need an executor row so the claim endpoint doesn't
    // 404 with "worker not found". Insert one via PostgREST-style
    // raw SQL since we don't have HTTP CRUD for executors directly
    // exposed.
    use sea_orm::{ConnectionTrait, Statement};
    app.db
        .execute(Statement::from_string(
            sea_orm::DbBackend::Postgres,
            "INSERT INTO executor \
                 (slurm_job_id, slurm_node_list, hostname) \
             VALUES (1, 'node-0', 'test-host')",
        ))
        .await
        .expect("seed executor");

    let resp = app
        .server
        .post("/task/claim")
        .json(&json!({ "executor_id": 1 }))
        .await;

    resp.assert_status_ok();
    // No pending tasks exist → manager returns JSON null.
    assert_eq!(resp.text(), "null");
}

#[tokio::test]
async fn create_concrete_runs_persists_rows_for_running_task_run() {
    let app = spawn_test_app().await;
    seed_running_task_run(&app).await;

    let resp = app
        .server
        .post("/task_run/100/concrete_runs")
        .json(&json!([
            {
                "concrete_key": "iteration_1",
                "status": "finished",
                "test_outcome": "success",
                "reason": "completed",
                "stop_condition": "timeout",
                "params": {"speed": 12},
                "final_sim_time_ms": 1000.0,
                "wall_time_ms": 1200.0,
                "total_steps": 10
            }
        ]))
        .await;

    resp.assert_status_ok();
    let rows: serde_json::Value = resp.json();
    assert_eq!(rows[0]["task_id"], 100);
    assert_eq!(rows[0]["task_run_id"], 100);
    assert_eq!(rows[0]["concrete_key"], "iteration_1");
    assert_eq!(rows[0]["status"], "finished");
}

#[tokio::test]
async fn create_concrete_runs_rejects_terminal_task_run() {
    let app = spawn_test_app().await;
    seed_running_task_run(&app).await;
    use sea_orm::{ConnectionTrait, Statement};
    app.db
        .execute(Statement::from_string(
            sea_orm::DbBackend::Postgres,
            "UPDATE task_run SET task_run_status = 'completed' WHERE id = 100",
        ))
        .await
        .expect("finalise task_run");

    let resp = app
        .server
        .post("/task_run/100/concrete_runs")
        .json(&json!([{ "concrete_key": "iteration_1", "status": "finished" }]))
        .await;

    resp.assert_status(StatusCode::GONE);
}

#[tokio::test]
async fn health_endpoint_returns_200() {
    let app = spawn_test_app().await;
    let resp = app.server.get("/health").await;
    resp.assert_status_ok();
}
