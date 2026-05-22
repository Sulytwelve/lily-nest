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
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,  // "ok"
    pub version: &'static str, // env!("CARGO_PKG_VERSION")
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
        }
    }
}

impl Default for HealthResponse {
    fn default() -> Self {
        Self {
            status: "ok",
            version: env!("CARGO_PKG_VERSION"),
        }
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
    #[serde(rename = "auth-ext-secq")]
    pub auth_ext_secq: Option<bool>,
    #[serde(rename = "auth-ext-warp")]
    pub auth_ext_warp: Option<bool>,
    #[serde(rename = "auth-ext-cftrace")]
    pub auth_ext_cftrace: Option<bool>,
    #[serde(rename = "cftrace-url")]
    pub cftrace_url: Option<String>,
    pub allowed_locs: Option<Vec<String>>,
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
            admin_security_answers: None,
            admin_security_questions: Some(vec![
                "我的暗恋对象是谁".to_string(),
                "我喜欢吃什么".to_string(),
                "我的专业是什么".to_string(),
            ]),
            auth_ext_secq: Some(false),
            auth_ext_warp: Some(false),
            auth_ext_cftrace: Some(false),
            cftrace_url: Some("https://cloudflare.com/cdn-cgi/trace".to_string()),
            allowed_locs: Some(vec!["CN".to_string()]),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct AssetsConfig {
    pub assets_dirs: Vec<String>,
    pub target_exts: Vec<String>,
    pub compression_types: Vec<String>,
    pub zstd_level: i32,
    pub brotli_quality: u32,
    pub gzip_level: u32,
}

impl Default for AssetsConfig {
    fn default() -> Self {
        Self {
            assets_dirs: vec![
                "./static/css".to_string(),
                "./static/js".to_string(),
                "./static/fonts".to_string(),
            ],
            target_exts: vec![
                "css".to_string(),
                "js".to_string(),
                "woff".to_string(),
                "woff2".to_string(),
                "ttf".to_string(),
                "otf".to_string(),
            ],
            compression_types: vec!["br".to_string(), "gz".to_string()],
            zstd_level: 3,
            brotli_quality: 4,
            gzip_level: 8,
        }
    }
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
pub struct AuthConfigResponse {
    pub auth_ext_secq: bool,
    pub auth_ext_cftrace: bool,
    pub cftrace_url: Option<String>,
    pub security_questions: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct SiteConfig {
    pub index_title: String,
    pub meta_desc: String,
}

impl Default for SiteConfig {
    fn default() -> Self {
        Self {
            index_title: "Lily Nest example - 梨窝".to_string(),
            meta_desc: "梨窝 meta example".to_string(),
        }
    }
}

