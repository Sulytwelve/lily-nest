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

/// 开发模式：从磁盘重载索引
async fn reload_index_in_debug(state: &Arc<AppState>) {
    if cfg!(debug_assertions) {
        let mut index = state.note_index.write().await;
        *index = crate::note_loader::load_all_notes().await;
    }
}

/// 解析日期字符串：优先按 RFC 3339 解析，失败时回溯旧格式 "%Y-%m-%d %H:%M:%S"
fn format_date(date_str: &str) -> String {
    chrono::DateTime::parse_from_rfc3339(date_str)
        .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
        .or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M:%S")
                .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
        })
        .unwrap_or_else(|_| date_str.to_string())
}

async fn handle_note_list(State(state): State<Arc<AppState>>) -> Response {
    reload_index_in_debug(&state).await;

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
        let excerpt = crate::utils::html_escape(&note.meta.excerpt.clone().unwrap_or_default());
        let title = crate::utils::html_escape(&note.meta.title);

        let tags_html = note.meta.tags.iter()
            .map(|t| format!(r#"<span class="tag">#{}</span>"#, crate::utils::html_escape(t)))
            .collect::<String>();

        let display_date = format_date(&note.meta.date);

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
            crate::utils::html_escape(&note.meta.slug), crate::utils::html_escape(&display_date), title, excerpt, tags_html
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
    reload_index_in_debug(&state).await;

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
                res.headers_mut().insert(
                    header::CACHE_CONTROL,
                    HeaderValue::from_static("private, no-cache, no-store, must-revalidate"),
                );
                res.headers_mut().insert(
                    header::VARY,
                    HeaderValue::from_static("Accept"),
                );
                return res;
            }

            if let Some((meta, markdown_body)) = crate::note_loader::parse_note(&content) {
                let parser = Parser::new(&markdown_body);
                let mut html_output = String::new();
                html::push_html(&mut html_output, parser);

                let template = tokio::fs::read_to_string("templates/note_detail.html").await.unwrap_or_else(|_| {
                    "<!DOCTYPE html><html><body><article><h1>{{title}}</h1><div class='content'>{{content}}</div></article></body></html>".to_string()
                });

                let display_date = format_date(&meta.date);

                let updated_at_html = if let Some(updated) = &meta.updated_at {
                    let disp = format_date(updated);
                    format!(r#"<span class="note-updated" style="margin-left: 8px;">(最后修改: <time datetime="{}">{}</time>)</span>"#,
                            crate::utils::html_escape(updated),
                            crate::utils::html_escape(&disp))
                } else {
                    String::new()
                };

                let final_html = template
                    .replace("{{title}}", &crate::utils::html_escape(&meta.title))
                    .replace("{{date}}", &crate::utils::html_escape(&display_date))
                    .replace("{{updated_at_html}}", &updated_at_html)
                    .replace("{{content}}", &html_output);

                let bytes = Bytes::from(final_html);

                if !cfg!(debug_assertions) {
                    let mut cache = state.note_html_cache.write().await;
                    if cache.len() > 20 {
                        // 淘汰一条而非清空全部，避免缓存雪崩
                        if let Some(key) = cache.keys().next().cloned() {
                            cache.remove(&key);
                        }
                    }
                    cache.insert(slug.clone(), crate::state::NoteHtmlCache { body: bytes.clone() });
                }

                let mut res = Response::new(axum::body::Body::from(bytes));
                res.headers_mut().insert(header::CONTENT_TYPE, HeaderValue::from_static("text/html; charset=utf-8"));
                res.headers_mut().insert(
                    header::VARY,
                    HeaderValue::from_static("Accept"),
                );
                res.headers_mut().insert(
                    header::LINK,
                    HeaderValue::from_static("</?format=markdown>; rel=\"alternate\"; type=\"text/markdown\""),
                );
                return res;
            }
        }
    }

    StatusCode::NOT_FOUND.into_response()
}
