use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::{
    extract::{Request, State},
    http::{HeaderName, HeaderValue, Method, StatusCode, header},
    middleware,
    middleware::Next,
    response::{IntoResponse, Response},
};
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info, warn};

use crate::{model::SecurityConfig, state::AppState};

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
        std::sync::Arc::new(tokio::task::spawn_blocking(crate::config::load_security_config).await.unwrap_or_else(|e| {
            tracing::error!("load_security_config panicked in spawn_blocking: {}", e);
            crate::model::SecurityConfig::default()
        }))
    } else {
        state.security_config.clone()
    };

    let headers_map = res.headers_mut();

    // 静态 headers — 零开销
    headers_map.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers_map.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    headers_map.insert(
        header::X_FRAME_OPTIONS,
        HeaderValue::from_static("DENY"),
    );
    headers_map.insert(
        header::STRICT_TRANSPORT_SECURITY,
        HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    );

    // 动态 headers — 仅这 2 个需要运行时构建
    if let Ok(v) = HeaderValue::try_from(config.csp_policy.as_str()) {
        headers_map.insert(header::CONTENT_SECURITY_POLICY, v);
    }
    if let Ok(v) = HeaderValue::try_from(config.permissions_policy.as_str()) {
        headers_map.insert(
            HeaderName::from_static("permissions-policy"),
            v,
        );
    }

    res
}

