use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use axum::{
    Json,
    extract::{Request, State},
    http::{HeaderName, HeaderValue, Method, StatusCode, header},
    middleware,
    middleware::Next,
    response::{IntoResponse, Response},
};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info, warn};

use crate::{
    model::{AdminLoginRequest, AdminLoginResponse, JwtClaims, SecurityConfig},
    state::AppState,
};

pub fn build_cors_layer(security_config: &SecurityConfig) -> CorsLayer {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);

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

/// POST /api/v1/admin/login — 验证凭据，签发 JWT
pub async fn handle_admin_login(
    State(state): State<Arc<AppState>>,
    req: Request,
) -> Response {
    let headers = req.headers().clone();
    let req_uri_authority = req.uri().authority().map(|a| a.as_str().to_string());

    let client_ip = headers.get("cf-connecting-ip")
        .or_else(|| headers.get("x-real-ip"))
        .or_else(|| headers.get("x-forwarded-for"))
        .and_then(|v| v.to_str().ok())
        .unwrap_or("Unknown IP");

    // Rate limiting — 只有登录请求才消耗限额，带 token 的正常请求不计入
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
            warn!("Admin login rate limit exceeded for IP: {}", client_ip);
            return (
                StatusCode::TOO_MANY_REQUESTS,
                [(header::RETRY_AFTER, "60")],
                "Too many requests",
            ).into_response();
        }
        window.push(now);
    }

    // 解析请求体
    let body_bytes = match axum::body::to_bytes(req.into_body(), 64 * 1024).await {
        Ok(b) => b,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid request body").into_response(),
    };
    let payload: AdminLoginRequest = match serde_json::from_slice(&body_bytes) {
        Ok(p) => p,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid JSON").into_response(),
    };

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
    let expiry_secs = security_config.jwt_expiry_secs.unwrap_or(28800);

    // 1. 验证密码
    let password_ok = match &actual_password {
        None => {
            error!("[Security] Admin login attempt rejected: Admin password not configured on server.");
            false
        }
        Some(a) if a.is_empty() || a == "CHANGE_YOUR_PASSWORD" => {
            error!("[Security] Admin login attempt rejected: The default placeholder password ('CHANGE_YOUR_PASSWORD') is in use. Please change your admin_password in config.toml!");
            false
        }
        Some(a) => payload.password == *a,
    };

    if !password_ok {
        warn!("Admin login failed: wrong password. IP: {}", client_ip);
        return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
    }

    // 2. 验证安全问题（可选）
    if auth_ext_secq {
        let answers = match actual_answers {
            Some(ref a) => a,
            None => {
                error!("[Security] auth_ext_secq enabled but no answers configured");
                return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
            }
        };
        match (payload.question_index, &payload.answer) {
            (Some(idx), Some(ans)) if idx < answers.len() => {
                if &answers[idx] != ans {
                    warn!("Admin login failed: wrong security answer (idx={}). IP: {}", idx, client_ip);
                    return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
                }
            }
            _ => {
                warn!("Admin login failed: security question index or answer missing. IP: {}", client_ip);
                return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
            }
        }
    }

    // 3. 验证 CF Trace（可选）
    if auth_ext_cftrace {
        let cf_trace = payload.cf_trace.as_deref().unwrap_or("");
        let mut loc = None;
        let mut warp = None;
        let mut gateway = None;
        let mut trace_ip = None;
        let mut trace_host = None;

        for line in cf_trace.lines() {
            if let Some((key, value)) = line.split_once('=') {
                match key.trim() {
                    "h"       => trace_host = Some(value.trim().to_string()),
                    "loc"     => loc         = Some(value.trim().to_string()),
                    "warp"    => warp        = Some(value.trim().to_string()),
                    "gateway" => gateway     = Some(value.trim().to_string()),
                    "ip"      => trace_ip    = Some(value.trim().to_string()),
                    _ => {}
                }
            }
        }

        let loc_ok = match &loc {
            Some(l) => {
                if !allowed_locs.contains(l) {
                    warn!("Admin login failed: CF Trace loc '{}' not allowed. IP: {}", l, client_ip);
                    return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
                }
                true
            }
            None => {
                warn!("Admin login failed: CF Trace loc missing. IP: {}", client_ip);
                return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
            }
        };

        if loc_ok && warp.as_deref() != Some("on") {
            warn!("Admin login failed: CF Trace WARP is off. IP: {}", client_ip);
            return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
        }
        if loc_ok && gateway.as_deref() != Some("on") {
            warn!("Admin login failed: CF Trace Gateway is off. IP: {}", client_ip);
            return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
        }

        // 校验 trace IP 与客户端 IP 一致性
        if let Some(ref tip) = trace_ip {
            if tip != client_ip {
                warn!("[Security] Trace IP '{}' does not match CF-Connecting-IP '{}'. IP: {}", tip, client_ip, client_ip);
                return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
            }
        }

        // 校验 trace host 与请求 host 一致性
        // 在 HTTP/2 下，客户端可能只发送 :authority 伪头而不发送 Host 头。
        // hyper 会将 :authority 放入 uri 的 authority 部分。
        let request_host = headers
            .get("x-forwarded-host")
            .or_else(|| headers.get("host"))
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
            .or_else(|| req_uri_authority.clone())
            .unwrap_or_else(|| "".to_string());

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

        if let Some(ref th) = trace_host {
            if clean_host(th) != clean_host(&request_host) {
                warn!("[Security] Trace host '{}' does not match request host '{}'. IP: {}", th, request_host, client_ip);
                return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
            }
        }
    }

    // 所有验证通过，签发 JWT
    let now_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let expires_at = now_secs + expiry_secs;

    let claims = JwtClaims {
        sub: "admin".to_string(),
        exp: expires_at,
    };

    let token = match encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(&state.jwt_secret),
    ) {
        Ok(t) => t,
        Err(e) => {
            error!("JWT encode failed: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Internal error").into_response();
        }
    };

    info!(
        "Admin login successful, JWT issued. IP: {}, expires_at: {}",
        client_ip, expires_at
    );

    Json(AdminLoginResponse { token, expires_at }).into_response()
}

/// admin_auth_middleware — 只验 Bearer JWT，不再接受密码 header
pub async fn admin_auth_middleware(
    State(state): State<Arc<AppState>>,
    req: Request,
    next: middleware::Next,
) -> Response {
    let token = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    let token = match token {
        Some(t) => t.to_string(),
        None => {
            return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
        }
    };

    let mut validation = Validation::new(Algorithm::HS256);
    validation.sub = Some("admin".to_string());

    match decode::<JwtClaims>(
        &token,
        &DecodingKey::from_secret(&state.jwt_secret),
        &validation,
    ) {
        Ok(_) => {
            // Token 有效，放行
            next.run(req).await
        }
        Err(e) => {
            warn!("Admin JWT validation failed: {}", e);
            (StatusCode::UNAUTHORIZED, "Unauthorized").into_response()
        }
    }
}
