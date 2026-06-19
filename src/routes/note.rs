use axum::{
    extract::{Path, State},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use std::sync::Arc;
use crate::state::AppState;
use bytes::Bytes;
use pulldown_cmark::{html, Parser};
use serde::Deserialize;

#[derive(Deserialize, Default)]
struct NoteQuery {
    format: Option<String>,
}

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/note", get(handle_note_list))
        .route("/note/{slug}", get(handle_note_detail))
        .with_state(state)
}

async fn handle_note_list(State(state): State<Arc<AppState>>) -> Response {
    // 开发模式：每次从磁盘重载索引，确保直接编辑 .md 文件也能立即反映
    if cfg!(debug_assertions) {
        let mut index = state.note_index.write().await;
        *index = crate::note_loader::load_all_notes().await;
    }

    if let Some(cached) = state.note_list_html_cache.read().await.clone() {
        if !cfg!(debug_assertions) {
            let mut res = Response::new(axum::body::Body::from(cached));
            res.headers_mut().insert(header::CONTENT_TYPE, HeaderValue::from_static("text/html; charset=utf-8"));
            return res;
        }
    }

    let notes = state.note_index.read().await;
    
    let template = tokio::fs::read_to_string("templates/note.html").await.unwrap_or_else(|_| {
        "<!DOCTYPE html><html><body><h1>templates/note.html not found</h1><div id='mainNotesList'>{{notes_html}}</div><script>const publicNotes = {{notes_json}};</script></body></html>".to_string()
    });

    let mut notes_html = String::new();
    for note in notes.iter() {
        let excerpt = crate::render::html_escape(&note.meta.excerpt.clone().unwrap_or_default());
        let title = crate::render::html_escape(&note.meta.title);
        
        let tags_html = note.meta.tags.iter()
            .map(|t| format!(r#"<span class="tag">#{}</span>"#, crate::render::html_escape(t)))
            .collect::<String>();

        let display_date = match chrono::DateTime::parse_from_rfc3339(&note.meta.date) {
            Ok(dt) => dt.format("%Y-%m-%d %H:%M").to_string(),
            Err(_) => note.meta.date.clone(),
        };

        notes_html.push_str(&format!(
            r#"<a class="note-card" style="text-decoration: none; color: inherit; display: flex;" href="/note/{}">
                 <div class="card-meta">
                   <span>{}</span>
                   <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="currentColor"><path d="M6 6v2h8.59L5 17.59 6.41 19 16 9.41V18h2V6z"/></svg>
                 </div>
                 <div class="note-title">{}</div>
                 <div class="note-excerpt">{}</div>
                 <div class="card-tags">{}</div>
               </a>"#,
            crate::render::html_escape(&note.meta.slug), crate::render::html_escape(&display_date), title, excerpt, tags_html
        ));
    }

    let notes_json = serde_json::to_string(&*notes)
        .unwrap_or_else(|_| "[]".to_string())
        .replace("</script>", "<\\/script>");
    
    let final_html = template
        .replace("{{notes_html}}", &notes_html)
        .replace("{{notes_json}}", &notes_json);
        
    let bytes = Bytes::from(final_html);
    if !cfg!(debug_assertions) {
        *state.note_list_html_cache.write().await = Some(bytes.clone());
    }

    let mut res = Response::new(axum::body::Body::from(bytes));
    res.headers_mut().insert(header::CONTENT_TYPE, HeaderValue::from_static("text/html; charset=utf-8"));
    res
}

async fn handle_note_detail(
    State(state): State<Arc<AppState>>, 
    Path(slug): Path<String>,
    axum::extract::Query(query): axum::extract::Query<NoteQuery>,
    req: axum::extract::Request,
) -> Response {
    // 开发模式：每次从磁盘重载索引
    if cfg!(debug_assertions) {
        let mut index = state.note_index.write().await;
        *index = crate::note_loader::load_all_notes().await;
    }

    let wants_markdown = query.format.as_deref() == Some("markdown") || 
        req.headers().get(header::ACCEPT).and_then(|v| v.to_str().ok())
        .map(|v| v.contains("text/markdown") || v.contains("text/x-markdown")).unwrap_or(false);

    if !wants_markdown {
        let cache = state.note_html_cache.read().await;
        if let Some(cached) = cache.get(&slug) {
            if !cfg!(debug_assertions) {
                let mut res = Response::new(axum::body::Body::from(cached.body.clone()));
                res.headers_mut().insert(header::CONTENT_TYPE, HeaderValue::from_static("text/html; charset=utf-8"));
                return res;
            }
        }
    }

    let filename = {
        let index = state.note_index.read().await;
        index.iter().find(|n| n.meta.slug == slug).map(|n| n.filename.clone())
    };

    if let Some(filename) = filename {
        let file_path = format!("notes/{}", filename);
        if let Ok(content) = tokio::fs::read_to_string(&file_path).await {
            if wants_markdown {
                let mut res = Response::new(axum::body::Body::from(content));
                res.headers_mut().insert(header::CONTENT_TYPE, HeaderValue::from_static("text/markdown; charset=utf-8"));
                return res;
            }

            if let Some((meta, markdown_body)) = crate::note_loader::parse_note(&content) {
                let parser = Parser::new(&markdown_body);
                let mut html_output = String::new();
                html::push_html(&mut html_output, parser);

                let template = tokio::fs::read_to_string("templates/note_detail.html").await.unwrap_or_else(|_| {
                    "<!DOCTYPE html><html><body><article><h1>{{title}}</h1><div class='content'>{{content}}</div></article></body></html>".to_string()
                });

                let display_date = match chrono::DateTime::parse_from_rfc3339(&meta.date) {
                    Ok(dt) => dt.format("%Y-%m-%d %H:%M").to_string(),
                    Err(_) => meta.date.clone(),
                };

                let updated_at_html = if let Some(updated) = &meta.updated_at {
                    let disp = match chrono::DateTime::parse_from_rfc3339(updated) {
                        Ok(dt) => dt.format("%Y-%m-%d %H:%M").to_string(),
                        Err(_) => updated.clone(),
                    };
                    format!(r#"<span class="note-updated" style="margin-left: 8px;">(最后修改: <time datetime="{}">{}</time>)</span>"#, 
                            crate::render::html_escape(updated), 
                            crate::render::html_escape(&disp))
                } else {
                    String::new()
                };

                let final_html = template
                    .replace("{{title}}", &crate::render::html_escape(&meta.title))
                    .replace("{{date}}", &crate::render::html_escape(&display_date))
                    .replace("{{updated_at_html}}", &updated_at_html)
                    .replace("{{content}}", &html_output);
                
                let bytes = Bytes::from(final_html);

                if !cfg!(debug_assertions) {
                    let mut cache = state.note_html_cache.write().await;
                    if cache.len() > 20 {
                        cache.clear();
                    }
                    cache.insert(slug.clone(), crate::state::NoteHtmlCache { body: bytes.clone() });
                }

                let mut res = Response::new(axum::body::Body::from(bytes));
                res.headers_mut().insert(header::CONTENT_TYPE, HeaderValue::from_static("text/html; charset=utf-8"));
                return res;
            }
        }
    }

    StatusCode::NOT_FOUND.into_response()
}
