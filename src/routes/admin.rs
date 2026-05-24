use std::sync::Arc;
use axum::{
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

async fn admin_page_handler() -> impl IntoResponse {
    let html = tokio::fs::read_to_string("templates/admin.html")
        .await
        .unwrap_or_else(|_| {
            "<!doctype html><html><body><h1>templates/admin.html not found</h1></body></html>".to_string()
        });
    Html(html)
}
