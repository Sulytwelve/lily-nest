use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router, middleware,
};
use std::sync::Arc;
use crate::{
    model::{AdminNoteSaveRequest, NoteSummary, NoteFrontmatter},
    state::AppState,
};
use chrono::Local;

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/admin/notes", get(list_notes).post(create_note))
        .route("/admin/notes/{slug}", get(get_note).put(update_note).delete(delete_note))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            crate::middlewares::admin_auth_middleware,
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
        index.iter().find(|n| n.meta.slug == slug).map(|n| n.filename.clone())
    };

    if let Some(filename) = filename {
        let file_path = format!("notes/{}", filename);
        if let Ok(content) = tokio::fs::read_to_string(&file_path).await {
            if let Some((meta, markdown_body)) = crate::note_loader::parse_note(&content) {
                return Ok(Json(AdminNoteSaveRequest {
                    title: meta.title,
                    tags: meta.tags,
                    excerpt: meta.excerpt,
                    content: markdown_body.trim().to_string(),
                    original_slug: Some(meta.slug),
                }));
            }
        }
    }
    Err(StatusCode::NOT_FOUND)
}

fn build_note_file_content(meta: &NoteFrontmatter, content: &str) -> String {
    let toml_str = toml::to_string(meta).unwrap_or_default();
    format!("---\n{}---\n\n{}", toml_str, content)
}

async fn create_note(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AdminNoteSaveRequest>,
) -> Result<StatusCode, StatusCode> {
    let now = Local::now();
    let date_str = now.format("%Y-%m-%dT%H:%M:%S%z").to_string();
    
    // simple slug generation
    let slug = payload.title.to_lowercase().replace(" ", "-").replace("/", "");
    let date_prefix = now.format("%Y%m%d-%H%M%S").to_string();
    let filename = format!("{}-{}.md", date_prefix, slug);
    let filepath = format!("notes/{}", filename);

    if tokio::fs::metadata(&filepath).await.is_ok() {
        return Err(StatusCode::CONFLICT);
    }

    let meta = NoteFrontmatter {
        title: payload.title,
        date: date_str,
        updated_at: None,
        slug: slug.clone(),
        tags: payload.tags,
        excerpt: payload.excerpt,
    };

    let file_content = build_note_file_content(&meta, &payload.content);
    tokio::fs::write(&filepath, file_content).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // reload index
    {
        let mut index = state.note_index.write().await;
        *index = crate::note_loader::load_all_notes();
    }
    {
        *state.note_list_html_cache.write().await = None;
    }

    Ok(StatusCode::CREATED)
}

async fn update_note(
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
    Json(payload): Json<AdminNoteSaveRequest>,
) -> Result<StatusCode, StatusCode> {
    let (old_filename, old_date) = {
        let index = state.note_index.read().await;
        let note = index.iter().find(|n| n.meta.slug == slug);
        match note {
            Some(n) => (n.filename.clone(), n.meta.date.clone()),
            None => return Err(StatusCode::NOT_FOUND),
        }
    };

    let now = Local::now();
    let updated_at_str = now.format("%Y-%m-%dT%H:%M:%S%z").to_string();
    
    let meta = NoteFrontmatter {
        title: payload.title.clone(),
        date: old_date,
        updated_at: Some(updated_at_str),
        slug: slug.clone(),
        tags: payload.tags,
        excerpt: payload.excerpt,
    };

    let file_content = build_note_file_content(&meta, &payload.content);
    let filepath = format!("notes/{}", old_filename);
    tokio::fs::write(&filepath, file_content).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // reload cache
    {
        let mut index = state.note_index.write().await;
        *index = crate::note_loader::load_all_notes();
    }
    {
        *state.note_list_html_cache.write().await = None;
        state.note_html_cache.write().await.remove(&slug);
    }

    Ok(StatusCode::OK)
}

async fn delete_note(
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let filename = {
        let index = state.note_index.read().await;
        index.iter().find(|n| n.meta.slug == slug).map(|n| n.filename.clone())
    };

    if let Some(filename) = filename {
        let filepath = format!("notes/{}", filename);
        let _ = tokio::fs::remove_file(&filepath).await;

        let mut index = state.note_index.write().await;
        *index = crate::note_loader::load_all_notes();

        *state.note_list_html_cache.write().await = None;
        state.note_html_cache.write().await.remove(&slug);

        return Ok(StatusCode::OK);
    }

    Err(StatusCode::NOT_FOUND)
}
