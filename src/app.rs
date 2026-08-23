use std::{collections::HashMap, sync::Arc, time::SystemTime};

use axum::{Router, middleware};
use rand::Rng;
use tokio::sync::{Mutex, RwLock};

use crate::{
    config::load_security_config,
    middlewares::{build_cors_layer, security_headers},
    model::AssetsConfig,
    routes,
    state::AppState,
};

/// 在 Unix 上把 `.jwt_secret` 权限收紧为 `0600`，防止同机其他用户读取（B9）。
#[cfg(unix)]
fn set_owner_only_permissions(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;
    if let Err(e) = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)) {
        tracing::error!("failed to chmod 0600 on {}: {}", path.display(), e);
    }
}

#[cfg(not(unix))]
fn set_owner_only_permissions(_path: &std::path::Path) {}

fn random_setup_code() -> String {
    let mut bytes = [0u8; 4];
    rand::rng().fill_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

pub async fn build_app(assets_config: AssetsConfig) -> Router {
    let security_config = load_security_config();
    let auth_secrets = crate::secrets::load_auth_secrets(&security_config);
    let setup_code = if auth_secrets.admin_password_hash.is_none() {
        let code = random_setup_code();
        tracing::warn!(
            "管理员未初始化。请运行 lily-nest set-password 或访问 /admin 输入 Setup Code: {}（10 分钟内有效）",
            code
        );
        Some((code, std::time::Instant::now()))
    } else {
        None
    };
    let assets_config = Arc::new(assets_config);
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
                    tracing::error!(
                        ".jwt_secret is too short ({} bytes); refusing to start. \
                         Delete the file or set LILY_JWT_SECRET to a >=32 byte value.",
                        short.len()
                    );
                    panic!(".jwt_secret must be at least 32 bytes");
                }
                Err(e) => {
                    tracing::error!("failed to read .jwt_secret ({}); refusing to start", e);
                    panic!("failed to read .jwt_secret: {e}");
                }
            }
        } else {
            let mut new_sec = vec![0u8; 64];
            rand::rng().fill_bytes(&mut new_sec);
            if let Err(e) = std::fs::write(secret_path, &new_sec) {
                tracing::error!("failed to write .jwt_secret ({}); refusing to start", e);
                panic!("failed to write .jwt_secret: {e}");
            }
            set_owner_only_permissions(secret_path);
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
        auth_secrets: RwLock::new(auth_secrets),
        setup_code: Mutex::new(setup_code),
        assets_config: assets_config.clone(),
        markdown_config: Arc::new(markdown_config),
        cloudflare_config: Arc::new(crate::config::load_cloudflare_config()),
        auth_rate_limiter: crate::state::RateLimitTable::new(16, 10_000),
        revoked_jtis: Mutex::new(HashMap::new()),
        jwt_secret,
        note_index: RwLock::new(crate::note_loader::load_all_notes().await),
        note_html_cache: RwLock::new(HashMap::new()),
        note_list_html_cache: RwLock::new(None),
    });

    let cors = build_cors_layer(&state.security_config);

    let api_public = routes::api::public_router(state.clone()).layer(cors);
    let api_sensitive = routes::api::sensitive_router(state.clone());
    let api_admin = routes::api::admin_router(state.clone());
    let api_routes = api_public.merge(api_sensitive).merge(api_admin);

    let app_routes = routes::home::router(state.clone())
        .nest("/api/v1", api_routes)
        .merge(routes::admin::router(state.clone()))
        .merge(routes::note::router(state.clone()))
        // B29：`/admin/notes*` 与 `/api/v1/notes*` 是 Agent 机器调用专线，
        // 走 note_auth_middleware 的 Bearer JWT 校验，刻意不挂 CORS。
        .merge(routes::note_admin::router(state.clone()))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            security_headers,
        ));

    let static_routes = routes::static_assets::router(assets_config)
        .layer(middleware::from_fn_with_state(state, security_headers));

    app_routes.merge(static_routes)
}
