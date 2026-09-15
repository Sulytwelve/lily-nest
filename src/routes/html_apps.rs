use crate::{
    model::{
        AdminHtmlAppCreateRequest, AdminHtmlAppUpdateRequest, HtmlAppCategory, HtmlAppManifest,
        HtmlAppMeta,
    },
    state::AppState,
};
use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Path, Request, State},
    http::{HeaderName, HeaderValue, StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
};
use chrono::Utc;
use rand::Rng;
use serde_json::json;
use std::{
    path::{Path as FsPath, PathBuf},
    sync::Arc,
};
use tokio::io::AsyncWriteExt;

const APPS_ROOT: &str = "apps";
const MANIFEST_PATH: &str = "apps/manifest.toml";
const MAX_HTML_BYTES: usize = 2 * 1024 * 1024;
// JSON may encode a single input byte as a six-byte `\u00xx` escape. The
// decoded HTML is still independently capped at MAX_HTML_BYTES below.
const MAX_UPLOAD_BODY_BYTES: usize = MAX_HTML_BYTES * 6 + 64 * 1024;
const APP_CSP: &str = "sandbox allow-scripts; default-src 'none'; script-src 'unsafe-inline' blob:; style-src 'unsafe-inline'; img-src data: blob:; media-src data: blob:; connect-src 'none'; form-action 'none'; base-uri 'none'; object-src 'none'; frame-src 'none'; frame-ancestors 'none'";

pub fn public_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/tools/{slug}", get(serve_tool))
        .route("/games/{slug}", get(serve_game))
        .layer(middleware::from_fn(app_security_headers))
        .with_state(state)
}

pub fn admin_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/v1/admin/apps", get(list_apps).post(create_app))
        .route(
            "/api/v1/admin/apps/{category}/{slug}",
            axum::routing::put(update_app).delete(delete_app),
        )
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            crate::middlewares::admin_auth_middleware,
        ))
        .layer(DefaultBodyLimit::max(MAX_UPLOAD_BODY_BYTES))
        .with_state(state)
}

pub async fn load_manifest() -> Vec<HtmlAppMeta> {
    if let Err(error) = tokio::fs::create_dir_all(format!("{APPS_ROOT}/tools")).await {
        tracing::error!("failed to create apps/tools: {error}");
    }
    if let Err(error) = tokio::fs::create_dir_all(format!("{APPS_ROOT}/games")).await {
        tracing::error!("failed to create apps/games: {error}");
    }

    let manifest_path = PathBuf::from(MANIFEST_PATH);
    recover_backup(&manifest_path).await;
    let Ok(contents) = tokio::fs::read_to_string(&manifest_path).await else {
        return Vec::new();
    };
    match toml::from_str::<HtmlAppManifest>(&contents) {
        Ok(manifest) => {
            let mut apps = Vec::with_capacity(manifest.apps.len());
            for mut app in manifest.apps {
                if !is_valid_slug(&app.slug) {
                    continue;
                }
                if apps.iter().any(|existing: &HtmlAppMeta| {
                    existing.category == app.category && existing.slug == app.slug
                }) {
                    tracing::warn!(
                        "ignoring duplicate app manifest entry: {}/{}",
                        app.category.as_str(),
                        app.slug
                    );
                    continue;
                }
                let path = app_path(app.category, &app.slug);
                recover_backup(&path).await;
                if !tokio::fs::metadata(&path)
                    .await
                    .is_ok_and(|metadata| metadata.is_file())
                {
                    tracing::warn!("ignoring app with missing HTML: {}", path.display());
                    continue;
                }
                app.url = app_url(app.category, &app.slug);
                apps.push(app);
            }
            sort_apps(&mut apps);
            apps
        }
        Err(error) => {
            tracing::error!("failed to parse {MANIFEST_PATH}: {error}");
            Vec::new()
        }
    }
}

