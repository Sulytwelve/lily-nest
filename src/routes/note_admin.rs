use crate::{
    model::{AdminNoteSaveRequest, NoteFrontmatter, NoteSummary},
    state::AppState,
};
use axum::{
    Json, Router,
    body::to_bytes,
    extract::{Path, Request, State},
    http::{HeaderValue, StatusCode, header},
    middleware,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use chrono::Local;
use rand::Rng;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::io::AsyncWriteExt;

const MAX_NOTE_IMAGE_BYTES: usize = 5 * 1024 * 1024;
const NOTE_IMAGE_DIR: &str = "static/images/notes";

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/admin/notes", get(list_notes).post(create_note))
        .route("/admin/notes/images", post(upload_note_image))
        .route(
            "/admin/notes/{slug}",
            get(get_note).put(update_note).delete(delete_note),
        )
        // 预留给 Agent 调用的无 /admin 前缀 REST API
        .route("/api/v1/notes", get(list_notes).post(create_note))
        .route("/api/v1/notes/images", post(upload_note_image))
        .route(
            "/api/v1/notes/{slug}",
            get(get_note).put(update_note).delete(delete_note),
        )
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            crate::middlewares::note_auth_middleware,
        ))
        .with_state(state)
}

async fn list_notes(State(state): State<Arc<AppState>>) -> Json<Vec<NoteSummary>> {
    let index = state.note_index.read().await;
    Json(index.clone())
}

async fn get_note(
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
) -> Result<Json<AdminNoteSaveRequest>, StatusCode> {
    let filename = {
        let index = state.note_index.read().await;
        index
            .iter()
            .find(|n| n.meta.slug == slug)
            .map(|n| n.filename.clone())
    };

    if let Some(filename) = filename {
        let file_path = format!("notes/{}", filename);
        if let Ok(content) = tokio::fs::read_to_string(&file_path).await
            && let Some((meta, markdown_body)) = crate::note_loader::parse_note(&content)
        {
            return Ok(Json(AdminNoteSaveRequest {
                title: meta.title,
                tags: meta.tags,
                excerpt: meta.excerpt,
                content: markdown_body.trim().to_string(),
            }));
        }
    }
    Err(StatusCode::NOT_FOUND)
}

fn validate_note_payload(payload: &AdminNoteSaveRequest) -> Result<(), StatusCode> {
    if payload.title.is_empty() || payload.title.chars().count() > 200 {
        return Err(StatusCode::BAD_REQUEST);
    }
    if let Some(excerpt) = payload.excerpt.as_deref()
        && excerpt.chars().count() > 500
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    if payload.tags.len() > 20 {
        return Err(StatusCode::BAD_REQUEST);
    }
    for tag in &payload.tags {
        if tag.is_empty() || tag.chars().count() > 64 {
            return Err(StatusCode::BAD_REQUEST);
        }
    }
    if payload.content.chars().count() > 262_144 {
        return Err(StatusCode::BAD_REQUEST);
    }
    if payload.title.contains('\0')
        || payload.content.contains('\0')
        || payload.excerpt.as_deref().is_some_and(|e| e.contains('\0'))
        || payload.tags.iter().any(|t| t.contains('\0'))
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    Ok(())
}

fn build_note_file_content(meta: &NoteFrontmatter, content: &str) -> Result<String, String> {
    let toml_str = toml::to_string(meta).map_err(|e| e.to_string())?;
    Ok(format!("---\n{}---\n\n{}", toml_str, content))
}

/// 仅允许 PNG / JPEG / GIF / WebP，且用魔数校验防止“改了 Content-Type 就传任意文件”。
fn detect_note_image_ext(content_type: &str, bytes: &[u8]) -> Option<&'static str> {
    let mime = content_type.split(';').next().unwrap_or("").trim();
    match mime {
        "image/png" => {
            if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
                Some("png")
            } else {
                None
            }
        }
        "image/jpeg" => {
            if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
                Some("jpg")
            } else {
                None
            }
        }
        "image/gif" => {
            if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
                Some("gif")
            } else {
                None
            }
        }
        "image/webp" => {
            if bytes.starts_with(b"RIFF") && bytes.len() >= 12 && bytes[8..12].starts_with(b"WEBP")
            {
                Some("webp")
            } else {
                None
            }
        }
        _ => None,
    }
}

