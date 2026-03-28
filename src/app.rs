use std::sync::Arc;

use axum::{
    Router,
    extract::State,
    response::{Html, IntoResponse, Redirect},
    routing::get,
};
use tokio::sync::RwLock;
use tower_http::services::{ServeDir, ServeFile};

use crate::config::{load_about_items, load_profile, load_projects};

pub struct AppState {
    pub html_cache: RwLock<String>,
}

pub fn build_app() -> Router {
    let api_routes = crate::routes::api::router();
    let state = Arc::new(AppState {
        html_cache: RwLock::new(render_index()),
    });

    Router::new()
        .route("/", get(handler_home_page))
        .with_state(state)
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
        .nest_service("/css", ServeDir::new("./static/css"))
        .nest_service("/js", ServeDir::new("./static/js"))
        .nest_service("/fonts", ServeDir::new("./static/fonts"))
}

fn render_index() -> String {
    let profile_data = load_profile();
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
        .map(|m| {
            format!(
                r#"<md-text-button>{}</md-text-button>"#,
                html_escape(m)
            )
        })
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
        <md-list-item type="button" href="{url}" target="_blank">
            <md-icon slot="start">code</md-icon>
            <div slot="headline">{name}</div>
            <div slot="supporting-text">{desc}</div>
            <md-icon slot="end">open_in_new</md-icon>
        </md-list-item>
        "#,
                divider = divider,
                url = html_escape(&proj.url),
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
            <img slot="start" src="{icon}" style="width: 24px; height: 24px;">
            <div slot="headline">{title}</div>
            <div slot="supporting-text">{content}</div>
        </md-list-item>
        "#,
                icon = html_escape(&item.icon_url),
                title = html_escape(&item.title),
                content = html_escape(&item.content)
            )
        })
        .collect::<String>();

    // 注入 JSON
    let profile_data_json = serde_json::to_string(&profile_data).unwrap();

    // 替换占位符
    html = html.replace("{{title}}", &html_escape(&profile_data.current_identity));
    html = html.replace("{{avatar}}", &html_escape(&profile_data.avatar_url));
    html = html.replace("{{bg}}", &html_escape(&profile_data.bg_url));
    html = html.replace("{{ver}}", &html_escape(&profile_data.site_version));
    html = html.replace("{{members_html}}", &members_html);
    html = html.replace("{{intro}}", &html_escape(&profile_data.intro));
    html = html.replace("{{p_json}}", &profile_data_json);
    // 注入项目 HTML
    html = html.replace("{{projects_html}}", &projects_html);
    html = html.replace("{{about_items_html}}", &about_items_html);

    html
}

async fn handler_home_page(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    if cfg!(debug_assertions) {
        // debug：每次请求都重算（开发爽）
        return Html(render_index());
    }
    // release：用缓存
    let cache = state.html_cache.read().await;
    Html(cache.clone())
}

// 简单转义：用于插入到 HTML 文本节点/属性里
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
