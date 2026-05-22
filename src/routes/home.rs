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
    let started_at = *state.started_at.read().await;

    // debug 模式下每次重新渲染，不做缓存
    if cfg!(debug_assertions) {
        let html = crate::render::render_index();
        let res = (
            [(header::CACHE_CONTROL, "no-cache")],
            Html(html),
        )
            .into_response();
        return res;
    }

    // release: 检查 If-Modified-Since
    if let Some(ims) = req.headers().get(header::IF_MODIFIED_SINCE) {
        if let Some(ims_time) = ims.to_str().ok().and_then(|v| httpdate::parse_http_date(v).ok()) {
            if ims_time >= started_at {
                let mut res = Response::new(axum::body::Body::empty());
                *res.status_mut() = StatusCode::NOT_MODIFIED;
                res.headers_mut().insert(
                    header::CACHE_CONTROL,
                    HeaderValue::from_static("public, max-age=300"),
                );
                return res;
            }
        }
    }

    let html = state.html_cache.read().await.clone();
    let mut res = (
        [(header::CACHE_CONTROL, "public, max-age=300")],
        Html(html),
    )
        .into_response();
    res.headers_mut().insert(
        header::LAST_MODIFIED,
        HeaderValue::from_str(&httpdate::fmt_http_date(started_at))
            .unwrap_or_else(|_| HeaderValue::from_static("Thu, 01 Jan 1970 00:00:00 GMT")),
    );
    res
}
