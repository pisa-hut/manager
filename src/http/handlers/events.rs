use std::{convert::Infallible, time::Duration};

use axum::{
    extract::State,
    http::{HeaderValue, header},
    response::{
        IntoResponse, Response,
        sse::{Event, KeepAlive, Sse},
    },
};
use tokio_stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;

use crate::app_state::AppState;

/// `GET /events` — Server-Sent Events feed mirroring every Postgres
/// `NOTIFY pisa_events` payload (insert/update/delete on task + task_run).
///
/// Each client gets its own broadcast receiver. We drop `Lagged` errors
/// so a stuck tab can't back-pressure the others.
pub async fn sse_events(State(state): State<AppState>) -> Response {
    let rx = state.events_tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|res| -> Option<Result<Event, Infallible>> {
        match res {
            Ok(payload) => Some(Ok(Event::default().event("pisa").data(payload))),
            // Subscriber fell behind — skip silently; the next event will catch them up.
            Err(_lagged) => None,
        }
    });

    let mut resp = Sse::new(stream)
        .keep_alive(
            KeepAlive::new()
                .interval(Duration::from_secs(15))
                .text("keep-alive"),
        )
        .into_response();

    // nginx will otherwise buffer the SSE stream and we'd see no events
    // until the connection closed; this header is the portable fix.
    resp.headers_mut().insert(
        header::HeaderName::from_static("x-accel-buffering"),
        HeaderValue::from_static("no"),
    );
    resp.headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    resp
}
