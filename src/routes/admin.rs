use std::sync::Arc;
use axum::{
    extract::State,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use crate::state::AppState;

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/admin", get(admin_page_handler))
        .with_state(state)
}

async fn admin_page_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let html = tokio::fs::read_to_string("templates/admin.html")
        .await
        .unwrap_or_else(|_| {
            "<!doctype html><html><body><h1>templates/admin.html not found</h1></body></html>".to_string()
        });

    let security_config = if cfg!(debug_assertions) {
        crate::config::load_security_config()
    } else {
        state.security_config.clone()
    };

    let auth_config = serde_json::json!({
        "auth_ext_secq": security_config.auth_ext_secq.unwrap_or(false),
        "auth_ext_cftrace": security_config.auth_ext_cftrace.unwrap_or(false),
        "cftrace_url": security_config.cftrace_url,
        "security_questions": security_config.admin_security_questions,
    });

    Html(html.replace("{{auth_config_json}}", &auth_config.to_string()))
}