async fn list_apps(State(state): State<Arc<AppState>>) -> Response {
    api_response(Json(state.html_apps.read().await.clone()).into_response())
}

async fn create_app(
    State(state): State<Arc<AppState>>,
    payload: Result<Json<AdminHtmlAppCreateRequest>, axum::extract::rejection::JsonRejection>,
) -> Response {
    let Json(payload) = match payload {
        Ok(payload) => payload,
        Err(rejection) => return json_rejection(rejection),
    };
    if let Err(message) = validate_text_fields(&payload.title, &payload.description) {
        return api_error(StatusCode::BAD_REQUEST, message);
    }
    if let Err(message) = validate_html(&payload.html) {
        return api_error(StatusCode::BAD_REQUEST, message);
    }
    let slug = match resolve_slug(payload.slug.as_deref(), payload.filename.as_deref()) {
        Ok(slug) => slug,
        Err(message) => return api_error(StatusCode::BAD_REQUEST, message),
    };

    let _mutation = state.html_app_mutations.lock().await;
    let mut apps = state.html_apps.write().await;
    if apps
        .iter()
        .any(|app| app.category == payload.category && app.slug == slug)
    {
        return api_error(
            StatusCode::CONFLICT,
            "an app with this category and slug exists",
        );
    }

    let now = Utc::now().to_rfc3339();
    let meta = HtmlAppMeta {
        category: payload.category,
        slug: slug.clone(),
        title: payload.title.trim().to_string(),
        description: payload.description.trim().to_string(),
        filename: clean_filename(payload.filename),
        created_at: now.clone(),
        updated_at: now,
        url: app_url(payload.category, &slug),
    };
    let path = app_path(meta.category, &meta.slug);
    // A valid file absent from the manifest can be left behind if the process
    // stopped between the two crash-recoverable writes. A new authenticated
    // create for the same slug deliberately adopts/replaces that orphan.
    let previous_orphan = match tokio::fs::read(&path).await {
        Ok(contents) => Some(contents),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => {
            tracing::error!("failed to inspect orphan {}: {error}", path.display());
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, "failed to inspect HTML");
        }
    };
    if let Err(error) = atomic_write(&path, payload.html.as_bytes()).await {
        tracing::error!("failed to write {}: {error}", path.display());
        return api_error(StatusCode::INTERNAL_SERVER_ERROR, "failed to save HTML");
    }

    apps.push(meta.clone());
    sort_apps(&mut apps);
    if let Err(error) = save_manifest(&apps).await {
        apps.retain(|app| !(app.category == meta.category && app.slug == meta.slug));
        if let Some(previous_orphan) = previous_orphan {
            let _ = atomic_write(&path, &previous_orphan).await;
        } else {
            let _ = tokio::fs::remove_file(&path).await;
        }
        tracing::error!("failed to save app manifest: {error}");
        return api_error(StatusCode::INTERNAL_SERVER_ERROR, "failed to save manifest");
    }
    api_response((StatusCode::CREATED, Json(meta)).into_response())
}

