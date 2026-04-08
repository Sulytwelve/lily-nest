use crate::model::{AboutList, AssetsConfig, HomeProfile, ProjectList, SecurityConfig, TlsConfig};
use serde::Deserialize;
use std::fs;
use tracing::error;

pub fn load_site_profile() -> HomeProfile {
    // 1. 尝试读取文件
    let content = match fs::read_to_string("site.toml") {
        Ok(c) => c,
        Err(e) => {
            error!("提示: 未找到 site.toml ({}), 使用内置默认配置", e);
            return HomeProfile::default();
        }
    };

    #[derive(Deserialize)]
    struct Config {
        site: HomeProfile,
    }

    // 2. 尝试解析
    toml::from_str::<Config>(&content)
        .map(|c| c.site)
        .unwrap_or_else(|e| {
            // 这里会打印出具体的错误：是少了双引号或字段类型对不上
            // 配合 `model.rs` 的#[serde(default)]，缺失字段不会报错，只有“类型错误”或“格式错误”才会走到这里
            error!("解析 site.toml 失败: {}. 请检查格式是否正确。", e);
            HomeProfile::default()
        })
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
