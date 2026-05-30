use std::sync::Arc;

use axum::{
    Router,
    extract::State,
    http::{HeaderValue, header, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
};

use crate::state::AppState;

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(handler_home_page))
        .route("/index.html", get(|| async { Redirect::permanent("/") }))
        .with_state(state)
}

async fn handler_home_page(
    State(state): State<Arc<AppState>>,
    req: axum::extract::Request,
) -> Response {
    let cache = {
        let lock = state.html_cache.read().await;
        lock.clone()
    };

    // debug 模式下每次重新渲染，不做缓存
    if cfg!(debug_assertions) {
        let html = tokio::task::spawn_blocking(crate::render::render_index)
            .await
            .unwrap_or_else(|e| {
                tracing::error!("render_index panicked in spawn_blocking: {e}");
                String::new()
            });
        let res = (
            [(header::CACHE_CONTROL, "no-cache")],
            Html(html),
        )
            .into_response();
        return res;
    }

    if let Some(ims) = req.headers().get(header::IF_MODIFIED_SINCE) {
        if let Some(ims_time) = ims.to_str().ok().and_then(|v| httpdate::parse_http_date(v).ok()) {
            let ims_secs = ims_time.duration_since(std::time::SystemTime::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
            let started_at_secs = cache.started_at.duration_since(std::time::SystemTime::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
            if ims_secs >= started_at_secs {
                let mut res = Response::new(axum::body::Body::empty());
                *res.status_mut() = StatusCode::NOT_MODIFIED;
                res.headers_mut().insert(
                    header::CACHE_CONTROL,
                    cache.cache_control.clone(),
                );
                return res;
            }
        }
    }

    let mut res = Response::new(axum::body::Body::from(cache.body));
    let headers = res.headers_mut();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/html; charset=utf-8"),
    );
    headers.insert(
        header::CACHE_CONTROL,
        cache.cache_control,
    );
    headers.insert(
        header::LAST_MODIFIED,
        cache.http_date,
    );
    res
}
