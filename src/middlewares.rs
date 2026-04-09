use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::{HeaderName, HeaderValue, Method, header},
    middleware::Next,
    response::Response,
};
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use tracing::error;

use crate::{config::load_security_config, model::SecurityConfig};

pub struct AppState {
    pub html_cache: RwLock<String>,
    pub security_config: SecurityConfig,
}

pub fn build_cors_layer(security_config: &SecurityConfig) -> CorsLayer {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE]);

    if security_config.allow_origins.contains(&"*".to_string()) {
        cors.allow_origin(Any)
    } else {
        let origins: Vec<HeaderValue> = security_config
            .allow_origins
            .iter()
            .filter_map(|o| match o.parse() {
                Ok(v) => Some(v),
                Err(_) => {
                    error!("[security] 解析失败, 非法的配置: {}", o);
                    None
                }
            })
            .collect();
        cors.allow_origin(origins)
    }
}

pub async fn security_headers(
    State(state): State<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Response {
    let mut res = next.run(req).await;

    let config = if cfg!(debug_assertions) {
        load_security_config()
    } else {
        state.security_config.clone()
    };

    // 用 (HeaderName, String) 而不是 (HeaderName, &'static str)
    let headers: [(HeaderName, String); 6] = [
        (header::CONTENT_SECURITY_POLICY, config.csp_policy),
        (header::X_CONTENT_TYPE_OPTIONS, "nosniff".into()),
        (
            header::REFERRER_POLICY,
            "strict-origin-when-cross-origin".into(),
        ),
        (header::X_FRAME_OPTIONS, "DENY".into()),
        (
            header::STRICT_TRANSPORT_SECURITY,
            "max-age=31536000; includeSubDomains".into(),
        ),
        (
            HeaderName::from_static("permissions-policy"),
            config.permissions_policy,
        ),
    ];

    let headers_map = res.headers_mut();
    for (name, value) in headers {
        if let Ok(v) = HeaderValue::from_str(&value) {
            headers_map.insert(name, v);
        }
    }

    res
}

pub async fn static_asset_cache_control(req: Request, next: Next) -> Response {
    let path = req.uri().path().to_owned();
    let mut response = next.run(req).await;

    if !response.status().is_success() {
        return response;
    }

    let cache_control = if path.starts_with("/fonts/") {
        Some("public, max-age=604800")
    } else if path.starts_with("/css/") || path.starts_with("/js/") || path.starts_with("/images/")
    {
        Some("public, max-age=86400")
    } else if matches!(
        path.as_str(),
        "/favicon.ico" | "/robots.txt" | "/sitemap.xml" | "/BingSiteAuth.xml"
    ) {
        Some("public, max-age=3600")
    } else {
        None
    };

    if let Some(cache_control) = cache_control {
        response.headers_mut().insert(
            axum::http::header::CACHE_CONTROL,
            HeaderValue::from_static(cache_control),
        );
    }

    // Cloudflare 可能需要的 Vary
    if path.starts_with("/fonts/")
        || path.starts_with("/css/")
        || path.starts_with("/js/")
        || path.starts_with("/images/")
    {
        response.headers_mut().insert(
            axum::http::header::VARY,
            HeaderValue::from_static("Accept-Encoding"),
        );
    }

    response
}
