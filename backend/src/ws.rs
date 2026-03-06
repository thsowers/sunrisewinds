use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::IntoResponse;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{info, warn};

use crate::models::WsMessage;
use crate::state::AppState;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    info!("WebSocket client connected");

    // Send full state snapshot immediately on connect
    let full_state = state.build_full_state();
    let json = serde_json::to_string(&WsMessage::FullState(full_state)).unwrap_or_default();
    if socket.send(Message::Text(json.into())).await.is_err() {
        return;
    }

    let mut rx = state.broadcast_tx.subscribe();

    loop {
        tokio::select! {
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Close(_))) | None => {
                        info!("WebSocket client disconnected");
                        break;
                    }
                    _ => {}
                }
            }
            result = rx.recv() => {
                match result {
                    Ok(ws_msg) => {
                        let json = serde_json::to_string(&ws_msg).unwrap_or_default();
                        if socket.send(Message::Text(json.into())).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        warn!("WebSocket client lagged behind {} messages", n);
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        break;
                    }
                }
            }
        }
    }
}