pub async fn admin_auth_middleware(
    State(state): State<Arc<AppState>>,
    req: Request,
    next: middleware::Next,
) -> Response {
    let headers = req.headers();
    let provided_password = headers
        .get("X-Admin-Password")
        .and_then(|v| v.to_str().ok())
        .map(percent_decode);
    let provided_answer = headers
        .get("X-Admin-Answer")
        .and_then(|v| v.to_str().ok())
        .map(percent_decode);
    let provided_index = headers
        .get("X-Admin-Question-Index")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<usize>().ok());
    let cf_trace_raw = headers.get("X-Admin-Trace").and_then(|v| v.to_str().ok()).unwrap_or("");
    let cf_trace = percent_decode(cf_trace_raw);

    let client_ip = headers.get("cf-connecting-ip")
        .or_else(|| headers.get("x-real-ip"))
        .or_else(|| headers.get("x-forwarded-for"))
        .and_then(|v| v.to_str().ok())
        .unwrap_or("Unknown IP");
    let user_agent = headers.get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("Unknown User-Agent");

    // Rate limiting: 5 attempts per 60 seconds per IP
    {
        let now = Instant::now();
        let mut limiter = state.auth_rate_limiter.lock().await;
        limiter.retain(|_, w| {
            w.retain(|t| now.duration_since(*t) < Duration::from_secs(60));
            !w.is_empty()
        });
        let window = limiter.entry(client_ip.to_string()).or_default();
        if window.len() >= 5 {
            drop(limiter);
            warn!(
                "Admin rate limit exceeded for IP: {}, Path: {}",
                client_ip,
                req.uri().path()
            );
            return (
                StatusCode::TOO_MANY_REQUESTS,
                [(header::RETRY_AFTER, "60")],
                "Too many requests",
            )
                .into_response();
        }
        window.push(now);
    }

    let security_config = if cfg!(debug_assertions) {
        std::sync::Arc::new(tokio::task::spawn_blocking(crate::config::load_security_config).await.unwrap_or_else(|e| {
            tracing::error!("load_security_config panicked in spawn_blocking: {}", e);
            crate::model::SecurityConfig::default()
        }))
    } else {
        state.security_config.clone()
    };

    let actual_password = security_config.admin_password.clone();
    let actual_answers = security_config.admin_security_answers.clone();
    let auth_ext_secq = security_config.auth_ext_secq.unwrap_or(false);
    let auth_ext_cftrace = security_config.auth_ext_cftrace.unwrap_or(false);
    let allowed_locs = security_config.allowed_locs.clone().unwrap_or_else(|| vec!["CN".to_string()]);

    let mut failure_reason = None;

    let mut loc = None;
    let mut warp = None;
    let mut gateway = None;
    let mut trace_ip = None;
    let mut trace_uag = None;
    let mut trace_host = None;

    for line in cf_trace.lines() {
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();
            match key {
                "h" => trace_host = Some(value.to_string()),
                "loc" => loc = Some(value.to_string()),
                "warp" => warp = Some(value.to_string()),
                "gateway" => gateway = Some(value.to_string()),
                "ip" => trace_ip = Some(value.to_string()),
                "uag" => trace_uag = Some(value.to_string()),
                _ => {}
            }
        }
    }

    let is_authenticated = match &actual_password {
        None => {
            error!("[Security] Admin login attempt rejected: Admin password not configured on server.");
            failure_reason = Some("Admin password not configured on server".to_string());
            false
        }
        Some(a) if a.is_empty() || a == "CHANGE_YOUR_PASSWORD" => {
            error!("[Security] Admin login attempt rejected: The default placeholder password ('CHANGE_YOUR_PASSWORD') is in use. Please change your admin_password in config.toml!");
            failure_reason = Some("Admin password is uninitialized or uses default placeholder, login disallowed".to_string());
            false
        }
        Some(a) => {
            match &provided_password {
                Some(p) if p == a => {
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

                    let trace_warp_ok = if secq_ok && auth_ext_cftrace {
                        let loc_ok = match loc {
                            Some(ref l) => {
                                let ok = allowed_locs.contains(l);
                                if !ok {
                                    warn!("CF Trace: location '{}' not allowed (allowed: {:?})", l, allowed_locs);
                                    failure_reason = Some(format!("CF Trace: location '{}' not allowed (allowed: {:?})", l, allowed_locs));
                                }
                                ok
                            }
                            None => {
                                warn!("CF Trace: location 'loc' missing in trace");
                                failure_reason = Some("CF Trace: location 'loc' missing in trace".to_string());
                                false
                            }
                        };

                        let warp_on = if warp.as_deref() == Some("on") {
                            true
                        } else {
                            if loc_ok {
                                warn!("CF Trace: WARP is off (warp={:?})", warp);
                                failure_reason = Some(format!("CF Trace: WARP is off (warp={:?})", warp));
                            }
                            false
                        };

                        let gateway_on = if gateway.as_deref() == Some("on") {
                            true
                        } else {
                            if loc_ok && warp_on {
                                warn!("CF Trace: Gateway is off (gateway={:?})", gateway);
                                failure_reason = Some(format!("CF Trace: Gateway is off (gateway={:?})", gateway));
                            }
                            false
                        };

                        let request_host = headers
                        .get("host")
                        .or_else(|| headers.get("x-forwarded-host"))
                        .and_then(|v| v.to_str().ok())
                        .or_else(|| req.uri().host())
                        .unwrap_or("");

                        let clean_host = |h: &str| -> String {
                            let h = h.trim();
                            let without_port = if h.starts_with('[') {
                                if let Some(end_idx) = h.find(']') {
                                    &h[..=end_idx]
                                } else {
                                    h
                                }
                            } else {
                                h.split(':').next().unwrap_or(h)
                            };
                            without_port.strip_prefix("www.").unwrap_or(without_port).to_string()
                        };

                        let trace_host_ok = match trace_host {
                            Some(ref th) => {
                                if clean_host(th) != clean_host(request_host) {
                                    warn!("[Security] Trace host '{}' does not match request host '{}'", th, request_host);
                                    failure_reason = Some(format!("CF Trace: host '{}' does not match request host '{}'", th, request_host));
                                    false
                                } else {
                                    true
                                }
                            }
                            None => {
                                warn!("CF Trace: host 'h' missing in trace");
                                failure_reason = Some("CF Trace: host 'h' missing in trace".to_string());
                                false
                            }
                        };

                        let trace_ip_ok = match trace_ip {
                            Some(ref tip) => {
                                if tip != client_ip {
                                    warn!("[Security] Trace IP '{}' does not match CF-Connecting-IP '{}'", tip, client_ip);
                                    failure_reason = Some(format!("CF Trace: IP '{}' does not match client IP '{}'", tip, client_ip));
                                    false
                                } else {
                                    true
                                }
                            }
                            None => {
                                warn!("CF Trace: IP 'ip' missing in trace");
                                failure_reason = Some("CF Trace: IP 'ip' missing in trace".to_string());
                                false
                            }
                        };

                        loc_ok && warp_on && gateway_on && trace_host_ok && trace_ip_ok
                    } else {
                        true
                    };

                    secq_ok && trace_warp_ok
                }
                Some(_) => {
                    failure_reason = Some("Password incorrect".to_string());
                    false
                }
                None => {
                    failure_reason = Some("No password provided".to_string());
                    false
                }
            }
        }
    };

    let final_ip = trace_ip.as_deref().unwrap_or(client_ip);
    let final_loc = loc.as_deref().unwrap_or("Unknown Location");
    let final_uag = trace_uag.as_deref().unwrap_or(user_agent);

    if is_authenticated {
        info!(
            "Admin authentication successful! IP: {}, Geolocation: {}, User-Agent: {}, Path: {}",
            final_ip, final_loc, final_uag, req.uri().path()
        );
        next.run(req).await
    } else {
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

fn percent_decode(s: &str) -> String {
    let mut bytes = Vec::new();
    let mut chars = s.as_bytes().iter().peekable();
    while let Some(&b) = chars.next() {
        if b == b'%' {
            let mut chars_clone = chars.clone();
            if let (Some(&h1), Some(&h2)) = (chars_clone.next(), chars_clone.next()) {
                let d1 = (h1 as char).to_digit(16);
                let d2 = (h2 as char).to_digit(16);
                if let (Some(v1), Some(v2)) = (d1, d2) {
                    bytes.push((v1 << 4 | v2) as u8);
                    chars.next();
                    chars.next();
                    continue;
                }
            }
        }
        bytes.push(b);
    }
    String::from_utf8_lossy(&bytes).into_owned()
}
