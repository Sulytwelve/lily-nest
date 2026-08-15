use crate::{
    model::{AdminNoteSaveRequest, NoteFrontmatter, NoteSummary},
    state::AppState,
};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    middleware,
    routing::get,
};
use chrono::Local;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/admin/notes", get(list_notes).post(create_note))
        .route(
            "/admin/notes/{slug}",
            get(get_note).put(update_note).delete(delete_note),
        )
        // 预留给 Agent 调用的无 /admin 前缀 REST API
        .route("/api/v1/notes", get(list_notes).post(create_note))
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

async fn create_note(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AdminNoteSaveRequest>,
) -> Result<StatusCode, StatusCode> {
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
    Json(payload): Json<AdminNoteSaveRequest>,
) -> Result<StatusCode, StatusCode> {
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

    if let Err(e) = tokio::fs::write(&tmp, file_content).await {
        tracing::error!("写入临时笔记文件失败 ({}): {}", tmp, e);
        let _ = tokio::fs::remove_file(&tmp).await;
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

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
