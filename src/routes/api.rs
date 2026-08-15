use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Path, State},
    http::{HeaderValue, StatusCode, header},
    middleware,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{error, info};

use crate::{
    config::{get_editable_configs, load_site_profile},
    middlewares::handle_admin_login,
    model::{ConfigFile, HealthResponse, HomeProfile, SaveConfigRequest},
    state::AppState,
};

pub fn router(state: Arc<AppState>) -> Router {
    let public_routes = Router::new()
        .route("/home/profile", get(get_home_profile))
        .route("/health", get(health_handler))
        // 登录端点：不在 admin_auth_middleware 范围内，但有 rate limit
        .route("/admin/login", post(handle_admin_login));

    let admin_routes = Router::new()
        .route("/admin/configs", get(list_configs))
        .route("/admin/configs/{name}", get(get_config).post(save_config))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            crate::middlewares::admin_auth_middleware,
        ))
        .layer(DefaultBodyLimit::max(5 * 1024 * 1024));

    public_routes.merge(admin_routes).with_state(state)
}

async fn get_home_profile() -> Json<HomeProfile> {
    Json(load_site_profile())
}

async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
    })
}

async fn atomic_write_text(path: &str, content: &str) -> Result<(), std::io::Error> {
    let tmp = format!(
        "{}.{}.{}.tmp",
        path,
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    );
    tokio::fs::write(&tmp, content).await?;
    match tokio::fs::rename(&tmp, path).await {
        Ok(()) => Ok(()),
        Err(e) => {
            let _ = tokio::fs::remove_file(&tmp).await;
            Err(e)
        }
    }
}

async fn list_configs() -> Response {
    let configs = get_editable_configs()
        .await
        .into_iter()
        .map(|name| ConfigFile { name })
        .collect::<Vec<_>>();
    let mut res = Json(configs).into_response();
    res.headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    res
}

async fn get_config(Path(name): Path<String>) -> Result<Response, StatusCode> {
    let editable = get_editable_configs().await;
    if !editable.contains(&name) {
        return Err(StatusCode::FORBIDDEN);
    }
    let file_path = if name == "sitemap.xml" {
        if tokio::fs::metadata("static/sitemap.xml").await.is_err() {
            let default_sitemap = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n    <url>\n        <loc>https://example.com/</loc>\n        <changefreq>daily</changefreq>\n        <priority>1.0</priority>\n    </url>\n</urlset>";
            let _ = atomic_write_text("static/sitemap.xml", default_sitemap).await;
        }
        "static/sitemap.xml"
    } else {
        name.as_str()
    };
    let content = tokio::fs::read_to_string(file_path)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    let mut res = content.into_response();
    res.headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    Ok(res)
}

async fn save_config(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Json(payload): Json<SaveConfigRequest>,
) -> Result<Response, StatusCode> {
    let editable = get_editable_configs().await;
    if !editable.contains(&name) {
        return Err(StatusCode::FORBIDDEN);
    }

    let file_path = if name == "sitemap.xml" {
        if !payload.content.trim().starts_with("<?xml")
            && !payload.content.trim().starts_with("<urlset")
            && !payload.content.trim().starts_with("<sitemapindex")
        {
            error!("Invalid syntax for sitemap.xml: must be valid XML");
            return Err(StatusCode::BAD_REQUEST);
        }
        "static/sitemap.xml"
    } else {
        if let Err(e) = toml::from_str::<toml::Value>(&payload.content) {
            error!("Invalid TOML syntax for {}: {}", name, e);
            return Err(StatusCode::BAD_REQUEST);
        }

        let schema_ok = match name.as_str() {
            "site.toml" => {
                #[derive(serde::Deserialize)]
                struct SiteWrapper {
                    #[serde(default)]
                    profile: crate::model::HomeProfile,
                    #[serde(default)]
                    site: crate::model::SiteConfig,
                    #[serde(default)]
                    note: crate::model::NoteConfig,
                }
                toml::from_str::<SiteWrapper>(&payload.content)
                    .map(|w| {
                        let _ = w.profile;
                        let _ = w.site;
                        let _ = w.note;
                    })
                    .is_ok()
            }
            "projects.toml" => {
                toml::from_str::<crate::model::ProjectList>(&payload.content).is_ok()
            }
            "about.toml" => toml::from_str::<crate::model::AboutList>(&payload.content).is_ok(),
            "changelog.toml" => {
                toml::from_str::<crate::model::ChangelogList>(&payload.content).is_ok()
            }
            _ => true,
        };

        if !schema_ok {
            error!("Invalid schema for {}", name);
            return Err(StatusCode::BAD_REQUEST);
        }
        name.as_str()
    };

    atomic_write_text(file_path, &payload.content)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    info!(
        "Updated config file on disk asynchronously: {} (path: {})",
        name, file_path
    );

    let rendered = tokio::task::spawn_blocking(crate::render::render_index)
        .await
        .unwrap_or_else(|e| {
            error!("render_index panicked in spawn_blocking (save_config): {e}");
            String::new()
        });

    let markdown_enabled = state.markdown_config.enable;
    let rendered_md = if markdown_enabled {
        tokio::task::spawn_blocking(crate::render::render_index_markdown)
            .await
            .unwrap_or_else(|e| {
                error!("render_index_markdown panicked in spawn_blocking (save_config): {e}");
                String::new()
            })
    } else {
        String::new()
    };

    {
        let mut cache = state.html_cache.write().await;
        *cache = crate::state::HtmlCache::new(
            std::time::SystemTime::now(),
            rendered.into(),
            rendered_md.into(),
            state.assets_config.html_cache_seconds,
        );
    }

    *state.note_list_html_cache.write().await = None;
    state.note_html_cache.write().await.clear();

    info!(
        "In-memory HTML cache and started_at refreshed successfully after saving {}",
        name
    );

    let mut res = StatusCode::OK.into_response();
    res.headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    Ok(res)
}
