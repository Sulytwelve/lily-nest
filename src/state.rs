use std::collections::HashMap;
use std::time::{Instant, SystemTime};
use tokio::sync::{Mutex, RwLock};
use crate::model::{SecurityConfig, AssetsConfig};
use std::sync::Arc;
use bytes::Bytes;
use axum::http::HeaderValue;

#[derive(Clone)]
pub struct HtmlCache {
    pub started_at: SystemTime,
    pub http_date: HeaderValue,
    pub cache_control: HeaderValue,
    pub body: Bytes,
}

impl HtmlCache {
    pub fn new(started_at: SystemTime, body: Bytes, html_cache_seconds: u32) -> Self {
        let http_date_str = httpdate::fmt_http_date(started_at);
        let http_date = HeaderValue::try_from(http_date_str)
            .unwrap_or_else(|_| HeaderValue::from_static("Thu, 01 Jan 1970 00:00:00 GMT"));
        let cc_str = format!("public, max-age={}", html_cache_seconds);
        let cache_control = HeaderValue::try_from(cc_str)
            .unwrap_or_else(|_| HeaderValue::from_static("public, max-age=3600"));
        Self {
            started_at,
            http_date,
            cache_control,
            body,
        }
    }
}

pub struct AppState {
    pub html_cache: RwLock<HtmlCache>,
    pub security_config: Arc<SecurityConfig>,
    pub assets_config: Arc<AssetsConfig>,
    pub auth_rate_limiter: Mutex<HashMap<String, Vec<Instant>>>,
    /// 启动时随机生成，重启后所有 token 自动失效
    pub jwt_secret: Vec<u8>,
}
