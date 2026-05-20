use std::sync::Arc;
use axum::{
    extract::{Path, State, Request},
    http::{StatusCode, Method},
    response::{Html, IntoResponse, Response},
    routing::get,
    Json, Router, middleware,
};
use serde::{Deserialize, Serialize};
use std::fs;
use tracing::{info, warn, error};

use crate::middlewares::AppState;

#[derive(Serialize)]
pub struct ConfigFile {
    pub name: String,
}

#[derive(Deserialize)]
pub struct SaveConfigRequest {
    pub content: String,
}

#[derive(Serialize)]
pub struct AuthConfigResponse {
    pub auth_ext_secq: bool,
}

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/admin", get(admin_page_handler))
        .route("/api/v1/admin/configs", get(list_configs))
        .route("/api/v1/admin/configs/{name}", get(get_config).post(save_config))
        .route_layer(middleware::from_fn_with_state(state.clone(), admin_auth_middleware))
        .route("/api/v1/admin/auth_config", get(get_auth_config))
        .with_state(state)
}

async fn get_auth_config(State(state): State<Arc<AppState>>) -> Json<AuthConfigResponse> {
    let security_config = if cfg!(debug_assertions) {
        crate::config::load_security_config()
    } else {
        state.security_config.clone()
    };
    Json(AuthConfigResponse {
        auth_ext_secq: security_config.auth_ext_secq.unwrap_or(false),
    })
}

