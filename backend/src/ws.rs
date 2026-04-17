use axum::{
    extract::{ws::Message, WebSocketUpgrade},
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{auth::Claims, error::AppError, state::AppState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsEnvelope {
    pub kind: String,
    pub payload: Value,
}

#[derive(Debug, Deserialize)]
pub struct WsQuery {
    pub token: String,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    axum::extract::Query(query): axum::extract::Query<WsQuery>,
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<Response, AppError> {
    let token_data = jsonwebtoken::decode::<Claims>(
        &query.token,
        &jsonwebtoken::DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
        &jsonwebtoken::Validation::default(),
    )
    .map_err(|_| AppError::Unauthorized)?;

    let user_id = token_data.claims.sub;
    Ok(ws.on_upgrade(move |socket| handle_socket(socket, state, user_id)))
}

async fn handle_socket(stream: axum::extract::ws::WebSocket, state: AppState, user_id: String) {
    let (mut sender, mut receiver) = stream.split();
    let mut rx = state.ws_tx.subscribe();

    let (snapshot_online_ids, became_online) = {
        let mut lock = state.presence_connections.lock().await;
        let current_online_ids = lock.keys().cloned().collect::<Vec<_>>();

        let counter = lock.entry(user_id.clone()).or_insert(0);
        *counter += 1;

        (current_online_ids, *counter == 1)
    };

    if let Ok(snapshot_text) = serde_json::to_string(&WsEnvelope {
        kind: "presence.snapshot".to_string(),
        payload: serde_json::json!({"online_user_ids": snapshot_online_ids}),
    }) {
        let _ = sender.send(Message::Text(snapshot_text.into())).await;
    }

    if became_online {
        emit(
            &state,
            "presence.updated",
            serde_json::json!({"user_id": user_id, "online": true}),
        );
    }

    let send_task = tokio::spawn(async move {
        while let Ok(evt) = rx.recv().await {
            if let Ok(text) = serde_json::to_string(&evt)
                && sender.send(Message::Text(text.into())).await.is_err()
            {
                break;
            }
        }
    });

    let recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Close(_)) => break,
                Ok(_) => {}
                Err(_) => break,
            }
        }
    });

    let _ = tokio::join!(send_task, recv_task);

    let became_offline = {
        let mut lock = state.presence_connections.lock().await;

        if let Some(counter) = lock.get_mut(&user_id) {
            if *counter > 1 {
                *counter -= 1;
                false
            } else {
                lock.remove(&user_id);
                true
            }
        } else {
            false
        }
    };

    if became_offline {
        emit(
            &state,
            "presence.updated",
            serde_json::json!({"user_id": user_id, "online": false}),
        );
    }
}

pub fn emit(state: &AppState, kind: &str, payload: Value) {
    let _ = state.ws_tx.send(WsEnvelope {
        kind: kind.to_string(),
        payload,
    });
}
