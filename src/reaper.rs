//! Periodically mark task_run rows that have gone silent as `aborted`.
//!
//! Executors send a log-append PUT every ~1 s while alive; the handler
//! bumps `last_heartbeat_at`. A run that hasn't heart-beat in
//! `STALE_AFTER` seconds is presumed dead (node crash, SIGKILL, network
//! partition) and the reaper flips the row. Parent task is pulled back to
//! `pending` so the scheduler can redispatch on the next poll.

use std::time::Duration;

use sea_orm::DatabaseConnection;
use tracing::{info, warn};

/// Runs older than this without a log-append heartbeat get reaped. Needs
/// to be comfortably larger than the executor's streamer interval (1 s)
/// plus any stop-the-world pauses a live run might have.
const STALE_AFTER_SECS: i64 = 180;
/// How often to scan the table. Cheap — one indexed query.
const POLL_INTERVAL: Duration = Duration::from_secs(30);

pub fn spawn(db: DatabaseConnection) {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(POLL_INTERVAL);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        // Skip the first immediate tick — no point reaping before any
        // executor has had a chance to register.
        ticker.tick().await;

        loop {
            ticker.tick().await;
            match crate::db::task_run::reap_stale_runs(&db, STALE_AFTER_SECS).await {
                Ok(ids) if ids.is_empty() => {}
                Ok(ids) => {
                    info!(
                        "reaper: aborted {} stale task_run(s) with no heartbeat in {}s: {:?}",
                        ids.len(),
                        STALE_AFTER_SECS,
                        ids
                    );
                }
                Err(e) => {
                    warn!("reaper: scan failed: {e}");
                }
            }
        }
    });
}
