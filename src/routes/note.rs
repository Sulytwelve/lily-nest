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

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/note", get(handle_note_list))
        .route("/note/{slug}", get(handle_note_detail))
        .with_state(state)
}

async fn handle_note_list(State(state): State<Arc<AppState>>) -> Response {
    if let Some(cached) = state.note_list_html_cache.read().await.clone() {
        if !cfg!(debug_assertions) {
            let mut res = Response::new(axum::body::Body::from(cached));
            res.headers_mut().insert(header::CONTENT_TYPE, HeaderValue::from_static("text/html; charset=utf-8"));
            return res;
        }
    }

    let notes = state.note_index.read().await;
    
    // Default barebones HTML. Will be replaced by frontend templates.
    let mut html_content = String::from("<!DOCTYPE html><html><head><meta charset=\"UTF-8\"><title>Notes</title></head><body><h1>Notes Stream</h1><ul>");
    for note in notes.iter() {
        html_content.push_str(&format!(
            r#"<li><a href="/note/{}">{}</a> - {}<br><span>{}</span></li>"#,
            note.meta.slug, note.meta.title, note.meta.date, note.meta.excerpt.clone().unwrap_or_default()
        ));
    }
    html_content.push_str("</ul></body></html>");

    let bytes = Bytes::from(html_content);
    if !cfg!(debug_assertions) {
        *state.note_list_html_cache.write().await = Some(bytes.clone());
    }

    let mut res = Response::new(axum::body::Body::from(bytes));
    res.headers_mut().insert(header::CONTENT_TYPE, HeaderValue::from_static("text/html; charset=utf-8"));
    res
}

async fn handle_note_detail(State(state): State<Arc<AppState>>, Path(slug): Path<String>) -> Response {
    {
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
            if let Some((meta, markdown_body)) = crate::note_loader::parse_note(&content) {
                let parser = Parser::new(&markdown_body);
                let mut html_output = String::new();
                html::push_html(&mut html_output, parser);

                let final_html = format!(
                    "<!DOCTYPE html><html><head><meta charset=\"UTF-8\"><title>{}</title></head><body><article><h1>{}</h1><div class='content'>{}</div></article></body></html>",
                    meta.title, meta.title, html_output
                );
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
