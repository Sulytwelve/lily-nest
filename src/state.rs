use std::collections::HashMap;
use std::time::{Instant, SystemTime};
use tokio::sync::{Mutex, RwLock};
use crate::model::SecurityConfig;
use std::sync::Arc;
use bytes::Bytes;
use axum::http::HeaderValue;

#[derive(Clone)]
pub struct HtmlCache {
    pub started_at: SystemTime,
    pub http_date: HeaderValue,
    pub body: Bytes,
}

impl HtmlCache {
    pub fn new(started_at: SystemTime, body: Bytes) -> Self {
        let http_date_str = httpdate::fmt_http_date(started_at);
        let http_date = HeaderValue::try_from(http_date_str)
            .unwrap_or_else(|_| HeaderValue::from_static("Thu, 01 Jan 1970 00:00:00 GMT"));
        Self {
            started_at,
            http_date,
            body,
        }
    }
}

pub struct AppState {
    pub html_cache: RwLock<HtmlCache>,
    pub security_config: Arc<SecurityConfig>,
    pub auth_rate_limiter: Mutex<HashMap<String, Vec<Instant>>>,
}
