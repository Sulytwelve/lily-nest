use crate::model::{AssetsConfig, AuthSecrets, MarkdownConfig, NoteSummary, SecurityConfig};
use axum::http::HeaderValue;
use bytes::Bytes;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Instant, SystemTime};
use tokio::sync::{Mutex, RwLock};

#[derive(Clone)]
pub struct HtmlCache {
    pub started_at: SystemTime,
    pub http_date: HeaderValue,
    pub cache_control: HeaderValue,
    pub body: Bytes,
    pub markdown_body: Bytes,
}

impl HtmlCache {
    pub fn new(
        started_at: SystemTime,
        body: Bytes,
        markdown_body: Bytes,
        html_cache_seconds: u32,
    ) -> Self {
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

/// 单个登录限流桶：固定 60 秒窗口 + 请求计数（B3/B26）。
pub struct RateLimitBucket {
    pub window_start: Instant,
    pub count: u32,
}

/// 有界登录限流分片：按 client key 分桶，并定期清理过期桶（B26）。
pub struct RateLimitShard {
    pub buckets: HashMap<String, RateLimitBucket>,
    pub last_cleanup: Instant,
}

impl Default for RateLimitShard {
    fn default() -> Self {
        Self {
            buckets: HashMap::new(),
            last_cleanup: Instant::now(),
        }
    }
}

/// 分片限流表（B26）：把登录请求分散到多个互斥分片上，避免单个
/// `Mutex<HashMap>` 成为全登录请求的串行点；每个分片仍有独立上限。
pub struct RateLimitTable {
    shards: Vec<Mutex<RateLimitShard>>,
    per_shard_limit: usize,
}

impl RateLimitTable {
    pub fn new(shard_count: usize, per_shard_limit: usize) -> Self {
        let shard_count = shard_count.max(1);
        let mut shards = Vec::with_capacity(shard_count);
        for _ in 0..shard_count {
            shards.push(Mutex::new(RateLimitShard::default()));
        }
        Self {
            shards,
            per_shard_limit,
        }
    }

    fn shard_index(&self, key: &str) -> usize {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        key.hash(&mut hasher);
        (hasher.finish() as usize) % self.shards.len()
    }

    pub async fn lock_shard(&self, key: &str) -> tokio::sync::MutexGuard<'_, RateLimitShard> {
        let idx = self.shard_index(key);
        self.shards[idx].lock().await
    }

    pub fn per_shard_limit(&self) -> usize {
        self.per_shard_limit
    }
}

pub struct AppState {
    pub html_cache: RwLock<HtmlCache>,
    pub security_config: Arc<SecurityConfig>,
    /// 认证秘密（密码/密保答案哈希）运行时状态；Web 初始化与在线改密会更新它。
    pub auth_secrets: RwLock<AuthSecrets>,
    /// 首次 Web 初始化的一次性 setup code，10 分钟有效。
    pub setup_code: Mutex<Option<(String, Instant)>>,
    pub assets_config: Arc<AssetsConfig>,
    pub markdown_config: Arc<MarkdownConfig>,
    pub cloudflare_config: Arc<crate::model::CloudflareConfig>,
    pub auth_rate_limiter: RateLimitTable,
    /// B19：服务端吊销表，key 为 JWT 的 jti，value 为过期时间戳；过期后由登录/登出流程清理。
    pub revoked_jtis: Mutex<HashMap<String, u64>>,
    /// 优先从环境变量 LILY_JWT_SECRET 或本地 .jwt_secret 文件加载，保证重启后会话持久
    pub jwt_secret: Vec<u8>,
    /// Agent 公钥（优先 .agent.pub，支持 Ed25519 非对称 JWT 验签）
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
