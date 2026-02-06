use axum::extract::State;
use axum::response::sse::{Event, KeepAlive, Sse};
use futures::stream::Stream;
use std::convert::Infallible;
use std::sync::Arc;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

use crate::state::AppState;

/// GET /api/events — Server-Sent Events stream of network events
pub async fn event_stream(
    State(state): State<Arc<AppState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = state.event_tx.subscribe();

    let stream = BroadcastStream::new(rx).filter_map(|result| match result {
        Ok(value) => Some(Ok(Event::default()
            .json_data(value)
            .unwrap_or_else(|_| Event::default().data("{}")))),
        Err(_) => None,
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}
