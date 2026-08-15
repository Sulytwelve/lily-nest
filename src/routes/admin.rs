use crate::state::AppState;
use axum::{
    Router,
    extract::State,
    http::HeaderName,
    response::{Html, IntoResponse},
    routing::get,
};
use std::sync::Arc;

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/admin", get(admin_page_handler))
        .with_state(state)
}

async fn admin_page_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let html = tokio::fs::read_to_string("templates/admin.html")
        .await
        .unwrap_or_else(|_| {
            "<!doctype html><html><body><h1>templates/admin.html not found</h1></body></html>"
                .to_string()
        });

    let security_config = if cfg!(debug_assertions) {
        std::sync::Arc::new(
            tokio::task::spawn_blocking(crate::config::load_security_config)
                .await
                .unwrap_or_else(|e| {
                    tracing::error!("load_security_config panicked in spawn_blocking: {}", e);
                    crate::model::SecurityConfig::default()
                }),
        )
    } else {
        state.security_config.clone()
    };

    let auth_config = serde_json::json!({
        "auth_ext_secq": security_config.auth_ext_secq.unwrap_or(false),
        "auth_ext_cftrace": security_config.auth_ext_cftrace.unwrap_or(false),
        "cftrace_url": security_config.cftrace_url,
        "security_questions": security_config.admin_security_questions,
    });

    let auth_config_str = auth_config.to_string().replace("</", "<\\/");

    let body = Html(html.replace("{{auth_config_json}}", &auth_config_str));
    (
        [
            (
                HeaderName::from_static("cache-control"),
                "no-store, no-cache, must-revalidate",
            ),
            (HeaderName::from_static("pragma"), "no-cache"),
            (HeaderName::from_static("expires"), "0"),
            (HeaderName::from_static("x-robots-tag"), "noindex, nofollow"),
        ],
        body,
    )
}
