use std::time::SystemTime;
use tokio::sync::RwLock;
use crate::model::SecurityConfig;

pub struct AppState {
    pub html_cache: RwLock<String>,
    pub security_config: SecurityConfig,
    pub started_at: RwLock<SystemTime>,
}