async fn update_app(
    State(state): State<Arc<AppState>>,
    Path((category, slug)): Path<(String, String)>,
    payload: Result<Json<AdminHtmlAppUpdateRequest>, axum::extract::rejection::JsonRejection>,
) -> Response {
    let category = match parse_category(&category) {
        Some(category) if is_valid_slug(&slug) => category,
        _ => return api_error(StatusCode::NOT_FOUND, "app not found"),
    };
    let Json(payload) = match payload {
        Ok(payload) => payload,
        Err(rejection) => return json_rejection(rejection),
    };
    if payload.category.is_some_and(|value| value != category)
        || payload.slug.as_deref().is_some_and(|value| value != slug)
    {
        return api_error(StatusCode::BAD_REQUEST, "category and slug are immutable");
    }
    if let Some(html) = payload.html.as_deref()
        && let Err(message) = validate_html(html)
    {
        return api_error(StatusCode::BAD_REQUEST, message);
    }

    let _mutation = state.html_app_mutations.lock().await;
    let mut apps = state.html_apps.write().await;
    let Some(index) = apps
        .iter()
        .position(|app| app.category == category && app.slug == slug)
    else {
        return api_error(StatusCode::NOT_FOUND, "app not found");
    };
    let mut updated = apps[index].clone();
    if let Some(title) = payload.title.as_deref() {
        if let Err(message) = validate_text_fields(title, &updated.description) {
            return api_error(StatusCode::BAD_REQUEST, message);
        }
        updated.title = title.trim().to_string();
    }
    if let Some(description) = payload.description.as_deref() {
        if let Err(message) = validate_text_fields(&updated.title, description) {
            return api_error(StatusCode::BAD_REQUEST, message);
        }
        updated.description = description.trim().to_string();
    }
    if let Some(filename) = payload.filename {
        updated.filename = clean_filename(Some(filename));
    }
    updated.updated_at = Utc::now().to_rfc3339();

    let path = app_path(category, &slug);
    let previous_html = if payload.html.is_some() {
        match tokio::fs::read(&path).await {
            Ok(contents) => Some(contents),
            Err(error) => {
                tracing::error!("failed to read {} before update: {error}", path.display());
                return api_error(StatusCode::INTERNAL_SERVER_ERROR, "failed to read HTML");
            }
        }
    } else {
        None
    };
    if let Some(html) = payload.html {
        if let Err(error) = atomic_write(&path, html.as_bytes()).await {
            tracing::error!("failed to update {}: {error}", path.display());
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, "failed to save HTML");
        }
    }
    let old = std::mem::replace(&mut apps[index], updated.clone());
    if let Err(error) = save_manifest(&apps).await {
        apps[index] = old;
        if let Some(previous_html) = previous_html {
            if let Err(rollback_error) = atomic_write(&path, &previous_html).await {
                tracing::error!(
                    "failed to roll back {} after manifest error: {rollback_error}",
                    path.display()
                );
            }
        }
        tracing::error!("failed to update app manifest: {error}");
        return api_error(StatusCode::INTERNAL_SERVER_ERROR, "failed to save manifest");
    }
    api_response(Json(updated).into_response())
}

async fn delete_app(
    State(state): State<Arc<AppState>>,
    Path((category, slug)): Path<(String, String)>,
) -> Response {
    let category = match parse_category(&category) {
        Some(category) if is_valid_slug(&slug) => category,
        _ => return api_error(StatusCode::NOT_FOUND, "app not found"),
    };
    let _mutation = state.html_app_mutations.lock().await;
    let mut apps = state.html_apps.write().await;
    let Some(index) = apps
        .iter()
        .position(|app| app.category == category && app.slug == slug)
    else {
        return api_error(StatusCode::NOT_FOUND, "app not found");
    };
    let removed = apps.remove(index);
    if let Err(error) = save_manifest(&apps).await {
        apps.insert(index, removed);
        tracing::error!("failed to update app manifest: {error}");
        return api_error(StatusCode::INTERNAL_SERVER_ERROR, "failed to save manifest");
    }
    let path = app_path(category, &slug);
    match tokio::fs::remove_file(&path).await {
        Ok(()) => api_response(StatusCode::NO_CONTENT.into_response()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            api_response(StatusCode::NO_CONTENT.into_response())
        }
        Err(error) => {
            tracing::error!("failed to delete {}: {error}", path.display());
            apps.insert(index, removed);
            if let Err(rollback_error) = save_manifest(&apps).await {
                tracing::error!("failed to restore manifest after delete error: {rollback_error}");
            }
            api_error(StatusCode::INTERNAL_SERVER_ERROR, "failed to delete HTML")
        }
    }
}

async fn serve_tool(state: State<Arc<AppState>>, Path(slug): Path<String>) -> Response {
    serve_app(state, HtmlAppCategory::Tools, slug).await
}

