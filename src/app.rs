use std::sync::Arc;

use crate::{
    config::{load_about_items, load_projects, load_security_config, load_site_profile},
    model::SecurityConfig,
};
use axum::{
    Router,
    extract::{Request, State},
    http::{HeaderName, HeaderValue, Method, header},
    middleware::{self, Next},
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
};
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
use tracing::error;

pub struct AppState {
    pub html_cache: RwLock<String>,
    pub security_config: SecurityConfig,
}

pub fn build_app() -> Router {
    let api_routes = crate::routes::api::router();
    let security_config = load_security_config();
    let state = Arc::new(AppState {
        html_cache: RwLock::new(render_index()),
        security_config,
    });

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE]);

    let cors = if state
        .security_config
        .allow_origins
        .contains(&"*".to_string())
    {
        cors.allow_origin(Any)
    } else {
        let origins: Vec<HeaderValue> = state
            .security_config
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
    };

    Router::new()
        .route("/", get(handler_home_page))
        .with_state(state.clone())
        .route("/index.html", get(|| async { Redirect::permanent("/") }))
        .nest("/api/v1", api_routes)
        .route_service("/robots.txt", ServeFile::new("./static/robots.txt"))
        .route_service(
            "/BingSiteAuth.xml",
            ServeFile::new("./static/BingSiteAuth.xml"),
        )
        .route_service("/sitemap.xml", ServeFile::new("./static/sitemap.xml"))
        .route_service("/favicon.ico", ServeFile::new("./static/favicon.ico"))
        .nest_service("/images", ServeDir::new("./static/images"))
        .nest_service(
            "/css",
            ServeDir::new("./static/css")
                .precompressed_br()
                .precompressed_gzip(),
        )
        .nest_service(
            "/js",
            ServeDir::new("./static/js")
                .precompressed_br()
                .precompressed_gzip(),
        )
        .nest_service(
            "/fonts",
            ServeDir::new("./static/fonts")
                .precompressed_br()
                .precompressed_gzip(),
        )
        .layer(cors)
        .layer(middleware::from_fn_with_state(
            state.clone(),
            security_headers,
        ))
        .layer(middleware::from_fn(static_asset_cache_control))
}

async fn security_headers(
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

async fn static_asset_cache_control(req: Request, next: Next) -> Response {
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

    response
}

fn sanitize_url(url: &str) -> &str {
    let url = url.trim();
    if url.starts_with("http://") || url.starts_with("https://") || url.starts_with('/') {
        url
    } else {
        "#projects"
    }
}

fn render_index() -> String {
    let profile_data = load_site_profile();
    let projects_data = load_projects();
    let about_data = load_about_items();

    let mut html = std::fs::read_to_string("templates/index.html").unwrap_or_else(|_| {
        "<!doctype html><html><body><h1>templates/index.html not found</h1></body></html>"
            .to_string()
    });
    // 1. 组装成员
    let members_html = profile_data
        .team_members
        .iter()
        .map(|m| format!(r#"<md-text-button>{}</md-text-button>"#, html_escape(m)))
        .collect::<String>();

    // 2. 组装项目预览
    let projects_html = projects_data
        .items
        .iter()
        .enumerate()
        .map(|(i, proj)| {
            // 如果不是第一个元素，在前面加一个分割线
            let divider = if i > 0 {
                "<md-divider></md-divider>"
            } else {
                ""
            };

            format!(
                r#"{divider}
                  <md-list-item type="button" href="{url}" target="_blank" rel="noopener">
                    <md-icon slot="start">
                      <svg style="height: 48px; width: 48x" viewBox="0 -960 960 960">
                        <path
                          d="M320-240 80-480l240-240 57 57-184 184 183 183-56 56Zm320 0-57-57 184-184-183-183 56-56 240 240-240 240Z"
                        />
                      </svg>
                    </md-icon>
                    <div slot="headline">{name}</div>
                    <div slot="supporting-text">{desc}</div>
                    <md-icon slot="end">
                      <svg style="height: 48px; width: 48x" viewBox="0 -960 960 960">
                        <path
                          d="M200-120q-33 0-56.5-23.5T120-200v-560q0-33 23.5-56.5T200-840h280v80H200v560h560v-280h80v280q0 33-23.5 56.5T760-120H200Zm188-212-56-56 372-372H560v-80h280v280h-80v-144L388-332Z"
                        />
                      </svg>
                    </md-icon>
                  </md-list-item>
                "#,
                divider = divider,
                url = html_escape(sanitize_url(&proj.url)),
                name = html_escape(&proj.name),
                desc = html_escape(&proj.desc)
            )
        })
        .collect::<String>();

    // 3. 关于我
    let about_items_html = about_data
        .items
        .iter()
        .map(|item| {
            format!(
                r#"
        <md-list-item>
            <img slot="start" src="{icon}" style="width: 24px; height: 24px; border-radius: 50%;" alt="{title}">
            <div slot="headline">{title}</div>
            <div slot="supporting-text">{content}</div>
        </md-list-item>
        "#,
                icon = html_escape(sanitize_url(&item.icon_url)),
                title = html_escape(&item.title),
                content = html_escape(&item.content)
            )
        })
        .collect::<String>();

    // 替换占位符
    html = html.replace("{{title}}", &html_escape(&profile_data.current_identity));
    html = html.replace(
        "{{avatar}}",
        &html_escape(sanitize_url(&profile_data.avatar_url)),
    );
    html = html.replace("{{bg}}", &html_escape(sanitize_url(&profile_data.bg_url)));
    html = html.replace("{{ver}}", &html_escape(&profile_data.site_version));
    html = html.replace("{{members_html}}", &members_html);
    html = html.replace("{{intro}}", &html_escape(&profile_data.intro));
    // 注入项目 HTML
    html = html.replace("{{projects_html}}", &projects_html);
    html = html.replace("{{about_items_html}}", &about_items_html);

    html
}

async fn handler_home_page(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let html = if cfg!(debug_assertions) {
        render_index()
    } else {
        state.html_cache.read().await.clone()
    };

    (
        [(axum::http::header::CACHE_CONTROL, "public, max-age=300")],
        Html(html),
    )
}

// 简单转义：用于插入到 HTML 文本节点/属性里
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
