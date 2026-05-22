use std::collections::HashMap;
use std::time::{Instant, SystemTime};
use tokio::sync::{Mutex, RwLock};
use crate::model::SecurityConfig;

pub struct AppState {
    pub html_cache: RwLock<String>,
    pub security_config: SecurityConfig,
    pub started_at: RwLock<SystemTime>,
    pub auth_rate_limiter: Mutex<HashMap<String, Vec<Instant>>>,
}