async fn serve_game(state: State<Arc<AppState>>, Path(slug): Path<String>) -> Response {
    serve_app(state, HtmlAppCategory::Games, slug).await
}

async fn serve_app(
    State(state): State<Arc<AppState>>,
    category: HtmlAppCategory,
    slug: String,
) -> Response {
    if !is_valid_slug(&slug)
        || !state
            .html_apps
            .read()
            .await
            .iter()
            .any(|app| app.category == category && app.slug == slug)
    {
        return StatusCode::NOT_FOUND.into_response();
    }
    let bytes = match tokio::fs::read(app_path(category, &slug)).await {
        Ok(bytes) => bytes,
        Err(_) => return StatusCode::NOT_FOUND.into_response(),
    };
    app_html_response(bytes)
}

fn app_html_response(bytes: Vec<u8>) -> Response {
    let mut response = bytes.into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/html; charset=utf-8"),
    );
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    apply_app_security_headers(&mut response);
    response
}

async fn app_security_headers(req: Request, next: Next) -> Response {
    let mut response = next.run(req).await;
    apply_app_security_headers(&mut response);
    response
}

fn apply_app_security_headers(response: &mut Response) {
    let headers = response.headers_mut();
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(APP_CSP),
    );
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("no-referrer"),
    );
    headers.insert(
        HeaderName::from_static("permissions-policy"),
        HeaderValue::from_static("camera=(), microphone=(), geolocation=(), payment=()"),
    );
    headers.insert(
        header::STRICT_TRANSPORT_SECURITY,
        HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    );
    headers.insert(
        HeaderName::from_static("cross-origin-opener-policy"),
        HeaderValue::from_static("same-origin"),
    );
    headers.insert(
        HeaderName::from_static("cross-origin-resource-policy"),
        HeaderValue::from_static("same-origin"),
    );
}

fn parse_category(value: &str) -> Option<HtmlAppCategory> {
    match value {
        "tools" => Some(HtmlAppCategory::Tools),
        "games" => Some(HtmlAppCategory::Games),
        _ => None,
    }
}

fn app_path(category: HtmlAppCategory, slug: &str) -> PathBuf {
    PathBuf::from(APPS_ROOT)
        .join(category.as_str())
        .join(format!("{slug}.html"))
}

fn app_url(category: HtmlAppCategory, slug: &str) -> String {
    format!("/{}/{}", category.as_str(), slug)
}

fn is_valid_slug(slug: &str) -> bool {
    let windows_reserved = matches!(
        slug,
        "con"
            | "prn"
            | "aux"
            | "nul"
            | "com1"
            | "com2"
            | "com3"
            | "com4"
            | "com5"
            | "com6"
            | "com7"
            | "com8"
            | "com9"
            | "lpt1"
            | "lpt2"
            | "lpt3"
            | "lpt4"
            | "lpt5"
            | "lpt6"
            | "lpt7"
            | "lpt8"
            | "lpt9"
    );
    !windows_reserved
        && !slug.is_empty()
        && slug.len() <= 80
        && !slug.starts_with('-')
        && !slug.ends_with('-')
        && !slug.contains("--")
        && slug
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn resolve_slug(slug: Option<&str>, filename: Option<&str>) -> Result<String, &'static str> {
    let explicit = slug.unwrap_or("").trim();
    let candidate = if explicit.is_empty() {
        let filename = filename
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or("slug or filename is required")?;
        let basename = filename.rsplit(['/', '\\']).next().unwrap_or(filename);
        let basename = if basename.to_ascii_lowercase().ends_with(".html") {
            &basename[..basename.len() - 5]
        } else if basename.to_ascii_lowercase().ends_with(".htm") {
            &basename[..basename.len() - 4]
        } else {
            basename
        };
        slugify(basename)
    } else {
        explicit.to_string()
    };
    if is_valid_slug(&candidate) {
        Ok(candidate)
    } else {
        Err("slug must contain only lowercase ASCII letters, digits and single hyphens")
    }
}

