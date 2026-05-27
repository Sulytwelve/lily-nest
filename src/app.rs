use std::{collections::HashMap, sync::Arc, time::SystemTime};

use axum::{Router, middleware};
use tokio::sync::{Mutex, RwLock};

use crate::{
    config::load_security_config,
    middlewares::{build_cors_layer, security_headers},
    routes,
    state::AppState,
};

pub fn build_app() -> Router {
    let security_config = load_security_config();
    let state = Arc::new(AppState {
        html_cache: RwLock::new(crate::render::render_index()),
        security_config,
        started_at: RwLock::new(SystemTime::now()),
        auth_rate_limiter: Mutex::new(HashMap::new()),
    });

    let cors = build_cors_layer(&state.security_config);

    let api_routes = routes::api::router(state.clone()).layer(cors);

    let app_routes = routes::home::router(state.clone())
        .nest("/api/v1", api_routes)
        .merge(routes::admin::router(state.clone()))
        .layer(middleware::from_fn_with_state(state, security_headers));

    app_routes.merge(routes::static_assets::router())
}
