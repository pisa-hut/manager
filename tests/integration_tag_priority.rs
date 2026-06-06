//! Integration tests for tag-derived task priority (tag_priority table,
//! priority function, recompute triggers, queued_at decoupling).

mod common;

use axum::http::StatusCode;
use sea_orm::{ConnectionTrait, DatabaseBackend, Statement};
use serde_json::json;

use crate::common::spawn_test_app;

/// Insert the shared parent rows (id 1) every task needs. Plans/tasks/
/// tag_priority rows are inserted per-test on top of this.
async fn seed_base(app: &crate::common::TestApp) {
    app.db
        .execute_unprepared(
            r#"
            INSERT INTO map (id, name) VALUES (1, 'map');
            INSERT INTO scenario (id, scenario_format, title)
            VALUES (1, 'open_scenario1', 'scenario');
            INSERT INTO av (id, name, image_path, nv_runtime, ros_runtime, carla_runtime, cpu_count, memory_gb, gpu_count)
            VALUES (1, 'av', '{}', false, false, false, 1, 1, 0);
            INSERT INTO simulator (id, name, image_path, nv_runtime, ros_runtime, carla_runtime, cpu_count, memory_gb, gpu_count)
            VALUES (1, 'sim', '{}', false, false, false, 1, 1, 0);
            INSERT INTO sampler (id, name) VALUES (1, 'sampler');
            INSERT INTO monitor (id, name) VALUES (1, 'monitor')
                ON CONFLICT (id) DO NOTHING;
            "#,
        )
        .await
        .expect("seed base rows");
}

async fn priority_of(app: &crate::common::TestApp, task_id: i32) -> i32 {
    let row = app
        .db
        .query_one(Statement::from_string(
            DatabaseBackend::Postgres,
            format!("SELECT queue_priority FROM task WHERE id = {task_id}"),
        ))
        .await
        .expect("query")
        .expect("task row");
    row.try_get::<i32>("", "queue_priority")
        .expect("queue_priority")
}

async fn queued_at_text(app: &crate::common::TestApp, task_id: i32) -> String {
    let row = app
        .db
        .query_one(Statement::from_string(
            DatabaseBackend::Postgres,
            format!("SELECT queued_at::text AS qa FROM task WHERE id = {task_id}"),
        ))
        .await
        .expect("query")
        .expect("task row");
    row.try_get::<String>("", "qa").expect("queued_at text")
}

#[tokio::test]
async fn task_insert_uses_highest_ranked_tag() {
    let app = spawn_test_app().await;
    let _ = &app.server;
    seed_base(&app).await;
    app.db
        .execute_unprepared(
            r#"
            INSERT INTO tag_priority (tag, position) VALUES ('urgent', 0), ('nightly', 1);
            INSERT INTO plan (id, name, map_id, scenario_id, tags)
            VALUES (1, 'plan', 1, 1, '{nightly,urgent}');
            INSERT INTO task (id, plan_id, av_id, simulator_id, sampler_id, monitor_id, task_status)
            VALUES (1, 1, 1, 1, 1, 1, 'queued');
            "#,
        )
        .await
        .expect("seed plan/task");
    assert_eq!(priority_of(&app, 1).await, 1_000_000);
}

#[tokio::test]
async fn untagged_or_unranked_task_priority_is_zero() {
    let app = spawn_test_app().await;
    seed_base(&app).await;
    app.db
        .execute_unprepared(
            r#"
            INSERT INTO tag_priority (tag, position) VALUES ('urgent', 0);
            INSERT INTO plan (id, name, map_id, scenario_id, tags)
            VALUES (1, 'p-none', 1, 1, '{}');
            INSERT INTO plan (id, name, map_id, scenario_id, tags)
            VALUES (2, 'p-unranked', 1, 1, '{misc}');
            INSERT INTO task (id, plan_id, av_id, simulator_id, sampler_id, monitor_id, task_status)
            VALUES (1, 1, 1, 1, 1, 1, 'queued');
            INSERT INTO task (id, plan_id, av_id, simulator_id, sampler_id, monitor_id, task_status)
            VALUES (2, 2, 1, 1, 1, 1, 'queued');
            "#,
        )
        .await
        .expect("seed");
    assert_eq!(priority_of(&app, 1).await, 0);
    assert_eq!(priority_of(&app, 2).await, 0);
}

#[tokio::test]
async fn reordering_ranking_recomputes_queued_tasks() {
    let app = spawn_test_app().await;
    seed_base(&app).await;
    app.db
        .execute_unprepared(
            r#"
            INSERT INTO tag_priority (tag, position) VALUES ('a', 0), ('b', 1);
            INSERT INTO plan (id, name, map_id, scenario_id, tags)
            VALUES (1, 'plan', 1, 1, '{b}');
            INSERT INTO task (id, plan_id, av_id, simulator_id, sampler_id, monitor_id, task_status)
            VALUES (1, 1, 1, 1, 1, 1, 'queued');
            "#,
        )
        .await
        .expect("seed");
    assert_eq!(priority_of(&app, 1).await, 999_999);

    app.db
        .execute_unprepared(
            r#"
            DELETE FROM tag_priority;
            INSERT INTO tag_priority (tag, position) VALUES ('b', 0), ('a', 1);
            "#,
        )
        .await
        .expect("reorder");
    assert_eq!(priority_of(&app, 1).await, 1_000_000);
}

