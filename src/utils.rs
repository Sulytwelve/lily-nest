//! 跨模块共享的纯工具函数。
//! 不依赖 AppState，不依赖外部 I/O。

use std::time::{Duration, SystemTime, UNIX_EPOCH};

// ── HTTP 日期 ────────────────────────────────────────

/// HTTP 日期（RFC 2822 / IMF-fixdate）字符串 → SystemTime（秒级精度）
pub fn parse_http_date(s: &str) -> Option<SystemTime> {
    chrono::DateTime::parse_from_rfc2822(s)
        .ok()
        .and_then(|dt| {
            let ts = dt.timestamp();
            if ts >= 0 {
                UNIX_EPOCH.checked_add(Duration::from_secs(ts as u64))
            } else {
                UNIX_EPOCH.checked_sub(Duration::from_secs((-ts) as u64))
            }
        })
}

/// SystemTime → HTTP 日期（RFC 2822 / IMF-fixdate）字符串
pub fn fmt_http_date(st: SystemTime) -> String {
    let dt: chrono::DateTime<chrono::Utc> = st.into();
    dt.format("%a, %d %b %Y %H:%M:%S GMT").to_string()
}

// ── 安全过滤 ─────────────────────────────────────────

/// 仅放行 http:// https:// 和 / 开头的 URL，其余落地到页内锚点
pub fn sanitize_url(url: &str) -> &str {
    let url = url.trim();
    if (url.starts_with("http://") || url.starts_with("https://"))
        || (url.starts_with('/') && !url.starts_with("//"))
    {
        url
    } else {
        "#projects"
    }
}

// ── HTML 转义 ────────────────────────────────────────

/// 将字符串中转义为 HTML 实体，安全插入文本节点 / 属性值
pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
