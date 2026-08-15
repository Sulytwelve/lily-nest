use crate::state::AppState;
use axum::{
    Router,
    extract::{Path, State},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use bytes::Bytes;
use pulldown_cmark::{Parser, html};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize, Default)]
struct NoteQuery {
    format: Option<String>,
}

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/note", get(handle_note_list))
        .route(
            "/note/",
            get(|| async { axum::response::Redirect::permanent("/note") }),
        )
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

async fn handle_note_list(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(query): axum::extract::Query<NoteQuery>,
    req: axum::extract::Request,
) -> Response {
    reload_index_in_debug(&state).await;

    let wants_markdown = query.format.as_deref() == Some("markdown")
        || req
            .headers()
            .get(header::ACCEPT)
            .and_then(|v| v.to_str().ok())
            .map(|v| v.contains("text/markdown") || v.contains("text/x-markdown"))
            .unwrap_or(false);

    if wants_markdown {
        let (_, _, note_config) = crate::config::load_site_data();
        let notes = state.note_index.read().await;

        let mut md = String::new();
        md.push_str(&format!("# {}\n\n", note_config.note_title));
        md.push_str(&format!("> {}\n\n---\n\n", note_config.meta_desc));

        for note in notes.iter() {
            let display_date = format_date(&note.meta.date);
            md.push_str(&format!(
                "*   **[{}](/note/{})** — {}\n",
                note.meta.title, note.meta.slug, display_date
            ));
            if let Some(excerpt) = &note.meta.excerpt {
                let clean_excerpt = excerpt.replace('\n', " ");
                md.push_str(&format!("    > {}\n", clean_excerpt));
            }
        }

        let mut res = Response::new(axum::body::Body::from(md));
        res.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/markdown; charset=utf-8"),
        );
        res.headers_mut().insert(
            header::CACHE_CONTROL,
            HeaderValue::from_static("private, no-cache, no-store, must-revalidate"),
        );
        res.headers_mut()
            .insert(header::VARY, HeaderValue::from_static("Accept"));
        return res;
    }

    if let Some(cached) = state.note_list_html_cache.read().await.clone() {
        if !cfg!(debug_assertions) {
            let mut res = Response::new(axum::body::Body::from(cached));
            res.headers_mut().insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("text/html; charset=utf-8"),
            );
            res.headers_mut().insert(
                header::CACHE_CONTROL,
                HeaderValue::from_static("public, max-age=300"),
            );
            res.headers_mut()
                .insert(header::VARY, HeaderValue::from_static("Accept"));
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

        let tags_html = note
            .meta
            .tags
            .iter()
            .map(|t| {
                format!(
                    r#"<span class="tag">#{}</span>"#,
                    crate::utils::html_escape(t)
                )
            })
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

    let notes_json = crate::utils::escape_json_for_html_script(
        &serde_json::to_string(&*notes).unwrap_or_else(|_| "[]".to_string()),
    );

    let (_, site_config, note_config) = crate::config::load_site_data();

    let raw_head = site_config
        .custom_head
        .as_deref()
        .unwrap_or_default()
        .trim();
    let custom_head = raw_head
        .replace('\n', "\n    ")
        .replace("{{url_path}}", "note");

    let raw_footer = site_config
        .footer_html
        .as_deref()
        .unwrap_or_default()
        .trim();
    let footer_html = if raw_footer.is_empty() {
        "".to_string()
    } else {
        format!(r#"<footer class="site-footer">{}</footer>"#, raw_footer)
    };

    let note_title_html = crate::utils::html_escape(&note_config.note_title);
    let note_description_html = crate::utils::html_escape(&note_config.meta_desc);
    let note_keywords_html = crate::utils::html_escape(&note_config.meta_keywords);
    let final_html = crate::utils::render_once(
        &template,
        &[
            ("{{note_title}}", note_title_html.as_str()),
            ("{{note_description}}", note_description_html.as_str()),
            ("{{note_keywords}}", note_keywords_html.as_str()),
            ("{{notes_html}}", notes_html.as_str()),
            ("{{notes_json}}", notes_json.as_str()),
            ("{{custom_head}}", custom_head.as_str()),
            ("{{footer_html}}", footer_html.as_str()),
        ],
    );

    let bytes = Bytes::from(final_html);
    if !cfg!(debug_assertions) {
        *state.note_list_html_cache.write().await = Some(bytes.clone());
    }

    let mut res = Response::new(axum::body::Body::from(bytes));
    res.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/html; charset=utf-8"),
    );
    if !cfg!(debug_assertions) {
        res.headers_mut().insert(
            header::CACHE_CONTROL,
            HeaderValue::from_static("public, max-age=300"),
        );
    }
    res.headers_mut()
        .insert(header::VARY, HeaderValue::from_static("Accept"));
    res
}