#[tokio::test]
async fn plan_tag_change_recomputes_tasks() {
    let app = spawn_test_app().await;
    seed_base(&app).await;
    app.db
        .execute_unprepared(
            r#"
            INSERT INTO tag_priority (tag, position) VALUES ('hot', 0);
            INSERT INTO plan (id, name, map_id, scenario_id, tags)
            VALUES (1, 'plan', 1, 1, '{}');
            INSERT INTO task (id, plan_id, av_id, simulator_id, sampler_id, monitor_id, task_status)
            VALUES (1, 1, 1, 1, 1, 1, 'queued');
            "#,
        )
        .await
        .expect("seed");
    assert_eq!(priority_of(&app, 1).await, 0);

    app.db
        .execute_unprepared("UPDATE plan SET tags = '{hot}' WHERE id = 1;")
        .await
        .expect("retag plan");
    assert_eq!(priority_of(&app, 1).await, 1_000_000);
}

#[tokio::test]
async fn tag_reorder_does_not_reset_queued_at() {
    let app = spawn_test_app().await;
    seed_base(&app).await;
    app.db
        .execute_unprepared(
            r#"
            INSERT INTO tag_priority (tag, position) VALUES ('x', 0);
            INSERT INTO plan (id, name, map_id, scenario_id, tags)
            VALUES (1, 'plan', 1, 1, '{x}');
            INSERT INTO task (id, plan_id, av_id, simulator_id, sampler_id, monitor_id, task_status)
            VALUES (1, 1, 1, 1, 1, 1, 'queued');
            "#,
        )
        .await
        .expect("seed");
    let before = queued_at_text(&app, 1).await;

    app.db
        .execute_unprepared("UPDATE tag_priority SET position = 5 WHERE tag = 'x';")
        .await
        .expect("reorder");
    let after = queued_at_text(&app, 1).await;
    assert_eq!(
        before, after,
        "queued_at must be unchanged by a tag reorder"
    );
    assert_eq!(priority_of(&app, 1).await, 999_995);
}

#[tokio::test]
async fn ranking_change_leaves_running_tasks_untouched() {
    let app = spawn_test_app().await;
    seed_base(&app).await;
    app.db
        .execute_unprepared(
            r#"
            INSERT INTO tag_priority (tag, position) VALUES ('r', 0);
            INSERT INTO plan (id, name, map_id, scenario_id, tags)
            VALUES (1, 'plan', 1, 1, '{r}');
            INSERT INTO task (id, plan_id, av_id, simulator_id, sampler_id, monitor_id, task_status)
            VALUES (1, 1, 1, 1, 1, 1, 'queued');
            "#,
        )
        .await
        .expect("seed");
    // BEFORE INSERT stamped priority from the ranking (r at pos 0 -> 1_000_000).
    assert_eq!(priority_of(&app, 1).await, 1_000_000);

    // Flip to running, then reorder the ranking. The recompute must skip it.
    app.db
        .execute_unprepared("UPDATE task SET task_status = 'running' WHERE id = 1;")
        .await
        .expect("to running");
    app.db
        .execute_unprepared("UPDATE tag_priority SET position = 7 WHERE tag = 'r';")
        .await
        .expect("reorder");

    // Still the original value — running tasks are excluded from recompute.
    assert_eq!(priority_of(&app, 1).await, 1_000_000);
}

#[tokio::test]
async fn put_then_get_tag_priority_roundtrips_in_order() {
    let app = spawn_test_app().await;

    let resp = app
        .server
        .put("/tag/priority")
        .json(&json!({ "tags": ["alpha", "beta", "gamma"] }))
        .await;
    resp.assert_status(StatusCode::OK);

    let resp = app.server.get("/tag/priority").await;
    resp.assert_status(StatusCode::OK);
    resp.assert_json(&json!([
        { "tag": "alpha", "position": 0 },
        { "tag": "beta",  "position": 1 },
        { "tag": "gamma", "position": 2 }
    ]));
}

#[tokio::test]
async fn put_tag_priority_replaces_and_recomputes() {
    let app = spawn_test_app().await;
    seed_base(&app).await;
    app.db
        .execute_unprepared(
            r#"
            INSERT INTO plan (id, name, map_id, scenario_id, tags)
            VALUES (1, 'plan', 1, 1, '{beta}');
            INSERT INTO task (id, plan_id, av_id, simulator_id, sampler_id, monitor_id, task_status)
            VALUES (1, 1, 1, 1, 1, 1, 'queued');
            "#,
        )
        .await
        .expect("seed");

    app.server
        .put("/tag/priority")
        .json(&json!({ "tags": ["alpha", "beta"] }))
        .await
        .assert_status(StatusCode::OK);
    assert_eq!(priority_of(&app, 1).await, 999_999);

    app.server
        .put("/tag/priority")
        .json(&json!({ "tags": ["beta", "alpha"] }))
        .await
        .assert_status(StatusCode::OK);
    assert_eq!(priority_of(&app, 1).await, 1_000_000);
}

#[tokio::test]
async fn put_tag_priority_rejects_duplicate_and_empty() {
    let app = spawn_test_app().await;
    app.server
        .put("/tag/priority")
        .json(&json!({ "tags": ["a", "a"] }))
        .await
        .assert_status(StatusCode::BAD_REQUEST);
    app.server
        .put("/tag/priority")
        .json(&json!({ "tags": ["a", "  "] }))
        .await
        .assert_status(StatusCode::BAD_REQUEST);
}
