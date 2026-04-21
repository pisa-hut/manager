use sea_orm::DatabaseConnection;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    /// Fan-out channel for `pisa_events` NOTIFY payloads. Each SSE
    /// subscriber holds its own receiver; the single PgListener task
    /// (see `events::spawn_listener`) is the sole producer.
    pub events_tx: broadcast::Sender<String>,
}