fn slugify(value: &str) -> String {
    let mut result = String::new();
    let mut last_hyphen = false;
    for byte in value.bytes() {
        let byte = byte.to_ascii_lowercase();
        if byte.is_ascii_lowercase() || byte.is_ascii_digit() {
            result.push(byte as char);
            last_hyphen = false;
        } else if !result.is_empty() && !last_hyphen {
            result.push('-');
            last_hyphen = true;
        }
    }
    result.trim_matches('-').chars().take(80).collect()
}

fn validate_text_fields(title: &str, description: &str) -> Result<(), &'static str> {
    if title.trim().is_empty() || title.chars().count() > 200 || title.contains('\0') {
        return Err("title must be between 1 and 200 characters");
    }
    if description.chars().count() > 1000 || description.contains('\0') {
        return Err("description must be at most 1000 characters");
    }
    Ok(())
}

fn validate_html(html: &str) -> Result<(), &'static str> {
    if html.trim().is_empty() {
        return Err("HTML must not be empty");
    }
    if html.len() > MAX_HTML_BYTES {
        return Err("HTML exceeds the 2 MiB limit");
    }
    if html.contains('\0') {
        return Err("HTML contains a NUL byte");
    }
    Ok(())
}

fn clean_filename(filename: Option<String>) -> Option<String> {
    filename.and_then(|value| {
        value
            .rsplit(['/', '\\'])
            .next()
            .map(str::trim)
            .filter(|value| !value.is_empty() && !value.contains('\0'))
            .map(|value| value.chars().take(255).collect())
    })
}

fn sort_apps(apps: &mut [HtmlAppMeta]) {
    apps.sort_by(|left, right| {
        left.category
            .as_str()
            .cmp(right.category.as_str())
            .then_with(|| left.slug.cmp(&right.slug))
    });
}

async fn save_manifest(apps: &[HtmlAppMeta]) -> std::io::Result<()> {
    let contents = toml::to_string_pretty(&HtmlAppManifest {
        apps: apps.to_vec(),
    })
    .map_err(std::io::Error::other)?;
    atomic_write(&PathBuf::from(MANIFEST_PATH), contents.as_bytes()).await
}

async fn atomic_write(path: &std::path::Path, contents: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let mut random = [0_u8; 8];
    rand::rng().fill_bytes(&mut random);
    let suffix: String = random.iter().map(|byte| format!("{byte:02x}")).collect();
    let temp = path.with_extension(format!("tmp-{suffix}"));
    let mut file = tokio::fs::File::create(&temp).await?;
    file.write_all(contents).await?;
    file.flush().await?;
    file.sync_all().await?;
    drop(file);

    if tokio::fs::try_exists(path).await.unwrap_or(false) {
        let backup = backup_path(path);
        if tokio::fs::try_exists(&backup).await.unwrap_or(false) {
            tokio::fs::remove_file(&backup).await?;
        }
        tokio::fs::rename(path, &backup).await?;
        if let Err(error) = tokio::fs::rename(&temp, path).await {
            if let Err(rollback_error) = tokio::fs::rename(&backup, path).await {
                tracing::error!(
                    "failed to restore {} after replacement error: {rollback_error}",
                    path.display()
                );
            }
            let _ = tokio::fs::remove_file(&temp).await;
            return Err(error);
        }
        if let Err(error) = tokio::fs::remove_file(&backup).await {
            tracing::warn!(
                "failed to remove stale backup {}: {error}",
                backup.display()
            );
        }
    } else if let Err(error) = tokio::fs::rename(&temp, path).await {
        let _ = tokio::fs::remove_file(&temp).await;
        return Err(error);
    }
    Ok(())
}

fn backup_path(path: &FsPath) -> PathBuf {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("file");
    path.with_extension(format!("{extension}.bak"))
}