/// POST /admin/notes/images 与 /api/v1/notes/images
/// 接收原始图片字节（Content-Type: image/png 等），保存到 static/images/notes/ 并返回 Markdown 可用的 URL。
async fn upload_note_image(req: Request) -> Response {
    if let Some(cl) = req
        .headers()
        .get(header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<usize>().ok())
        && cl > MAX_NOTE_IMAGE_BYTES
    {
        return (StatusCode::PAYLOAD_TOO_LARGE, "Image too large").into_response();
    }

    let content_type = req
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_ascii_lowercase();

    let body_bytes = match to_bytes(req.into_body(), MAX_NOTE_IMAGE_BYTES).await {
        Ok(bytes) => bytes,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, "Failed to read image body").into_response();
        }
    };

    if body_bytes.is_empty() {
        return (StatusCode::BAD_REQUEST, "Empty image body").into_response();
    }

    let Some(ext) = detect_note_image_ext(&content_type, &body_bytes) else {
        return (
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "Only PNG, JPG, GIF and WebP images are allowed",
        )
            .into_response();
    };

    if let Err(e) = tokio::fs::create_dir_all(NOTE_IMAGE_DIR).await {
        tracing::error!("创建笔记图片目录失败 ({}): {}", NOTE_IMAGE_DIR, e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to create image directory",
        )
            .into_response();
    }

    let mut random_bytes = [0u8; 8];
    rand::rng().fill_bytes(&mut random_bytes);
    let suffix: String = random_bytes.iter().map(|b| format!("{:02x}", b)).collect();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let filename = format!("{}-{}.{}", nanos, suffix, ext);
    let filepath = format!("{}/{}", NOTE_IMAGE_DIR, filename);

    if let Err(e) = tokio::fs::write(&filepath, &body_bytes).await {
        tracing::error!("保存笔记图片失败 ({}): {}", filepath, e);
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save image").into_response();
    }

    let url = format!("/images/notes/{}", filename);
    tracing::info!("上传笔记图片成功: {}, size={}", filepath, body_bytes.len());

    let mut res = Json(serde_json::json!({
        "url": url,
        "filename": filename,
    }))
    .into_response();
    res.headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    res
}

