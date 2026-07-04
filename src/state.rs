use std::collections::HashMap;
use std::time::{Instant, SystemTime};
use tokio::sync::{Mutex, RwLock};
use crate::model::{SecurityConfig, AssetsConfig, MarkdownConfig, NoteSummary};
use std::sync::Arc;
use bytes::Bytes;
use axum::http::HeaderValue;

#[derive(Clone)]
pub struct HtmlCache {
    pub started_at: SystemTime,
    pub http_date: HeaderValue,
    pub cache_control: HeaderValue,
    pub body: Bytes,
    pub markdown_body: Bytes,
}

impl HtmlCache {
    pub fn new(started_at: SystemTime, body: Bytes, markdown_body: Bytes, html_cache_seconds: u32) -> Self {
        let http_date_str = crate::utils::fmt_http_date(started_at);
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
            markdown_body,
        }
    }
}

pub struct AppState {
    pub html_cache: RwLock<HtmlCache>,
    pub security_config: Arc<SecurityConfig>,
    pub assets_config: Arc<AssetsConfig>,
    pub markdown_config: Arc<MarkdownConfig>,
    pub auth_rate_limiter: Mutex<HashMap<String, Vec<Instant>>>,
    /// 优先从环境变量 LILY_JWT_SECRET 或本地 .jwt_secret 文件加载，保证重启后会话持久
    pub jwt_secret: Vec<u8>,
    /// Agent 公钥（优先 .agent.pub，支持 Ed25519 或 RSA 非对称 JWT 验签）
    pub agent_pub_key: Option<Vec<u8>>,

    // Notes
    pub note_index: RwLock<Vec<NoteSummary>>,
    pub note_html_cache: RwLock<HashMap<String, NoteHtmlCache>>,
    pub note_list_html_cache: RwLock<Option<Bytes>>,
}

#[derive(Clone)]
pub struct NoteHtmlCache {
    pub body: Bytes,
}
