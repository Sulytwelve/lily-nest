use std::vec;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct HomeProfile {
    pub current_identity: String,  // 比如 "Home Page"
    pub avatar_url: String,        // 头像地址
    pub bg_url: String,            // 背景图地址
    pub team_members: Vec<String>, // ["User_1", "User_2"]
    pub site_version: String,      // 版本号
    pub intro: String,             // 自我介绍
    pub note_url: String,          // 笔记地址
    pub note_enable: bool,         // 是否启用笔记
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str, // "ok"
}

impl Default for HomeProfile {
    fn default() -> Self {
        Self {
            current_identity: "Default".to_string(),
            avatar_url: "/images/avatar.webp".to_string(),
            bg_url: "/images/bg.webp".to_string(),
            team_members: vec!["User_1".into(), "User_2".into(), "User_3".into()],
            site_version: env!("CARGO_PKG_VERSION").to_string(),
            intro: "Hi！欢迎下滑探索我的项目～".to_string(),
            note_url: "https://sulyhub.cn".to_string(),
            note_enable: false,
        }
    }
}

impl Default for HealthResponse {
    fn default() -> Self {
        Self { status: "ok" }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    pub name: String,
    pub desc: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectList {
    pub items: Vec<Project>,
}

// 为 ProjectList 实现 Default
impl Default for ProjectList {
    fn default() -> Self {
        Self {
            items: vec![Project {
                name: "更多项目".into(),
                desc: "探索".into(),
                url: "https://github.com/".into(),
            }],
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AboutItem {
    pub icon_url: String,
    pub title: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AboutList {
    pub items: Vec<AboutItem>,
}

impl Default for AboutList {
    fn default() -> Self {
        Self {
            items: vec![AboutItem {
                icon_url: "/images/default_icon.webp".into(),
                title: "Template".into(),
                content: "default.".into(),
            }],
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChangelogItem {
    pub date: String,
    pub title: String,
    pub content: String,
    pub tag: Option<String>,
    pub since: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChangelogList {
    pub items: Vec<ChangelogItem>,
}

impl Default for ChangelogList {
    fn default() -> Self {
        Self {
            items: vec![ChangelogItem {
                date: "2026-01-01".to_string(),
                title: "初始版本".to_string(),
                content: "梨窝上线。".to_string(),
                tag: None,
                since: None,
            }],
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub http_port: u16,
    pub https_port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            http_port: 8880,
            https_port: 8443,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TlsConfig {
    pub cert_path: String,
    pub key_path: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SecurityConfig {
    pub allow_origins: Vec<String>,
    pub csp_policy: String,
    pub permissions_policy: String,
    pub admin_password: Option<String>,
    pub admin_security_answers: Option<Vec<String>>,
    pub admin_security_questions: Option<Vec<String>>,
    pub auth_ext_secq: Option<bool>,
    pub auth_ext_cftrace: Option<bool>,
    pub cftrace_url: Option<String>,
    pub allowed_locs: Option<Vec<String>>,
    pub jwt_expiry_secs: Option<u64>,
    pub trusted_proxy_ips: Option<Vec<String>>,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            allow_origins: vec!["*".into()],
            csp_policy: "default-src 'self'; script-src 'self'; \
                         style-src 'self' 'unsafe-inline'; img-src 'self' data:; \
                         connect-src 'self' https://cloudflare.com https://*.cloudflare.com; font-src 'self'; object-src 'none'; \
                         base-uri 'self'; form-action 'self'; frame-ancestors 'none'"
                .into(),
            permissions_policy: "camera=(), microphone=(), geolocation=(), payment=()".into(),
            admin_password: None,
            admin_security_answers: Some(vec![
                "default1".to_string(),
                "default2".to_string(),
                "default3".to_string(),
            ]),
            admin_security_questions: Some(vec![
                "default1".to_string(),
                "default2".to_string(),
                "default3".to_string(),
            ]),
            auth_ext_secq: Some(false),
            auth_ext_cftrace: Some(false),
            cftrace_url: Some("https://cloudflare.com/cdn-cgi/trace".to_string()),
            allowed_locs: Some(vec!["CN".to_string()]),
            jwt_expiry_secs: Some(28800),
            trusted_proxy_ips: Some(vec![]),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct AssetsConfig {
    pub precompress: bool,
    pub assets_dirs: Vec<String>,
    pub target_exts: Vec<String>,
    pub compression_types: Vec<String>,
    pub zstd_level: i32,
    pub brotli_quality: u32,
    pub gzip_level: u32,
    pub html_cache_seconds: u32,
    pub api_cache_seconds: u32,
    pub js_css_cache_seconds: u32,
    pub image_cache_seconds: u32,
    pub font_cache_seconds: u32,
    pub other_cache_seconds: u32,
}

impl Default for AssetsConfig {
    fn default() -> Self {
        Self {
            precompress: false,
            assets_dirs: vec![
                "./static/css".to_string(),
                "./static/css/vendor".to_string(),
                "./static/css/vendor/fonts".to_string(),
                "./static/js".to_string(),
                "./static/js/vendor".to_string(),
                "./static/fonts".to_string(),
                "./static/fonts/vendor".to_string(),
            ],
            target_exts: vec![
                "css".to_string(),
                "js".to_string(),
                "woff".to_string(),
                "woff2".to_string(),
                "ttf".to_string(),
                "otf".to_string(),
            ],
            compression_types: vec!["br".to_string()],
            zstd_level: 3,
            brotli_quality: 11,
            gzip_level: 9,
            html_cache_seconds: 3600,
            api_cache_seconds: 0,
            js_css_cache_seconds: 86400,
            image_cache_seconds: 86400,
            font_cache_seconds: 604800,
            other_cache_seconds: 3600,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CloudflareConfig {
    pub web_analytics_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct MarkdownConfig {
    #[serde(default)]
    pub enable: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigFile {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SaveConfigRequest {
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdminLoginRequest {
    pub password: String,
    pub answer: Option<String>,
    pub question_index: Option<usize>,
    pub cf_trace: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdminAuthPageConfig {
    pub auth_ext_secq: bool,
    pub auth_ext_cftrace: bool,
    pub question_count: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdminLoginQuestion {
    pub question_index: usize,
    pub question: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminLoginResponse {
    pub token: String,
    pub expires_at: u64,
    pub role: String,
    pub name: String,
}

/// 统一的 JWT 认证数据结构，支持显式角色与昵称
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthClaims {
    pub sub: String,  // "admin" 或用户标识
    pub name: String, // 昵称/展示名字
    pub role: String, // "admin" 等角色标识
    pub exp: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct SiteConfig {
    pub index_title: String,
    pub meta_desc: String,
    pub custom_head: Option<String>,
    pub footer_html: Option<String>,
}

impl Default for SiteConfig {
    fn default() -> Self {
        Self {
            index_title: "Lily Nest example - 梨窝".to_string(),
            meta_desc: "梨窝 meta example".to_string(),
            custom_head: None,
            footer_html: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct NoteConfig {
    pub note_title: String,
    pub meta_desc: String,
    pub meta_keywords: String,
}

impl Default for NoteConfig {
    fn default() -> Self {
        Self {
            note_title: "梨记".to_string(),
            meta_desc: "记录每一次进步".to_string(),
            meta_keywords: "笔记, 博客, 梨记".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NoteFrontmatter {
    pub title: String,
    pub date: String,
    pub updated_at: Option<String>,
    pub slug: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub excerpt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteSummary {
    pub meta: NoteFrontmatter,
    pub filename: String,
    #[serde(default, skip_serializing)]
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminNoteSaveRequest {
    pub title: String,
    pub tags: Vec<String>,
    pub excerpt: Option<String>,
    pub content: String,
}
