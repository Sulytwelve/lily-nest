use crate::model::{AboutList, AssetsConfig, HomeProfile, ProjectList, SecurityConfig, TlsConfig, SiteConfig, ServerConfig};
use serde::Deserialize;
use std::fs;
use tracing::error;

#[derive(Deserialize)]
struct SiteToml {
    #[serde(default)]
    profile: HomeProfile,
    #[serde(default)]
    site: SiteConfig,
}

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
    // 尝试读取 projects.toml
    let content = match std::fs::read_to_string("projects.toml") {
        Ok(s) => s,
        Err(_) => return ProjectList::default(), // 找不到文件，直接给默认值
    };

    // 尝试解析 TOML 内容
    match toml::from_str::<ProjectList>(&content) {
        Ok(list) => list,
        Err(e) => {
            error!("解析 projects.toml 失败: {}, 使用默认配置", e);
            ProjectList::default()
        }
    }
}

pub fn load_about_items() -> AboutList {
    // 尝试读取 about.toml
    let content = match std::fs::read_to_string("about.toml") {
        Ok(s) => s,
        Err(_) => return AboutList::default(), // 找不到文件，直接给默认值
    };

    // 尝试解析 TOML 内容
    match toml::from_str::<AboutList>(&content) {
        Ok(list) => list,
        Err(e) => {
            error!("解析 about.toml 失败: {}, 使用默认配置", e);
            AboutList::default()
        }
    }
}

pub fn load_server_config() -> ServerConfig {
    let content = std::fs::read_to_string("config.toml").unwrap_or_default();

    #[derive(Deserialize)]
    struct Wrapper {
        server: ServerConfig,
    }

    toml::from_str::<Wrapper>(&content)
        .map(|w| w.server)
        .unwrap_or_else(|e| {
            if !content.is_empty() {
                error!("警告: server 配置解析失败 ({}), 使用默认服务器配置", e);
            }
            ServerConfig::default()
        })
}

pub fn load_tls_config() -> Option<TlsConfig> {
    let content = fs::read_to_string("config.toml").ok()?;
    // 局部 Wrapper，以便提取 [tls] 节
    #[derive(Deserialize)]
    struct Wrapper {
        tls: TlsConfig,
    }
    toml::from_str::<Wrapper>(&content).ok().map(|w| w.tls)
}

pub fn load_security_config() -> SecurityConfig {
    let content = std::fs::read_to_string("config.toml").unwrap_or_default();

    #[derive(Deserialize)]
    struct Wrapper {
        security: SecurityConfig,
    }

    toml::from_str::<Wrapper>(&content)
        .map(|w| w.security)
        .unwrap_or_else(|e| {
            if !content.is_empty() {
                error!("警告: security 配置解析失败 ({}), 使用默认安全策略", e);
            }
            SecurityConfig::default()
        })
}

pub fn load_assets_config() -> AssetsConfig {
    let content = std::fs::read_to_string("config.toml").unwrap_or_default();

    #[derive(Deserialize)]
    struct Wrapper {
        assets: AssetsConfig,
    }

    toml::from_str::<Wrapper>(&content)
        .map(|w| w.assets)
        .unwrap_or_else(|e| {
            if !content.is_empty() {
                error!("警告: assets 配置解析失败 ({}), 使用默认压缩配置", e);
            }
            AssetsConfig::default()
        })
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
