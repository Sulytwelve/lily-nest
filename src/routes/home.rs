use std::sync::Arc;

use axum::{
    Router,
    extract::{Query, State},
    http::{HeaderValue, StatusCode, header},
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
};
use serde::Deserialize;

use crate::state::AppState;

#[derive(Deserialize, Default)]
struct HomeQuery {
    format: Option<String>,
}

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(handler_home_page))
        .route("/index.html", get(|| async { Redirect::permanent("/") }))
        .with_state(state)
}

async fn handler_home_page(
    State(state): State<Arc<AppState>>,
    Query(query): Query<HomeQuery>,
    req: axum::extract::Request,
) -> Response {
    let wants_markdown = state.markdown_config.enable
        && (query.format.as_deref() == Some("markdown")
            || req
                .headers()
                .get(header::ACCEPT)
                .and_then(|v| v.to_str().ok())
                .map(|v| v.contains("text/markdown") || v.contains("text/x-markdown"))
                .unwrap_or(false));

    if wants_markdown {
        let mut res = serve_markdown_or_debug(state, req).await;
        res.headers_mut().insert(
            header::CACHE_CONTROL,
            HeaderValue::from_static("private, no-cache, no-store, must-revalidate"),
        );
        res.headers_mut()
            .insert(header::VARY, HeaderValue::from_static("Accept"));
        return res;
    }

    if cfg!(debug_assertions) {
        let html = tokio::task::spawn_blocking(crate::render::render_index)
            .await
            .unwrap_or_else(|e| {
                tracing::error!("render_index panicked in spawn_blocking: {e}");
                String::new()
            });
        let mut res = (
            [
                (header::CACHE_CONTROL, "no-cache"),
                (
                    header::LINK,
                    "</?format=markdown>; rel=\"alternate\"; type=\"text/markdown\"",
                ),
            ],
            Html(html),
        )
            .into_response();
        res.headers_mut()
            .insert(header::VARY, HeaderValue::from_static("Accept"));
        return res;
    }

    let cache = {
        let lock = state.html_cache.read().await;
        lock.clone()
    };

    if let Some(res) = check_304(&req, &cache) {
        return res;
    }

    let mut res = serve_cache_response(cache.body.clone(), "text/html; charset=utf-8", &cache);
    if state.markdown_config.enable {
        res.headers_mut().insert(
            header::LINK,
            HeaderValue::from_static(
                "</?format=markdown>; rel=\"alternate\"; type=\"text/markdown\"",
            ),
        );
    }
    res
}

async fn serve_markdown_or_debug(state: Arc<AppState>, req: axum::extract::Request) -> Response {
    if cfg!(debug_assertions) {
        let md = tokio::task::spawn_blocking(crate::render::render_index_markdown)
            .await
            .unwrap_or_else(|e| {
                tracing::error!("render_index_markdown panicked in spawn_blocking: {e}");
                String::new()
            });
        let mut res = Response::new(axum::body::Body::from(md));
        res.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/markdown; charset=utf-8"),
        );
        res.headers_mut()
            .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
        return res;
    }

    let cache = {
        let lock = state.html_cache.read().await;
        lock.clone()
    };

    if let Some(res) = check_304(&req, &cache) {
        return res;
    }

    serve_cache_response(
        cache.markdown_body.clone(),
        "text/markdown; charset=utf-8",
        &cache,
    )
}

fn check_304(req: &axum::extract::Request, cache: &crate::state::HtmlCache) -> Option<Response> {
    if let Some(ims) = req.headers().get(header::IF_MODIFIED_SINCE)
        && let Some(ims_time) = ims.to_str().ok().and_then(crate::utils::parse_http_date)
    {
        let ims_secs = ims_time
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let started_at_secs = cache
            .started_at
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        if ims_secs >= started_at_secs {
            let mut res = Response::new(axum::body::Body::empty());
            *res.status_mut() = StatusCode::NOT_MODIFIED;
            res.headers_mut()
                .insert(header::CACHE_CONTROL, cache.cache_control.clone());
            res.headers_mut()
                .insert(header::VARY, HeaderValue::from_static("Accept"));
            return Some(res);
        }
    }
    None
}

fn serve_cache_response(
    body: bytes::Bytes,
    content_type: &'static str,
    cache: &crate::state::HtmlCache,
) -> Response {
    let mut res = Response::new(axum::body::Body::from(body));
    let headers = res.headers_mut();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
    headers.insert(header::CACHE_CONTROL, cache.cache_control.clone());
    headers.insert(header::LAST_MODIFIED, cache.http_date.clone());
    headers.insert(header::VARY, HeaderValue::from_static("Accept"));
    res
}