async fn create_note(
    State(state): State<Arc<AppState>>,
    payload: Result<Json<AdminNoteSaveRequest>, axum::extract::rejection::JsonRejection>,
) -> Result<StatusCode, StatusCode> {
    let Json(payload) = payload.map_err(|_| StatusCode::BAD_REQUEST)?;
    validate_note_payload(&payload)?;

    let now = Local::now();
    let date_str = now.to_rfc3339();

    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    let slug = format!("{}-{}", now.format("%Y%m%d%H%M%S"), nanos);
    let date_prefix = now.format("%Y%m%d-%H%M%S").to_string();
    let filename = format!("{}-{}.md", date_prefix, slug);
    let filepath = format!("notes/{}", filename);

    let meta = NoteFrontmatter {
        title: payload.title,
        date: date_str,
        updated_at: None,
        slug: slug.clone(),
        tags: payload.tags,
        excerpt: payload.excerpt,
    };

    let file_content = build_note_file_content(&meta, &payload.content).map_err(|e| {
        tracing::error!("序列化笔记 frontmatter 失败 ({}): {}", filepath, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // B27：原子独占创建，避免 check-then-write 的 TOCTOU 覆盖
    let mut file = match tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&filepath)
        .await
    {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            return Err(StatusCode::CONFLICT);
        }
        Err(e) => {
            tracing::error!("创建笔记文件失败 ({}): {}", filepath, e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    if let Err(e) = file.write_all(file_content.as_bytes()).await {
        tracing::error!("写入笔记文件失败 ({}): {}", filepath, e);
        drop(file);
        let _ = tokio::fs::remove_file(&filepath).await;
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    if let Err(e) = file.flush().await {
        tracing::error!("刷新笔记文件失败 ({}): {}", filepath, e);
        drop(file);
        let _ = tokio::fs::remove_file(&filepath).await;
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    // M3：数据落盘后再关文件，避免崩溃后文件内容为空
    if let Err(e) = file.sync_all().await {
        tracing::error!("同步笔记文件失败 ({}): {}", filepath, e);
        drop(file);
        let _ = tokio::fs::remove_file(&filepath).await;
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    drop(file);

    // B28：在锁外完成全盘加载，锁内只做指针替换
    let new_index = crate::note_loader::load_all_notes().await;
    {
        let mut index = state.note_index.write().await;
        *index = new_index;
    }
    {
        *state.note_list_html_cache.write().await = None;
    }

    tracing::info!("创建笔记成功: slug={}, filename={}", slug, filename);
    Ok(StatusCode::CREATED)
}

async fn update_note(
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
    payload: Result<Json<AdminNoteSaveRequest>, axum::extract::rejection::JsonRejection>,
) -> Result<StatusCode, StatusCode> {
    let Json(payload) = payload.map_err(|_| StatusCode::BAD_REQUEST)?;
    validate_note_payload(&payload)?;

    let (old_filename, old_date) = {
        let index = state.note_index.read().await;
        let note = index.iter().find(|n| n.meta.slug == slug);
        match note {
            Some(n) => (n.filename.clone(), n.meta.date.clone()),
            None => return Err(StatusCode::NOT_FOUND),
        }
    };

    let now = Local::now();
    let updated_at_str = now.to_rfc3339();

    let meta = NoteFrontmatter {
        title: payload.title.clone(),
        date: old_date,
        updated_at: Some(updated_at_str),
        slug: slug.clone(),
        tags: payload.tags,
        excerpt: payload.excerpt,
    };

    let file_content = build_note_file_content(&meta, &payload.content).map_err(|e| {
        tracing::error!("序列化笔记 frontmatter 失败 ({}): {}", old_filename, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let filepath = format!("notes/{}", old_filename);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let tmp = format!("{}.{}.{}.tmp", filepath, std::process::id(), nanos);

    let mut tmp_file = match tokio::fs::File::create(&tmp).await {
        Ok(f) => f,
        Err(e) => {
            tracing::error!("创建临时笔记文件失败 ({}): {}", tmp, e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    if let Err(e) = tmp_file.write_all(file_content.as_bytes()).await {
        tracing::error!("写入临时笔记文件失败 ({}): {}", tmp, e);
        drop(tmp_file);
        let _ = tokio::fs::remove_file(&tmp).await;
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    if let Err(e) = tmp_file.flush().await {
        tracing::error!("刷新临时笔记文件失败 ({}): {}", tmp, e);
        drop(tmp_file);
        let _ = tokio::fs::remove_file(&tmp).await;
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    if let Err(e) = tmp_file.sync_all().await {
        tracing::error!("同步临时笔记文件失败 ({}): {}", tmp, e);
        drop(tmp_file);
        let _ = tokio::fs::remove_file(&tmp).await;
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    drop(tmp_file);

    if let Err(e) = tokio::fs::rename(&tmp, &filepath).await {
        tracing::error!("替换笔记文件失败 ({} -> {}): {}", tmp, filepath, e);
        let _ = tokio::fs::remove_file(&tmp).await;
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    // B28：在锁外完成全盘加载，锁内只做指针替换
    let new_index = crate::note_loader::load_all_notes().await;
    {
        let mut index = state.note_index.write().await;
        *index = new_index;
    }
    {
        *state.note_list_html_cache.write().await = None;
        state.note_html_cache.write().await.remove(&slug);
    }

    tracing::info!("更新笔记成功: slug={}, filename={}", slug, old_filename);
    Ok(StatusCode::OK)
}

async fn delete_note(
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let filename = {
        let index = state.note_index.read().await;
        index
            .iter()
            .find(|n| n.meta.slug == slug)
            .map(|n| n.filename.clone())
    };

    if let Some(filename) = filename {
        let filepath = format!("notes/{}", filename);
        tokio::fs::remove_file(&filepath).await.map_err(|e| {
            tracing::error!("删除笔记文件失败 ({}): {}", filepath, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        // B28：在锁外完成全盘加载，锁内只做指针替换
        let new_index = crate::note_loader::load_all_notes().await;
        {
            let mut index = state.note_index.write().await;
            *index = new_index;
        }

        *state.note_list_html_cache.write().await = None;
        state.note_html_cache.write().await.remove(&slug);

        tracing::info!("删除笔记成功: slug={}, filename={}", slug, filename);
        return Ok(StatusCode::OK);
    }

    Err(StatusCode::NOT_FOUND)
}
