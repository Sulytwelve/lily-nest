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
    model::{AdminLoginRequest, AdminLoginResponse, AuthClaims, SecurityConfig},
    state::AppState,
};

const PLACEHOLDER_ANSWERS: &[&str] = &["default1", "default2", "default3"];

pub fn build_cors_layer(security_config: &SecurityConfig) -> CorsLayer {
    let cors = CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
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

/// 零依赖恒定时间字符串比较（仅用于密码/密保答案这类短字符串）。
fn constant_time_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.bytes()
        .zip(b.bytes())
        .fold(0u8, |acc, (x, y)| acc | (x ^ y))
        == 0
}

pub async fn security_headers(
    State(state): State<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Response {
    let mut res = next.run(req).await;

    let config = if cfg!(debug_assertions) {
        std::sync::Arc::new(
            tokio::task::spawn_blocking(crate::config::load_security_config)
                .await
                .unwrap_or_else(|e| {
                    tracing::error!("load_security_config panicked in spawn_blocking: {}", e);
                    crate::model::SecurityConfig::default()
                }),
        )
    } else {
        state.security_config.clone()
    };

    let mut csp_policy = if config.csp_policy.trim().is_empty() {
        error!("[security] CSP policy is empty; falling back to the default policy");
        crate::model::SecurityConfig::default().csp_policy
    } else {
        config.csp_policy.clone()
    };

    if let Some(token) = state.cloudflare_config.web_analytics_token.as_deref() {
        let token = token.trim();
        if !token.is_empty()
            && token
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            if csp_policy.contains("script-src 'self'")
                && !csp_policy.contains("https://static.cloudflareinsights.com")
            {
                csp_policy = csp_policy.replace(
                    "script-src 'self'",
                    "script-src 'self' https://static.cloudflareinsights.com",
                );
            }
            if csp_policy.contains("connect-src 'self'")
                && !csp_policy.contains("https://cloudflareinsights.com")
            {
                csp_policy = csp_policy.replace(
                    "connect-src 'self'",
                    "connect-src 'self' https://cloudflareinsights.com",
                );
            }
        }
    }

    let permissions_policy = if config.permissions_policy.trim().is_empty() {
        error!("[security] Permissions-Policy is empty; falling back to the default policy");
        crate::model::SecurityConfig::default()
            .permissions_policy
            .clone()
    } else {
        config.permissions_policy.clone()
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
    headers_map.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
    headers_map.insert(
        header::STRICT_TRANSPORT_SECURITY,
        HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    );

    // 动态 headers — 仅这 2 个需要运行时构建
    match HeaderValue::try_from(csp_policy.as_str()) {
        Ok(v) => {
            headers_map.insert(header::CONTENT_SECURITY_POLICY, v);
        }
        Err(e) => {
            error!("[security] invalid CSP policy ({e}); falling back to the default policy");
            let default_cfg = crate::model::SecurityConfig::default();
            if let Ok(v) = HeaderValue::try_from(default_cfg.csp_policy.as_str()) {
                headers_map.insert(header::CONTENT_SECURITY_POLICY, v);
            }
        }
    }
    match HeaderValue::try_from(permissions_policy.as_str()) {
        Ok(v) => {
            headers_map.insert(HeaderName::from_static("permissions-policy"), v);
        }
        Err(e) => {
            error!(
                "[security] invalid Permissions-Policy ({e}); falling back to the default policy"
            );
            let default_cfg = crate::model::SecurityConfig::default();
            if let Ok(v) = HeaderValue::try_from(default_cfg.permissions_policy.as_str()) {
                headers_map.insert(HeaderName::from_static("permissions-policy"), v);
            }
        }
    }

    res
}

/// POST /api/v1/admin/login — 验证凭据，签发 JWT
pub async fn handle_admin_login(State(state): State<Arc<AppState>>, req: Request) -> Response {
    let headers = req.headers().clone();
    let req_uri_authority = req.uri().authority().map(|a| a.as_str().to_string());

    let client_ip = headers
        .get("cf-connecting-ip")
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
            )
                .into_response();
        }
        window.push(now);
    }

    // 解析请求体（带 15s 超时，避免慢速 body 长期占连接）
    let body_bytes = match tokio::time::timeout(
        Duration::from_secs(15),
        axum::body::to_bytes(req.into_body(), 64 * 1024),
    )
    .await
    {
        Ok(Ok(b)) => b,
        Ok(Err(_)) => return (StatusCode::BAD_REQUEST, "Invalid request body").into_response(),
        Err(_) => return (StatusCode::REQUEST_TIMEOUT, "Request timeout").into_response(),
    };
    let payload: AdminLoginRequest = match serde_json::from_slice(&body_bytes) {
        Ok(p) => p,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid JSON").into_response(),
    };

    let security_config = if cfg!(debug_assertions) {
        std::sync::Arc::new(
            tokio::task::spawn_blocking(crate::config::load_security_config)
                .await
                .unwrap_or_else(|e| {
                    tracing::error!("load_security_config panicked in spawn_blocking: {}", e);
                    crate::model::SecurityConfig::default()
                }),
        )
    } else {
        state.security_config.clone()
    };

    let actual_password = security_config.admin_password.clone();
    let actual_answers = security_config.admin_security_answers.clone();
    let auth_ext_secq = security_config.auth_ext_secq.unwrap_or(false);
    let auth_ext_cftrace = security_config.auth_ext_cftrace.unwrap_or(false);
    let allowed_locs = security_config
        .allowed_locs
        .clone()
        .unwrap_or_else(|| vec!["CN".to_string()]);
    let expiry_secs = security_config.jwt_expiry_secs.unwrap_or(28800);

    // 1. 验证密码
    let password_ok = match &actual_password {
        None => {
            error!(
                "[Security] Admin login attempt rejected: Admin password not configured on server."
            );
            false
        }
        Some(a) if a.is_empty() || a == "CHANGE_YOUR_PASSWORD" => {
            error!(
                "[Security] Admin login attempt rejected: The default placeholder password ('CHANGE_YOUR_PASSWORD') is in use. Please change your admin_password in config.toml!"
            );
            false
        }
        Some(a) => constant_time_eq(&payload.password, a),
    };

    if !password_ok {
        warn!("Admin login failed: wrong password. IP: {}", client_ip);
        return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
    }

    // 2. 验证安全问题（可选）
    if auth_ext_secq {
        let answers = match actual_answers.as_ref() {
            Some(a) => a,
            None => {
                error!("[Security] auth_ext_secq enabled but no answers configured");
                return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
            }
        };
        let questions = match security_config.admin_security_questions.as_ref() {
            Some(q) => q,
            None => {
                error!("[Security] auth_ext_secq enabled but no security questions configured");
                return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
            }
        };
        if answers.len() != questions.len() {
            error!(
                "[Security] auth_ext_secq enabled but answers ({}) and questions ({}) count mismatch",
                answers.len(),
                questions.len()
            );
            return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
        }
        if answers
            .iter()
            .any(|a| PLACEHOLDER_ANSWERS.contains(&a.as_str()))
            || questions
                .iter()
                .any(|q| PLACEHOLDER_ANSWERS.contains(&q.as_str()))
        {
            error!(
                "[Security] auth_ext_secq enabled but placeholder/default security answers/questions are in use"
            );
            return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
        }
        match (payload.question_index, &payload.answer) {
            (Some(idx), Some(ans)) if idx < answers.len() => {
                if !constant_time_eq(&answers[idx], ans) {
                    warn!(
                        "Admin login failed: wrong security answer (idx={}). IP: {}",
                        idx, client_ip
                    );
                    return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
                }
            }
            _ => {
                warn!(
                    "Admin login failed: security question index or answer missing. IP: {}",
                    client_ip
                );
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
                    "h" => trace_host = Some(value.trim().to_string()),
                    "loc" => loc = Some(value.trim().to_string()),
                    "warp" => warp = Some(value.trim().to_string()),
                    "gateway" => gateway = Some(value.trim().to_string()),
                    "ip" => trace_ip = Some(value.trim().to_string()),
                    _ => {}
                }
            }
        }

        let loc_ok = match &loc {
            Some(l) => {
                if !allowed_locs.contains(l) {
                    warn!(
                        "Admin login failed: CF Trace loc '{}' not allowed. IP: {}",
                        l, client_ip
                    );
                    return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
                }
                true
            }
            None => {
                warn!(
                    "Admin login failed: CF Trace loc missing. IP: {}",
                    client_ip
                );
                return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
            }
        };

        if loc_ok && warp.as_deref() != Some("on") {
            warn!(
                "Admin login failed: CF Trace WARP is off. IP: {}",
                client_ip
            );
            return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
        }
        if loc_ok && gateway.as_deref() != Some("on") {
            warn!(
                "Admin login failed: CF Trace Gateway is off. IP: {}",
                client_ip
            );
            return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
        }

        // ip 与 h 为必填，缺失直接拒绝（fail-closed）
        let trace_ip = match trace_ip {
            Some(ip) => ip,
            None => {
                warn!("Admin login failed: CF Trace ip missing. IP: {}", client_ip);
                return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
            }
        };
        let trace_host = match trace_host {
            Some(h) => h,
            None => {
                warn!(
                    "Admin login failed: CF Trace host missing. IP: {}",
                    client_ip
                );
                return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
            }
        };

        // 校验 trace IP 与客户端 IP 一致性
        if trace_ip.as_str() != client_ip {
            warn!(
                "[Security] Trace IP '{}' does not match CF-Connecting-IP '{}'. IP: {}",
                trace_ip, client_ip, client_ip
            );
            return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
        }

        // 校验 trace host 与请求 host 一致性
        // 在 HTTP/2 下，客户端可能只发送 :authority 伪头而不发送 Host 头。
        // hyper 会将 :authority 放入 uri 的 authority 部分。
        let request_host = headers
            .get("host")
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
            without_port
                .strip_prefix("www.")
                .unwrap_or(without_port)
                .to_string()
        };

        if clean_host(&trace_host) != clean_host(&request_host) {
            warn!(
                "[Security] Trace host '{}' does not match request host '{}'. IP: {}",
                trace_host, request_host, client_ip
            );
            return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
        }
    }

    // 所有验证通过，签发 JWT
    let now_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let expires_at = now_secs.saturating_add(expiry_secs);

    let claims = AuthClaims {
        sub: "admin".to_string(),
        name: "管理员".to_string(),
        role: "admin".to_string(),
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

    let mut res = Json(AdminLoginResponse {
        token,
        expires_at,
        role: "admin".to_string(),
        name: "管理员".to_string(),
    })
    .into_response();
    res.headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    res.headers_mut()
        .insert(header::PRAGMA, HeaderValue::from_static("no-cache"));
    res.headers_mut()
        .insert(header::EXPIRES, HeaderValue::from_static("0"));
    res
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
            return (
                StatusCode::UNAUTHORIZED,
                [(header::WWW_AUTHENTICATE, "Bearer")],
                "Unauthorized",
            )
                .into_response();
        }
    };

    let validation = Validation::new(Algorithm::HS256);

    match decode::<AuthClaims>(
        &token,
        &DecodingKey::from_secret(&state.jwt_secret),
        &validation,
    ) {
        Ok(data) if data.claims.role == "admin" => {
            // Token 有效且角色为 admin，放行
            next.run(req).await
        }
        Ok(_) => {
            warn!("Admin JWT validation failed: role is not admin");
            (
                StatusCode::UNAUTHORIZED,
                [(header::WWW_AUTHENTICATE, "Bearer")],
                "Unauthorized",
            )
                .into_response()
        }
        Err(e) => {
            warn!("Admin JWT validation failed: {}", e);
            (
                StatusCode::UNAUTHORIZED,
                [(header::WWW_AUTHENTICATE, "Bearer")],
                "Unauthorized",
            )
                .into_response()
        }
    }
}

