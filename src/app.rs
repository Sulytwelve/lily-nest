use std::{collections::HashMap, sync::Arc, time::SystemTime};

use axum::{Router, middleware};
use rand::RngCore;
use tokio::sync::{Mutex, RwLock};

use crate::{
    config::load_security_config,
    middlewares::{build_cors_layer, security_headers},
    routes,
    state::AppState,
};

pub fn build_app() -> Router {
    let security_config = load_security_config();
    let assets_config = crate::config::load_assets_config();
    let markdown_config = crate::config::load_markdown_config();

    // 每次启动生成新的随机 secret，重启后所有 JWT 自动失效
    let mut jwt_secret = vec![0u8; 64];
    rand::rng().fill_bytes(&mut jwt_secret);

    let markdown_bytes = if markdown_config.enable {
        crate::render::render_index_markdown().into()
    } else {
        bytes::Bytes::new()
    };

    let state = Arc::new(AppState {
        html_cache: RwLock::new(crate::state::HtmlCache::new(
            SystemTime::now(),
            crate::render::render_index().into(),
            markdown_bytes,
            assets_config.html_cache_seconds,
        )),
        security_config: Arc::new(security_config),
        assets_config: Arc::new(assets_config),
        markdown_config: Arc::new(markdown_config),
        auth_rate_limiter: Mutex::new(HashMap::new()),
        jwt_secret,
    });


    let cors = build_cors_layer(&state.security_config);

    let api_routes = routes::api::router(state.clone()).layer(cors);

    let app_routes = routes::home::router(state.clone())
        .nest("/api/v1", api_routes)
        .merge(routes::admin::router(state.clone()))
        .layer(middleware::from_fn_with_state(state, security_headers));

    app_routes.merge(routes::static_assets::router())
}

