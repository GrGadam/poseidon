use std::{collections::HashMap, sync::Arc};

use axum::extract::FromRef;
use sqlx::SqlitePool;
use tokio::sync::{broadcast, Mutex};

use crate::{config::Config, ws::WsEnvelope};

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub config: Arc<Config>,
    pub ws_tx: broadcast::Sender<WsEnvelope>,
    pub presence_connections: Arc<Mutex<HashMap<String, usize>>>,
}

impl FromRef<AppState> for Arc<Config> {
    fn from_ref(input: &AppState) -> Self {
        input.config.clone()
    }
}
