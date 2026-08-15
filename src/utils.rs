use std::time::{Duration, SystemTime, UNIX_EPOCH};

// HTTP 日期

/// HTTP 日期（RFC 2822 / IMF-fixdate）字符串 → SystemTime（秒级精度）
pub fn parse_http_date(s: &str) -> Option<SystemTime> {
    chrono::DateTime::parse_from_rfc2822(s).ok().and_then(|dt| {
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

// 安全过滤

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

// HTML 转义

/// 将字符串中转义为 HTML 实体，安全插入文本节点 / 属性值
pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// 单遍模板渲染：只扫描原始 template 片段，插入值后绝不重扫插入值。
/// 用于阻断“值中包含 `{{placeholder}}` 被后续替换”的二阶模板注入。
pub fn render_once(template: &str, vars: &[(&str, &str)]) -> String {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    loop {
        let next = vars
            .iter()
            .filter_map(|(key, value)| rest.find(*key).map(|idx| (idx, *key, *value)))
            .min_by_key(|(idx, _, _)| *idx);
        match next {
            Some((idx, key, value)) => {
                out.push_str(&rest[..idx]);
                out.push_str(value);
                rest = &rest[idx + key.len()..];
            }
            None => {
                out.push_str(rest);
                break;
            }
        }
    }
    out
}

/// 将 JSON 文本转为可安全嵌入 HTML `<script>` 元素的合法 JSON 转义形式。
/// 消除 `</script>` 大小写绕过与 `<!--` 造成的 script 解析状态混淆。
pub fn escape_json_for_html_script(json: &str) -> String {
    json.replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('&', "\\u0026")
        .replace('\u{2028}', "\\u2028")
        .replace('\u{2029}', "\\u2029")
}
