use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Path, Request, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    middleware,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::io::AsyncWriteExt;
use tracing::{error, info, warn};

use crate::{
    config::{get_editable_configs, load_site_profile},
    middlewares::handle_admin_login,
    model::{
        AdminLoginQuestion, AuthClaims, AuthSecrets, ConfigFile, HealthResponse, HomeProfile,
        SaveConfigRequest,
    },
    state::AppState,
};

pub fn public_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/home/profile", get(get_home_profile))
        .route("/health", get(health_handler))
        .with_state(state)
}

/// 登录与按需取题端点：公开可达，但有意不挂 CORS 层（app.rs 会单独组装）。
pub fn sensitive_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/admin/login", post(handle_admin_login))
        .route("/admin/login/question", get(get_admin_login_question))
        // 首次 Web 初始化端点：公开可达，但有意不挂 CORS（与登录一致）。
        .route("/admin/setup/status", get(admin_setup_status))
        .route("/admin/setup", post(admin_setup))
        .with_state(state)
}

pub fn admin_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/admin/configs", get(list_configs))
        .route("/admin/configs/{name}", get(get_config).post(save_config))
        .route("/admin/logout", post(logout))
        .route("/admin/password", post(change_admin_password))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            crate::middlewares::admin_auth_middleware,
        ))
        .layer(DefaultBodyLimit::max(5 * 1024 * 1024))
        .with_state(state)
}

/// POST /api/v1/admin/logout — 服务端吊销当前 JWT（B19）。
/// 该路由位于 admin_auth_middleware 之后，中间件已校验 token 有效且未吊销。
async fn logout(State(state): State<Arc<AppState>>, req: Request) -> Response {
    let token = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .unwrap_or("");

    let validation = Validation::new(Algorithm::HS256);
    if let Ok(data) = decode::<AuthClaims>(
        token,
        &DecodingKey::from_secret(&state.jwt_secret),
        &validation,
    ) {
        crate::middlewares::revoke_jwt(&state, &data.claims).await;
    }

    StatusCode::NO_CONTENT.into_response()
}

#[derive(serde::Deserialize)]
struct AdminSetupRequest {
    setup_code: String,
    password: String,
}

#[derive(serde::Serialize)]
struct AdminSetupStatusResponse {
    setup_required: bool,
}

#[derive(serde::Deserialize)]
struct AdminPasswordRequest {
    old_password: String,
    new_password: String,
}

/// GET /api/v1/admin/setup/status — 供前端判断是否进入首次初始化流程。
async fn admin_setup_status(State(state): State<Arc<AppState>>) -> Response {
    let setup_required = state
        .auth_secrets
        .read()
        .await
        .admin_password_hash
        .is_none();
    let mut res = Json(AdminSetupStatusResponse { setup_required }).into_response();
    res.headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    res
}

/// POST /api/v1/admin/setup — 使用一次性 Setup Code 完成首次密码初始化。
async fn admin_setup(
    State(state): State<Arc<AppState>>,
    payload: Result<Json<AdminSetupRequest>, axum::extract::rejection::JsonRejection>,
) -> Response {
    let Json(payload) = match payload {
        Ok(payload) => payload,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid JSON body").into_response(),
    };

    if state
        .auth_secrets
        .read()
        .await
        .admin_password_hash
        .is_some()
    {
        return (StatusCode::CONFLICT, "Admin already initialized").into_response();
    }
    if payload.password.chars().count() < 8 {
        return (
            StatusCode::BAD_REQUEST,
            "Password must be at least 8 characters",
        )
            .into_response();
    }

    let code_ok = {
        let mut setup_code = state.setup_code.lock().await;
        match setup_code.as_ref() {
            Some((stored, created_at)) if created_at.elapsed() <= Duration::from_secs(600) => {
                let ok = crate::middlewares::constant_time_eq(&payload.setup_code, stored);
                if ok {
                    *setup_code = None;
                }
                ok
            }
            Some((_, _)) => {
                *setup_code = None;
                false
            }
            None => false,
        }
    };
    if !code_ok {
        warn!("Admin setup rejected: missing, invalid, or expired setup code");
        return (StatusCode::FORBIDDEN, "Invalid or expired setup code").into_response();
    }

    let answer_hashes = state
        .auth_secrets
        .read()
        .await
        .admin_security_answer_hashes
        .clone();
    let new_secrets = AuthSecrets {
        admin_password_hash: Some(crate::secrets::hash_secret(&payload.password)),
        admin_security_answer_hashes: answer_hashes,
    };
    if let Err(res) = persist_auth_secrets(new_secrets.clone()).await {
        return res;
    }

    *state.auth_secrets.write().await = new_secrets;
    StatusCode::NO_CONTENT.into_response()
}

