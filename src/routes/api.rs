use std::sync::Arc;
use axum::{
    extract::{DefaultBodyLimit, Path, State},
    http::StatusCode,
    routing::get,
    Json, Router, middleware,
};
use tracing::{info, error};

use crate::{
    config::{load_site_profile, get_editable_configs},
    model::{ConfigFile, HealthResponse, HomeProfile, SaveConfigRequest},
    state::AppState,
};

pub fn router(state: Arc<AppState>) -> Router {
    let public_routes = Router::new()
        .route("/home/profile", get(get_home_profile))
        .route("/health", get(health_handler));

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

async fn list_configs() -> Json<Vec<ConfigFile>> {
    let configs = get_editable_configs().into_iter()
        .map(|name| ConfigFile { name })
        .collect();
    Json(configs)
}

async fn get_config(Path(name): Path<String>) -> Result<String, StatusCode> {
    let editable = get_editable_configs();
    if !editable.contains(&name) {
        return Err(StatusCode::FORBIDDEN);
    }
    tokio::fs::read_to_string(&name)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)
}

async fn save_config(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Json(payload): Json<SaveConfigRequest>,
) -> Result<StatusCode, StatusCode> {
    let editable = get_editable_configs();
    if !editable.contains(&name) {
        return Err(StatusCode::FORBIDDEN);
    }
    if let Err(e) = toml::from_str::<toml::Value>(&payload.content) {
        error!("Invalid TOML syntax for {}: {}", name, e);
        return Err(StatusCode::BAD_REQUEST);
    }

    let schema_ok = match name.as_str() {
        "site.toml" => {
            #[derive(serde::Deserialize)]
            struct SiteWrapper {
                profile: crate::model::HomeProfile,
                site: crate::model::SiteConfig,
            }
            toml::from_str::<SiteWrapper>(&payload.content)
                .map(|w| {
                    let _ = w.profile;
                    let _ = w.site;
                })
                .is_ok()
        }
        "projects.toml" => toml::from_str::<crate::model::ProjectList>(&payload.content).is_ok(),
        "about.toml" => toml::from_str::<crate::model::AboutList>(&payload.content).is_ok(),
        _ => true,
    };

    if !schema_ok {
        error!("Invalid schema for {}", name);
        return Err(StatusCode::BAD_REQUEST);
    }

    tokio::fs::write(&name, &payload.content).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    info!("Updated config file on disk asynchronously: {}", name);

    let rendered = tokio::task::spawn_blocking(crate::render::render_index)
        .await
        .unwrap_or_else(|e| {
            error!("render_index panicked in spawn_blocking (save_config): {e}");
            String::new()
        });
    {
        let mut cache = state.html_cache.write().await;
        *cache = rendered;
    }

    {
        let mut started_at = state.started_at.write().await;
        *started_at = std::time::SystemTime::now();
    }
    info!("In-memory HTML cache and started_at refreshed successfully after saving {}", name);

    Ok(StatusCode::OK)
}
