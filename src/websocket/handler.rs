use axum::{
    extract::{ws::{ Message, WebSocket},State, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::{StreamExt, SinkExt};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tokio::sync::broadcast;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatMessage {
    username: String,
    message: String,
}

type SharedState = Arc<Mutex<HashMap<String, broadcast::Sender<ChatMessage>>>>;
async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<SharedState>,
)->impl IntoResponse{
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(
    ws: WebSocket,
    state:SharedState,
) {
    let (mut sender, mut receiver) = ws.split();
    let (tx, mut rx) = broadcast::channel(100);

    let id = format!("{}", Uuid::new_v4());
    state.lock().unwrap().insert(id.clone(), tx.clone());

    tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if let Ok(json) = serde_json::to_string(&msg) {
                if sender.send(Message::Text(json)).await.is_err() {
                    break;
                }
            }
        }
    });

    while let Some(Ok(Message::Text(text))) = receiver.next().await {
        if let Ok(msg) = serde_json::from_str::<ChatMessage>(&text) {
            let state = state.lock().unwrap();
            for (_, tx) in state.iter() {
                let _ = tx.send(msg.clone());
            }
        }
    }

    state.lock().unwrap().remove(&id);
}

pub fn ws_routes() -> Router {
    let clients = SharedState::default();

    Router::new()
        .route("/ws", get(ws_handler))
        .with_state(clients)
}