/// POST /api/v1/admin/password — 登录后在线修改管理员密码，并吊销当前 token。
async fn change_admin_password(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    payload: Result<Json<AdminPasswordRequest>, axum::extract::rejection::JsonRejection>,
) -> Response {
    let Json(payload) = match payload {
        Ok(payload) => payload,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid JSON body").into_response(),
    };

    let auth_secrets = state.auth_secrets.read().await;
    let Some(password_hash) = auth_secrets.admin_password_hash.clone() else {
        return (
            StatusCode::UNAUTHORIZED,
            "Admin password is not configured. Run lily-nest set-password.",
        )
            .into_response();
    };
    let answer_hashes = auth_secrets.admin_security_answer_hashes.clone();
    drop(auth_secrets);

    if !crate::secrets::verify_secret(&payload.old_password, &password_hash) {
        warn!("Admin password change rejected: wrong old password");
        return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
    }
    if payload.new_password.chars().count() < 8 {
        return (
            StatusCode::BAD_REQUEST,
            "New password must be at least 8 characters",
        )
            .into_response();
    }
    if payload.old_password == payload.new_password {
        return (
            StatusCode::BAD_REQUEST,
            "New password must be different from old password",
        )
            .into_response();
    }

    let new_secrets = AuthSecrets {
        admin_password_hash: Some(crate::secrets::hash_secret(&payload.new_password)),
        admin_security_answer_hashes: answer_hashes,
    };
    if let Err(res) = persist_auth_secrets(new_secrets.clone()).await {
        return res;
    }
    *state.auth_secrets.write().await = new_secrets;

    let token = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .unwrap_or("");
    if let Ok(data) = decode::<AuthClaims>(
        token,
        &DecodingKey::from_secret(&state.jwt_secret),
        &Validation::new(Algorithm::HS256),
    ) {
        crate::middlewares::revoke_jwt(&state, &data.claims).await;
    }

    StatusCode::NO_CONTENT.into_response()
}

async fn persist_auth_secrets(secrets: AuthSecrets) -> Result<(), Response> {
    let secrets_for_task = secrets.clone();
    match tokio::task::spawn_blocking(move || crate::secrets::save_auth_secrets(&secrets_for_task))
        .await
    {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => {
            error!("failed to save secrets.toml: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
        Err(e) => {
            error!("secrets.toml save task panicked: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
    }
}

async fn get_home_profile() -> Json<HomeProfile> {
    Json(load_site_profile())
}

async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

/// GET /api/v1/admin/login/question — 登录时按需随机下发一道密保题。
/// 与 B22 配合：`/admin` 页面只携带题目数量，不再公开完整题目集。
async fn get_admin_login_question(
    State(state): State<Arc<AppState>>,
) -> Result<Response, StatusCode> {
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

    if security_config.auth_ext_secq != Some(true) {
        return Err(StatusCode::NOT_FOUND);
    }
    let questions = match security_config.admin_security_questions.as_ref() {
        Some(questions) if !questions.is_empty() => questions,
        _ => return Err(StatusCode::NOT_FOUND),
    };

    const PLACEHOLDER_QUESTIONS: &[&str] = &["default1", "default2", "default3"];
    if questions
        .iter()
        .any(|question| PLACEHOLDER_QUESTIONS.contains(&question.as_str()))
    {
        return Err(StatusCode::NOT_FOUND);
    }

    let question_index = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as usize
        % questions.len();
    let mut res = Json(AdminLoginQuestion {
        question_index,
        question: questions[question_index].clone(),
    })
    .into_response();
    res.headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    Ok(res)
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
    // M3：temp + write + flush + fsync + rename 原子落盘
    let mut file = tokio::fs::File::create(&tmp).await?;
    file.write_all(content.as_bytes()).await?;
    file.flush().await?;
    file.sync_all().await?;
    drop(file);
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
