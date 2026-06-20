use crate::model::{
    AboutList, AssetsConfig, ChangelogList, CloudflareConfig, HomeProfile, MarkdownConfig,
    ProjectList, SecurityConfig, ServerConfig, SiteConfig, TlsConfig,
};
use serde::de::DeserializeOwned;
use std::fs;
use tracing::error;

#[derive(serde::Deserialize)]
struct SiteToml {
    #[serde(default)]
    profile: HomeProfile,
    #[serde(default)]
    site: SiteConfig,
}

// ── 复用边界 ──────────────────────────────────────────

/// 读取整个 TOML 文件反序列化为 T（如 projects.toml → ProjectList）
fn load_toml_file<T: DeserializeOwned + Default>(path: &str, label: &str) -> T {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return T::default(),
    };
    toml::from_str::<T>(&content).unwrap_or_else(|e| {
        error!("解析 {} 失败: {}, 使用默认{}配置", path, e, label);
        T::default()
    })
}

/// 从 config.toml 中提取 [section] 并反序列化为 T
fn load_config_section<T: DeserializeOwned + Default>(section: &str, label: &str) -> T {
    let content = fs::read_to_string("config.toml").unwrap_or_default();
    let section_val = match toml::from_str::<toml::Value>(&content) {
        Ok(full) => match full.get(section) {
            Some(v) => v.clone(),
            None => return T::default(),
        },
        Err(e) => {
            if !content.is_empty() {
                error!("解析 config.toml 失败: {}, 使用默认{}配置", e, label);
            }
            return T::default();
        }
    };
    let section_str = toml::to_string(&section_val).unwrap_or_default();
    toml::from_str::<T>(&section_str).unwrap_or_else(|e| {
        error!("解析 [{}] 失败: {}, 使用默认{}配置", section, e, label);
        T::default()
    })
}

// ── 公开接口 ──────────────────────────────────────────

pub fn load_site_data() -> (HomeProfile, SiteConfig) {
    let content = match fs::read_to_string("site.toml") {
        Ok(c) => c,
        Err(e) => {
            error!("提示: 未找到 site.toml ({}), 使用内置默认配置", e);
            return (HomeProfile::default(), SiteConfig::default());
        }
    };
    match toml::from_str::<SiteToml>(&content) {
        Ok(c) => (c.profile, c.site),
        Err(e) => {
            error!("解析 site.toml 失败: {}. 请检查格式是否正确。", e);
            (HomeProfile::default(), SiteConfig::default())
        }
    }
}

pub fn load_site_profile() -> HomeProfile {
    load_site_data().0
}

pub fn load_projects() -> ProjectList {
    load_toml_file::<ProjectList>("projects.toml", "项目")
}

pub fn load_about_items() -> AboutList {
    load_toml_file::<AboutList>("about.toml", "关于")
}

pub fn load_changelog() -> ChangelogList {
    load_toml_file::<ChangelogList>("changelog.toml", "更新日志")
}

pub fn load_server_config() -> ServerConfig {
    load_config_section::<ServerConfig>("server", "服务器")
}

pub fn load_tls_config() -> Option<TlsConfig> {
    let content = fs::read_to_string("config.toml").ok()?;
    #[derive(serde::Deserialize)]
    struct Wrapper {
        tls: TlsConfig,
    }
    toml::from_str::<Wrapper>(&content).ok().map(|w| w.tls)
}

pub fn load_security_config() -> SecurityConfig {
    load_config_section::<SecurityConfig>("security", "安全")
}

pub fn load_assets_config() -> AssetsConfig {
    load_config_section::<AssetsConfig>("assets", "压缩")
}

pub fn load_cloudflare_config() -> CloudflareConfig {
    load_config_section::<CloudflareConfig>("cloudflare", "Cloudflare")
}

pub fn load_markdown_config() -> MarkdownConfig {
    load_config_section::<MarkdownConfig>("markdown", "Markdown")
}

pub async fn get_editable_configs() -> Vec<String> {
    let mut editable = Vec::new();
    if let Ok(mut entries) = tokio::fs::read_dir(".").await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(".toml") && name != "config.toml" && name != "Cargo.toml" {
                    editable.push(name.to_string());
                }
            }
        }
    }
    editable
}