/// note_auth_middleware — 笔记与发文专线鉴权中间件
/// 支持两种认证机制：
/// 1. Admin / Agent 使用常规 HS256 JWT（本地 jwt_secret）
/// 2. Agent 使用 Ed25519 签名的 JWT（本地 .agent.pub 公钥验签），完全免除密码与 cf-trace
pub async fn note_auth_middleware(
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
            return (
                StatusCode::UNAUTHORIZED,
                [(header::WWW_AUTHENTICATE, "Bearer")],
                "Unauthorized",
            )
                .into_response();
        }
    };

    // 1. 优先尝试用本地 HS256 密钥解密（适用于 Web 端登录的 admin 角色，或使用对称密钥的 agent）
    let validation_hs256 = Validation::new(Algorithm::HS256);
    if let Ok(data) = decode::<AuthClaims>(
        &token,
        &DecodingKey::from_secret(&state.jwt_secret),
        &validation_hs256,
    ) {
        if data.claims.role == "admin" || data.claims.role == "agent" {
            return next.run(req).await;
        } else {
            warn!(
                "Note API request rejected: role '{}' is neither admin nor agent",
                data.claims.role
            );
            return (StatusCode::FORBIDDEN, "Forbidden: insufficient role").into_response();
        }
    }

    // 2. 尝试用服务器保存的 Agent 公钥（.agent.pub / LILY_AGENT_PUB_KEY）进行非对称验签
    if let Some(ref pub_key_bytes) = state.agent_pub_key {
        let validation_asym = Validation::new(Algorithm::EdDSA);

        let decoding_key = DecodingKey::from_ed_pem(pub_key_bytes);

        if let Ok(key) = decoding_key {
            if let Ok(data) = decode::<AuthClaims>(&token, &key, &validation_asym) {
                if data.claims.role == "agent" || data.claims.role == "admin" {
                    info!(
                        "Agent public key authentication successful for sub: {}",
                        data.claims.sub
                    );
                    return next.run(req).await;
                } else {
                    warn!(
                        "Agent JWT valid but role '{}' is not allowed",
                        data.claims.role
                    );
                    return (StatusCode::FORBIDDEN, "Forbidden").into_response();
                }
            } else {
                warn!("Agent public key signature verification failed for token");
            }
        } else {
            warn!("Failed to parse agent public key from PEM format (must be Ed25519 PEM)");
        }
    }

    warn!("Note API authentication failed: invalid token or signature");
    (
        StatusCode::UNAUTHORIZED,
        [(header::WWW_AUTHENTICATE, "Bearer")],
        "Unauthorized",
    )
        .into_response()
}
