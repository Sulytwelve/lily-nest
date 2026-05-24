use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::{HeaderName, HeaderValue, Method, StatusCode, header},
    middleware,
    middleware::Next,
    response::{IntoResponse, Response},
};
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info, warn};

use crate::{config::load_security_config, model::SecurityConfig, state::AppState};

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

    let security_config = if cfg!(debug_assertions) {
        crate::config::load_security_config()
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

    for line in cf_trace.lines() {
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();
            match key {
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
            failure_reason = Some("Admin password not configured on server".to_string());
            false
        }
        Some(a) if a.is_empty() || a == "CHANGE_YOUR_PASSWORD" || a == "CHANGE_YOUR_ADMIN_PSWD" => {
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
                                    failure_reason = Some(format!("CF Trace: location '{}' not allowed (allowed: {:?})", l, allowed_locs));
                                }
                                ok
                            }
                            None => {
                                failure_reason = Some("CF Trace: location 'loc' missing in trace".to_string());
                                false
                            }
                        };

                        let warp_on = if warp.as_deref() == Some("on") {
                            true
                        } else {
                            if loc_ok {
                                failure_reason = Some(format!("CF Trace: WARP is off (warp={:?})", warp));
                            }
                            false
                        };

                        let gateway_on = if gateway.as_deref() == Some("on") {
                            true
                        } else {
                            if loc_ok && warp_on {
                                failure_reason = Some(format!("CF Trace: Gateway is off (gateway={:?})", gateway));
                            }
                            // false
                            false
                        };

                        loc_ok && warp_on && gateway_on
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