async fn handle_note_detail(
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
    axum::extract::Query(query): axum::extract::Query<NoteQuery>,
    req: axum::extract::Request,
) -> Response {
    reload_index_in_debug(&state).await;

    let wants_markdown = query.format.as_deref() == Some("markdown")
        || req
            .headers()
            .get(header::ACCEPT)
            .and_then(|v| v.to_str().ok())
            .map(|v| v.contains("text/markdown") || v.contains("text/x-markdown"))
            .unwrap_or(false);

    if !wants_markdown {
        let cache = state.note_html_cache.read().await;
        if let Some(cached) = cache.get(&slug) {
            if !cfg!(debug_assertions) {
                let mut res = Response::new(axum::body::Body::from(cached.body.clone()));
                res.headers_mut().insert(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static("text/html; charset=utf-8"),
                );
                res.headers_mut().insert(
                    header::CACHE_CONTROL,
                    HeaderValue::from_static("public, max-age=300"),
                );
                res.headers_mut()
                    .insert(header::VARY, HeaderValue::from_static("Accept"));
                res.headers_mut().insert(
                    header::LINK,
                    HeaderValue::from_static(
                        "<?format=markdown>; rel=\"alternate\"; type=\"text/markdown\"",
                    ),
                );
                return res;
            }
        }
    }

    let filename = {
        let index = state.note_index.read().await;
        index
            .iter()
            .find(|n| n.meta.slug == slug)
            .map(|n| n.filename.clone())
    };

    if let Some(filename) = filename {
        let file_path = format!("notes/{}", filename);
        if let Ok(content) = tokio::fs::read_to_string(&file_path).await {
            if wants_markdown {
                let mut res = Response::new(axum::body::Body::from(content));
                res.headers_mut().insert(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static("text/markdown; charset=utf-8"),
                );
                res.headers_mut().insert(
                    header::CACHE_CONTROL,
                    HeaderValue::from_static("private, no-cache, no-store, must-revalidate"),
                );
                res.headers_mut()
                    .insert(header::VARY, HeaderValue::from_static("Accept"));
                return res;
            }

            if let Some((meta, markdown_body)) = crate::note_loader::parse_note(&content) {
                let mut options = pulldown_cmark::Options::empty();
                options.insert(pulldown_cmark::Options::ENABLE_TABLES);
                options.insert(pulldown_cmark::Options::ENABLE_MATH);
                options.insert(pulldown_cmark::Options::ENABLE_STRIKETHROUGH);
                options.insert(pulldown_cmark::Options::ENABLE_TASKLISTS);
                options.insert(pulldown_cmark::Options::ENABLE_FOOTNOTES);
                options.insert(pulldown_cmark::Options::ENABLE_SMART_PUNCTUATION);
                let parser = Parser::new_ext(&markdown_body, options);
                let mut html_output = String::new();
                html::push_html(&mut html_output, parser);

                let template = tokio::fs::read_to_string("templates/note_detail.html").await.unwrap_or_else(|_| {
                    "<!DOCTYPE html><html><body><article><h1>{{title}}</h1><div class='content'>{{content}}</div></article></body></html>".to_string()
                });

                let display_date = format_date(&meta.date);

                let updated_at_html = if let Some(updated) = &meta.updated_at {
                    let disp = format_date(updated);
                    format!(
                        r#"<span class="note-updated" style="margin-left: 8px;">(最后修改: <time datetime="{}">{}</time>)</span>"#,
                        crate::utils::html_escape(updated),
                        crate::utils::html_escape(&disp)
                    )
                } else {
                    String::new()
                };

                let excerpt_html = if let Some(excerpt) = &meta.excerpt {
                    crate::utils::html_escape(excerpt)
                } else {
                    String::new()
                };

                let keywords = meta.tags.join(", ");
                let keywords_html = crate::utils::html_escape(&keywords);

                let (_, site_config, _) = crate::config::load_site_data();
                let raw_head = site_config
                    .custom_head
                    .as_deref()
                    .unwrap_or_default()
                    .trim();
                let custom_head = raw_head
                    .replace('\n', "\n  ")
                    .replace("{{url_path}}", &format!("note/{}", slug));

                let raw_footer = site_config
                    .footer_html
                    .as_deref()
                    .unwrap_or_default()
                    .trim();
                let footer_html = if raw_footer.is_empty() {
                    "".to_string()
                } else {
                    format!(
                        r#"<footer class="site-footer detail-footer">{}</footer>"#,
                        raw_footer
                    )
                };

                let title_html = crate::utils::html_escape(&meta.title);
                let date_html = crate::utils::html_escape(&display_date);
                let final_html = crate::utils::render_once(
                    &template,
                    &[
                        ("{{title}}", title_html.as_str()),
                        ("{{excerpt}}", excerpt_html.as_str()),
                        ("{{keywords}}", keywords_html.as_str()),
                        ("{{date}}", date_html.as_str()),
                        ("{{updated_at_html}}", updated_at_html.as_str()),
                        ("{{content}}", html_output.as_str()),
                        ("{{custom_head}}", custom_head.as_str()),
                        ("{{footer_html}}", footer_html.as_str()),
                    ],
                );

                let bytes = Bytes::from(final_html);

                if !cfg!(debug_assertions) {
                    let mut cache = state.note_html_cache.write().await;
                    if cache.len() > 20 {
                        // 淘汰一条而非清空全部，避免缓存雪崩
                        if let Some(key) = cache.keys().next().cloned() {
                            cache.remove(&key);
                        }
                    }
                    cache.insert(
                        slug.clone(),
                        crate::state::NoteHtmlCache {
                            body: bytes.clone(),
                        },
                    );
                }

                let mut res = Response::new(axum::body::Body::from(bytes));
                res.headers_mut().insert(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static("text/html; charset=utf-8"),
                );
                if !cfg!(debug_assertions) {
                    res.headers_mut().insert(
                        header::CACHE_CONTROL,
                        HeaderValue::from_static("public, max-age=300"),
                    );
                }
                res.headers_mut()
                    .insert(header::VARY, HeaderValue::from_static("Accept"));
                res.headers_mut().insert(
                    header::LINK,
                    HeaderValue::from_static(
                        "<?format=markdown>; rel=\"alternate\"; type=\"text/markdown\"",
                    ),
                );
                return res;
            }
        }
    }

    StatusCode::NOT_FOUND.into_response()
}
