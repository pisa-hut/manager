//! Postgres → broadcast fan-out for realtime SSE events.
//!
//! The manager runs a single `LISTEN pisa_events` connection for the whole
//! process; each payload it receives is forwarded to a `tokio::sync::broadcast::Sender`
//! that every SSE handler subscribes to. Channel is capacity-bounded —
//! slow subscribers get `Lagged` errors which the handler skips past,
//! so one stuck tab can't wedge the stream for everyone else.

use sqlx::postgres::{PgListener, PgPoolOptions};
use tokio::sync::broadcast;
use tracing::{error, info, warn};

const CHANNEL: &str = "pisa_events";
const BUFFER: usize = 1024;

pub fn channel() -> (broadcast::Sender<String>, broadcast::Receiver<String>) {
    broadcast::channel(BUFFER)
}

/// Spawn a task that owns a dedicated sqlx pool + PgListener and
/// forwards every NOTIFY payload to `tx`. Reconnects on error with a
/// short backoff.
pub fn spawn_listener(database_url: String, tx: broadcast::Sender<String>) {
    tokio::spawn(async move {
        loop {
            match run_once(&database_url, &tx).await {
                Ok(()) => {
                    warn!("pg_notify listener exited cleanly; reconnecting");
                }
                Err(e) => {
                    error!("pg_notify listener error: {e}; reconnecting in 2s");
                }
            }
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
    });
}

async fn run_once(
    database_url: &str,
    tx: &broadcast::Sender<String>,
) -> Result<(), sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(database_url)
        .await?;
    let mut listener = PgListener::connect_with(&pool).await?;
    listener.listen(CHANNEL).await?;
    info!("pg_notify listener attached to channel `{CHANNEL}`");

    loop {
        let notif = listener.recv().await?;
        // Drop the result — if no subscribers are attached yet,
        // broadcast::Sender::send returns Err(SendError) and we
        // just discard the message.
        let _ = tx.send(notif.payload().to_string());
    }
}