async fn recover_backup(path: &FsPath) {
    let backup = backup_path(path);
    let target_exists = tokio::fs::try_exists(path).await.unwrap_or(false);
    let backup_exists = tokio::fs::try_exists(&backup).await.unwrap_or(false);
    match (target_exists, backup_exists) {
        (false, true) => {
            if let Err(error) = tokio::fs::rename(&backup, path).await {
                tracing::error!("failed to recover backup {}: {error}", backup.display());
            }
        }
        (true, true) => {
            if let Err(error) = tokio::fs::remove_file(&backup).await {
                tracing::warn!(
                    "failed to remove stale backup {}: {error}",
                    backup.display()
                );
            }
        }
        _ => {}
    }
}

fn api_error(status: StatusCode, message: &'static str) -> Response {
    api_response((status, Json(json!({ "error": message }))).into_response())
}

fn json_rejection(rejection: axum::extract::rejection::JsonRejection) -> Response {
    let status = rejection.status();
    let message = if status == StatusCode::PAYLOAD_TOO_LARGE {
        "request body is too large"
    } else {
        "invalid JSON"
    };
    api_error(status, message)
}

fn api_response(mut response: Response) -> Response {
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_safe_slugs() {
        assert!(is_valid_slug("curl-to-code"));
        assert!(is_valid_slug("2048"));
        for bad in [
            "", "Curl", "/note", "../note", "a/b", "a\\b", "a--b", "-a", "a-", "con", "nul",
            "com1", "lpt9",
        ] {
            assert!(!is_valid_slug(bad), "accepted unsafe slug: {bad}");
        }
    }

    #[test]
    fn derives_slug_from_filename() {
        assert_eq!(
            resolve_slug(None, Some("My Curl Converter.html")),
            Ok("my-curl-converter".to_string())
        );
        assert_eq!(resolve_slug(Some(""), Some("2048.html")), Ok("2048".into()));
        assert!(resolve_slug(None, Some("你好.html")).is_err());
        assert_eq!(resolve_slug(None, Some("GAME.HTML")), Ok("game".into()));
    }

    #[test]
    fn app_response_uses_isolated_csp() {
        let response = app_html_response(b"<script>globalThis.ok=true</script>".to_vec());
        let csp = response
            .headers()
            .get(header::CONTENT_SECURITY_POLICY)
            .unwrap()
            .to_str()
            .unwrap();
        assert!(csp.contains("sandbox allow-scripts"));
        assert!(!csp.contains("allow-same-origin"));
        assert!(csp.contains("connect-src 'none'"));
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            "text/html; charset=utf-8"
        );
    }

    #[tokio::test]
    async fn atomic_write_replaces_existing_file_on_windows() {
        let directory = std::env::temp_dir().join(format!(
            "lily-nest-html-app-test-{}-{}",
            std::process::id(),
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ));
        tokio::fs::create_dir_all(&directory).await.unwrap();
        let path = directory.join("demo.html");

        atomic_write(&path, b"old").await.unwrap();
        atomic_write(&path, b"new").await.unwrap();

        assert_eq!(tokio::fs::read(&path).await.unwrap(), b"new");
        assert!(!tokio::fs::try_exists(backup_path(&path)).await.unwrap());
        tokio::fs::remove_dir_all(&directory).await.unwrap();
    }

    #[tokio::test]
    async fn recovers_interrupted_replacement_from_backup() {
        let directory = std::env::temp_dir().join(format!(
            "lily-nest-html-app-recovery-test-{}-{}",
            std::process::id(),
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ));
        tokio::fs::create_dir_all(&directory).await.unwrap();
        let path = directory.join("demo.html");
        let backup = backup_path(&path);
        tokio::fs::write(&backup, b"known-good").await.unwrap();

        recover_backup(&path).await;

        assert_eq!(tokio::fs::read(&path).await.unwrap(), b"known-good");
        assert!(!tokio::fs::try_exists(&backup).await.unwrap());
        tokio::fs::remove_dir_all(&directory).await.unwrap();
    }
}
