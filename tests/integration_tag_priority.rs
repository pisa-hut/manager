//! Integration tests for tag-derived task priority (tag_priority table,
//! priority function, recompute triggers, queued_at decoupling).

mod common;

use sea_orm::{ConnectionTrait, DatabaseBackend, Statement};

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
    // Touch the server handle so the shared harness's `server` field is
    // not dead code in this binary (this test exercises only `app.db`).
    app.server.get("/health").await.assert_status_ok();
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