fn percent_decode(s: &str) -> String {
    let mut bytes = Vec::new();
    let mut chars = s.as_bytes().iter().peekable();
    while let Some(&b) = chars.next() {
        if b == b'%' {
            if let (Some(&h1), Some(&h2)) = (chars.next(), chars.next()) {
                let d1 = (h1 as char).to_digit(16);
                let d2 = (h2 as char).to_digit(16);
                if let (Some(v1), Some(v2)) = (d1, d2) {
                    bytes.push((v1 << 4 | v2) as u8);
                    continue;
                }
            }
        }
        bytes.push(b);
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

async fn admin_auth_middleware(
    State(state): State<Arc<AppState>>,
    req: Request,
    next: middleware::Next,
) -> Response {
    let headers = req.headers();
    let provided_password = headers
        .get("X-Admin-Password")
        .and_then(|v| v.to_str().ok())
        .map(|s| percent_decode(s));
    let provided_answer = headers
        .get("X-Admin-Answer")
        .and_then(|v| v.to_str().ok())
        .map(|s| percent_decode(s));
    let provided_index = headers
        .get("X-Admin-Question-Index")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<usize>().ok());
    let cf_trace_raw = headers.get("X-CF-Trace").and_then(|v| v.to_str().ok()).unwrap_or("");
    let cf_trace = percent_decode(cf_trace_raw);

    let client_ip = headers.get("cf-connecting-ip")
        .or_else(|| headers.get("x-real-ip"))
        .or_else(|| headers.get("x-forwarded-for"))
        .and_then(|v| v.to_str().ok())
        .unwrap_or("Unknown IP");
    let user_agent = headers.get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("Unknown User-Agent");

    let security_config = if cfg!(debug_assertions) {
        crate::config::load_security_config()
    } else {
        state.security_config.clone()
    };

    let actual_password = security_config.admin_password.clone();
    let actual_answers = security_config.admin_security_answers.clone();
    let auth_ext_secq = security_config.auth_ext_secq.unwrap_or(false);
    let auth_ext_warp = security_config.auth_ext_warp.unwrap_or(false);
    let allowed_locs = security_config.allowed_locs.clone().unwrap_or_else(|| vec!["CN".to_string()]);

    let mut failure_reason = None;

    let is_authenticated = match (&provided_password, &actual_password) {
        (Some(p), Some(a)) if p == a => {
            // Password matches.

            // 1. Verify security question if enabled
            let secq_ok = if auth_ext_secq {
                if let Some(answers) = actual_answers {
                    match (provided_index, &provided_answer) {
                        (Some(idx), Some(ans)) if idx < answers.len() => {
                            let correct = &answers[idx] == ans;
                            if !correct {
                                failure_reason = Some(format!("Security question answer incorrect (Question index: {})", idx));
                            }
                            correct
                        }
                        _ => {
                            failure_reason = Some("Security question index or answer missing/invalid".to_string());
                            false
                        }
                    }
                } else {
                    failure_reason = Some("Security question answers not configured in backend".to_string());
                    false
                }
            } else {
                true
            };

            // 2. Verify Cloudflare Trace if enabled
            let warp_ok = if secq_ok && auth_ext_warp {
                let mut loc = None;
                let mut warp = None;
                let mut gateway = None;

                for line in cf_trace.lines() {
                    let parts: Vec<&str> = line.splitn(2, '=').collect();
                    if parts.len() == 2 {
                        let key = parts[0].trim();
                        let value = parts[1].trim();
                        match key {
                            "loc" => loc = Some(value.to_string()),
                            "warp" => warp = Some(value.to_string()),
                            "gateway" => gateway = Some(value.to_string()),
                            _ => {}
                        }
                    }
                }

                let loc_ok = match loc {
                    Some(ref l) => {
                        let ok = allowed_locs.contains(l);
                        if !ok {
                            failure_reason = Some(format!("Cloudflare location '{}' not allowed (allowed: {:?})", l, allowed_locs));
                        }
                        ok
                    }
                    None => {
                        failure_reason = Some("Cloudflare location 'loc' missing in trace".to_string());
                        false
                    }
                };

                let warp_on = if warp.as_deref() == Some("on") {
                    true
                } else {
                    if loc_ok {
                        failure_reason = Some(format!("Cloudflare WARP is off (warp={:?})", warp));
                    }
                    false
                };

                let gateway_on = if gateway.as_deref() == Some("on") {
                    true
                } else {
                    if loc_ok && warp_on {
                        failure_reason = Some(format!("Cloudflare Gateway is off (gateway={:?})", gateway));
                    }
                    false
                };

                loc_ok && warp_on && gateway_on
            } else {
                true
            };

            secq_ok && warp_ok
        }
        (Some(_), Some(_)) => {
            failure_reason = Some("Password incorrect".to_string());
            false
        }
        _ => {
            failure_reason = Some("No password provided".to_string());
            false
        }
    };

    let mut trace_ip = None;
    let mut trace_loc = None;
    let mut trace_uag = None;
    for line in cf_trace.lines() {
        let parts: Vec<&str> = line.splitn(2, '=').collect();
        if parts.len() == 2 {
            let key = parts[0].trim();
            let value = parts[1].trim();
            match key {
                "ip" => trace_ip = Some(value.to_string()),
                "loc" => trace_loc = Some(value.to_string()),
                "uag" => trace_uag = Some(value.to_string()),
                _ => {}
            }
        }
    }

    let final_ip = trace_ip.as_deref().unwrap_or(client_ip);
    let final_loc = trace_loc.as_deref().unwrap_or("Unknown Location");
    let final_uag = trace_uag.as_deref().unwrap_or(user_agent);

    if is_authenticated {
        info!(
            "Admin authentication successful! IP: {}, Geolocation: {}, User-Agent: {}, Path: {}",
            final_ip, final_loc, final_uag, req.uri().path()
        );
        next.run(req).await
    } else {
        // Special case: allow initial GET /admin to load the page (which shows the login dialog)
        if req.uri().path() == "/admin" && req.method() == Method::GET && provided_password.is_none()
        {
            return next.run(req).await;
        }

        warn!(
            "Admin authentication failed! Reason: {}, IP: {}, Geolocation: {}, User-Agent: {}, Path: {}",
            failure_reason.unwrap_or_else(|| "Unknown failure".to_string()),
            final_ip,
            final_loc,
            final_uag,
            req.uri().path()
        );
        (StatusCode::UNAUTHORIZED, "Unauthorized").into_response()
    }
}

async fn admin_page_handler() -> impl IntoResponse {
    let html = fs::read_to_string("templates/admin.html").unwrap_or_else(|_| {
        "<!doctype html><html><body><h1>templates/admin.html not found</h1></body></html>".to_string()
    });
    Html(html)
}

async fn get_editable_configs() -> Vec<String> {
    let mut editable = Vec::new();
    if let Ok(entries) = fs::read_dir(".") {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(".toml") && name != "config.toml" && name != "Cargo.toml" {
                    editable.push(name.to_string());
                }
            }
        }
    }
    editable
}

async fn list_configs() -> Json<Vec<ConfigFile>> {
    let configs = get_editable_configs().await.into_iter()
        .map(|name| ConfigFile { name })
        .collect();
    Json(configs)
}

async fn get_config(Path(name): Path<String>) -> Result<String, StatusCode> {
    let editable = get_editable_configs().await;
    if !editable.contains(&name) {
        return Err(StatusCode::FORBIDDEN);
    }
    fs::read_to_string(&name).map_err(|_| StatusCode::NOT_FOUND)
}

async fn save_config(
    Path(name): Path<String>,
    Json(payload): Json<SaveConfigRequest>,
) -> Result<StatusCode, StatusCode> {
    let editable = get_editable_configs().await;
    if !editable.contains(&name) {
        return Err(StatusCode::FORBIDDEN);
    }
    
    // Validate TOML before saving
    if let Err(e) = toml::from_str::<toml::Value>(&payload.content) {
        error!("Invalid TOML for {}: {}", name, e);
        return Err(StatusCode::BAD_REQUEST);
    }

    fs::write(&name, payload.content).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    info!("Updated config file: {}", name);
    Ok(StatusCode::OK)
}
