use std::{collections::HashMap, sync::Arc, time::SystemTime};

use axum::{Router, middleware};
use rand::Rng;
use tokio::sync::{Mutex, RwLock};

use crate::{
    config::load_security_config,
    middlewares::{build_cors_layer, security_headers},
    routes,
    state::AppState,
};

pub async fn build_app() -> Router {
    let security_config = load_security_config();
    let assets_config = crate::config::load_assets_config();
    let markdown_config = crate::config::load_markdown_config();

    // 优先从环境变量 LILY_JWT_SECRET 或本地 .jwt_secret 文件加载，保证重启后会话持久
    let jwt_secret = if let Ok(sec) = std::env::var("LILY_JWT_SECRET") {
        let bytes = sec.into_bytes();
        if bytes.len() < 32 {
            tracing::error!(
                "LILY_JWT_SECRET is too short ({} bytes); refusing to start with a weak JWT secret",
                bytes.len()
            );
            panic!("LILY_JWT_SECRET must be at least 32 bytes");
        }
        bytes
    } else {
        let secret_path = std::path::Path::new(".jwt_secret");
        if secret_path.exists() {
            match std::fs::read(secret_path) {
                Ok(sec) if sec.len() >= 32 => sec,
                Ok(short) => {
                    tracing::warn!(
                        ".jwt_secret is too short ({} bytes); regenerating a new secret",
                        short.len()
                    );
                    let mut new_sec = vec![0u8; 64];
                    rand::rng().fill_bytes(&mut new_sec);
                    if let Err(e) = std::fs::write(secret_path, &new_sec) {
                        tracing::warn!("failed to write regenerated .jwt_secret: {e}");
                    }
                    new_sec
                }
                Err(e) => {
                    tracing::warn!("failed to read .jwt_secret ({}); generating a new one", e);
                    let mut new_sec = vec![0u8; 64];
                    rand::rng().fill_bytes(&mut new_sec);
                    if let Err(e) = std::fs::write(secret_path, &new_sec) {
                        tracing::warn!("failed to write .jwt_secret: {e}");
                    }
                    new_sec
                }
            }
        } else {
            let mut new_sec = vec![0u8; 64];
            rand::rng().fill_bytes(&mut new_sec);
            if let Err(e) = std::fs::write(secret_path, &new_sec) {
                tracing::warn!("failed to write .jwt_secret: {e}");
            }
            new_sec
        }
    };

    // 优先从环境变量 LILY_AGENT_PUB_KEY 或本地 .agent.pub 读取 Agent 公钥
    let agent_pub_key = std::env::var("LILY_AGENT_PUB_KEY")
        .ok()
        .map(|s| s.into_bytes())
        .or_else(|| std::fs::read(".agent.pub").ok());

    let markdown_bytes = if markdown_config.enable {
        crate::render::render_index_markdown().into()
    } else {
        bytes::Bytes::new()
    };

    let state = Arc::new(AppState {
        agent_pub_key,
        html_cache: RwLock::new(crate::state::HtmlCache::new(
            SystemTime::now(),
            crate::render::render_index().into(),
            markdown_bytes,
            assets_config.html_cache_seconds,
        )),
        security_config: Arc::new(security_config),
        assets_config: Arc::new(assets_config),
        markdown_config: Arc::new(markdown_config),
        cloudflare_config: Arc::new(crate::config::load_cloudflare_config()),
        auth_rate_limiter: Mutex::new(HashMap::new()),
        jwt_secret,
        note_index: RwLock::new(crate::note_loader::load_all_notes().await),
        note_html_cache: RwLock::new(HashMap::new()),
        note_list_html_cache: RwLock::new(None),
    });

    let cors = build_cors_layer(&state.security_config);

    let api_routes = routes::api::router(state.clone()).layer(cors);

    let app_routes = routes::home::router(state.clone())
        .nest("/api/v1", api_routes)
        .merge(routes::admin::router(state.clone()))
        .merge(routes::note::router(state.clone()))
        .merge(routes::note_admin::router(state.clone()))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            security_headers,
        ));

    let static_routes = routes::static_assets::router()
        .layer(middleware::from_fn_with_state(state, security_headers));

    app_routes.merge(static_routes)
}
