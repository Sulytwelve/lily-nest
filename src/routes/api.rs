use std::sync::Arc;
use axum::{
    extract::{Path, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router, middleware,
};
use std::time::{Duration, Instant};
use tracing::{info, error};

use crate::{
    config::{load_site_profile, get_editable_configs},
    model::{HealthResponse, HomeProfile, SaveConfigRequest, AuthConfigResponse, ConfigFile},
    state::AppState,
};

pub fn router(state: Arc<AppState>) -> Router {
    let public_routes = Router::new()
        .route("/home/profile", get(get_home_profile))
        .route("/health", get(health_handler));

    let admin_routes = Router::new()
        .route("/admin/configs", get(list_configs))
        .route("/admin/configs/{name}", get(get_config).post(save_config))
        .route_layer(middleware::from_fn_with_state(state.clone(), crate::middlewares::admin_auth_middleware));

    let admin_public_routes = Router::new()
        .route("/admin/auth_config", get(get_auth_config));

    public_routes
        .merge(admin_routes)
        .merge(admin_public_routes)
        .with_state(state)
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
    std::fs::read_to_string(&name).map_err(|_| StatusCode::NOT_FOUND)
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

    let rendered = crate::render::render_index();
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

async fn get_auth_config(
    State(state): State<Arc<AppState>>,
    req: axum::extract::Request,
) -> Response {
    let headers = req.headers();
    let client_ip = headers
        .get("cf-connecting-ip")
        .or_else(|| headers.get("x-real-ip"))
        .or_else(|| headers.get("x-forwarded-for"))
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    let now = Instant::now();
    let mut limiter = state.auth_rate_limiter.lock().await;
    
    limiter.retain(|_, w| {
        w.retain(|t| now.duration_since(*t) < Duration::from_secs(60));
        !w.is_empty()
    });

    let window = limiter.entry(client_ip).or_default();
    if window.len() >= 10 {
        drop(limiter);
        return (
            StatusCode::TOO_MANY_REQUESTS,
            [(header::RETRY_AFTER, "60")],
            "Too many requests",
        )
            .into_response();
    }
    window.push(now);
    drop(limiter);

    let security_config = if cfg!(debug_assertions) {
        crate::config::load_security_config()
    } else {
        state.security_config.clone()
    };
    Json(AuthConfigResponse {
        auth_ext_secq: security_config.auth_ext_secq.unwrap_or(false),
        auth_ext_cftrace: security_config.auth_ext_cftrace.unwrap_or(false),
        cftrace_url: security_config.cftrace_url.clone(),
        security_questions: security_config.admin_security_questions.clone(),
    })
    .into_response()
}
