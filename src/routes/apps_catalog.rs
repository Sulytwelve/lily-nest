use std::sync::Arc;

use axum::{
    Router,
    extract::{Query, State},
    http::{HeaderValue, StatusCode, header},
    response::{Html, IntoResponse, Response},
    routing::get,
};
use serde::Deserialize;

use crate::{
    model::{HtmlAppCategory, HtmlAppMeta},
    state::AppState,
};

#[derive(Debug, Default, Deserialize)]
struct AppsQuery {
    format: Option<String>,
    category: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CategoryFilter {
    All,
    Tools,
    Games,
}

impl CategoryFilter {
    fn parse(value: Option<&str>) -> Result<Self, ()> {
        match value {
            None | Some("") | Some("all") => Ok(Self::All),
            Some("tools") => Ok(Self::Tools),
            Some("games") => Ok(Self::Games),
            Some(_) => Err(()),
        }
    }

    fn includes(self, category: HtmlAppCategory) -> bool {
        matches!(self, Self::All)
            || matches!(
                (self, category),
                (Self::Tools, HtmlAppCategory::Tools) | (Self::Games, HtmlAppCategory::Games)
            )
    }

    fn query_value(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Tools => "tools",
            Self::Games => "games",
        }
    }
}

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/apps", get(handler_apps))
        .with_state(state)
}

async fn handler_apps(
    State(state): State<Arc<AppState>>,
    Query(query): Query<AppsQuery>,
    request: axum::extract::Request,
) -> Response {
    let filter = match CategoryFilter::parse(query.category.as_deref()) {
        Ok(filter) => filter,
        Err(()) => return (StatusCode::BAD_REQUEST, "unknown app category").into_response(),
    };
    let apps = state.html_apps.read().await.clone();
    let site_config = tokio::task::spawn_blocking(|| crate::config::load_site_data().1)
        .await
        .unwrap_or_else(|error| {
            tracing::error!("load_site_data panicked while rendering /apps: {error}");
            crate::model::SiteConfig::default()
        });
    let wants_markdown = query.format.as_deref() == Some("markdown")
        || request
            .headers()
            .get(header::ACCEPT)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| {
                value.contains("text/markdown") || value.contains("text/x-markdown")
            });

    let mut response = if wants_markdown {
        markdown_response(render_markdown(&apps, filter))
    } else {
        let template = tokio::fs::read_to_string("templates/apps.html")
            .await
            .unwrap_or_else(|error| {
                tracing::error!("failed to read templates/apps.html: {error}");
                include_str!("../../templates/apps.html").to_string()
            });
        Html(render_html(&template, &apps, filter, &site_config)).into_response()
    };

    let headers = response.headers_mut();
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, no-cache, no-store, must-revalidate"),
    );
    headers.insert(header::VARY, HeaderValue::from_static("Accept"));
    if !wants_markdown {
        let alternate = format!(
            "</apps?format=markdown&category={}>; rel=\"alternate\"; type=\"text/markdown\"",
            filter.query_value()
        );
        if let Ok(value) = HeaderValue::from_str(&alternate) {
            headers.insert(header::LINK, value);
        }
    }
    response
}

fn render_html(
    template: &str,
    apps: &[HtmlAppMeta],
    filter: CategoryFilter,
    site_config: &crate::model::SiteConfig,
) -> String {
    let visible_count = apps
        .iter()
        .filter(|app| filter.includes(app.category))
        .count();
    let filter_links = render_filter_links(filter);
    let app_sections = render_sections(apps, filter);
    let app_count = visible_count.to_string();
    let markdown_url = match filter {
        CategoryFilter::All => "/apps?format=markdown".to_string(),
        _ => format!(
            "/apps?format=markdown&amp;category={}",
            filter.query_value()
        ),
    };
    let raw_head = site_config
        .custom_head
        .as_deref()
        .unwrap_or_default()
        .trim();
    let custom_head = crate::utils::render_once(
        &raw_head.replace('\n', "\n    "),
        &[("{{url_path}}", "apps")],
    );
    let raw_footer = site_config
        .footer_html
        .as_deref()
        .unwrap_or_default()
        .trim();
    let footer_html = if raw_footer.is_empty() {
        String::new()
    } else {
        format!(r#"<footer class="site-footer">{raw_footer}</footer>"#)
    };
    crate::utils::render_once(
        template,
        &[
            ("{{filter_links}}", filter_links.as_str()),
            ("{{app_sections}}", app_sections.as_str()),
            ("{{app_count}}", app_count.as_str()),
            ("{{markdown_url}}", markdown_url.as_str()),
            ("{{custom_head}}", custom_head.as_str()),
            ("{{footer_html}}", footer_html.as_str()),
        ],
    )
}

fn render_filter_links(active: CategoryFilter) -> String {
    [
        (CategoryFilter::All, "全部"),
        (CategoryFilter::Tools, "工具"),
        (CategoryFilter::Games, "游戏"),
    ]
    .into_iter()
    .map(|(filter, label)| {
        let current = if filter == active {
            " aria-current=\"page\""
        } else {
            ""
        };
        format!(
            "<a class=\"filter-chip\" href=\"/apps?category={}\"{}>{}</a>",
            filter.query_value(),
            current,
            label
        )
    })
    .collect::<Vec<_>>()
    .join("")
}

fn render_sections(apps: &[HtmlAppMeta], filter: CategoryFilter) -> String {
    let mut sections = String::new();
    for (category, heading, description) in [
        (
            HtmlAppCategory::Tools,
            "工具",
            "一些专注、轻量、打开即用的小工具。",
        ),
        (
            HtmlAppCategory::Games,
            "游戏",
            "无需安装，直接在浏览器里玩的小游戏。",
        ),
    ] {
        if !filter.includes(category) {
            continue;
        }
        let category_apps: Vec<_> = apps.iter().filter(|app| app.category == category).collect();
        sections.push_str(&format!(
            "<section class=\"app-section\" aria-labelledby=\"{}-heading\"><div class=\"section-heading\"><div><p class=\"eyebrow\">{}</p><h2 id=\"{}-heading\">{}</h2></div><p>{}</p></div>",
            category.as_str(),
            category.as_str(),
            category.as_str(),
            heading,
            description
        ));
        if category_apps.is_empty() {
            sections.push_str("<p class=\"empty-state\">这个分类暂时还没有应用。</p>");
        } else {
            sections.push_str("<div class=\"app-grid\">");
            for app in category_apps {
                let title = escape_html(&app.title);
                let description = escape_html(&app.description);
                let url = escape_html(&app.url);
                let description = if description.trim().is_empty() {
                    "打开这个应用。".to_string()
                } else {
                    description
                };
                sections.push_str(&format!(
                    "<article class=\"app-card\"><div class=\"card-copy\"><span class=\"category-badge\">{}</span><h3>{}</h3><p>{}</p></div><a class=\"open-app\" href=\"{}\"><span>打开应用</span><span aria-hidden=\"true\">↗</span></a></article>",
                    heading, title, description, url
                ));
            }
            sections.push_str("</div>");
        }
        sections.push_str("</section>");
    }
    sections
}

fn render_markdown(apps: &[HtmlAppMeta], filter: CategoryFilter) -> String {
    let mut output = String::from("# Apps\n\n这里收录了本站可直接使用的工具和游戏。\n");
    for (category, heading) in [
        (HtmlAppCategory::Tools, "工具"),
        (HtmlAppCategory::Games, "游戏"),
    ] {
        if !filter.includes(category) {
            continue;
        }
        output.push_str(&format!("\n## {heading}\n\n"));
        let mut found = false;
        for app in apps.iter().filter(|app| app.category == category) {
            found = true;
            output.push_str(&format!("- [{}]({})", escape_markdown(&app.title), app.url));
            if !app.description.trim().is_empty() {
                output.push_str(&format!(" — {}", escape_markdown(&app.description)));
            }
            output.push('\n');
        }
        if !found {
            output.push_str("这个分类暂时还没有应用。\n");
        }
    }
    output
}

fn markdown_response(body: String) -> Response {
    let mut response = Response::new(axum::body::Body::from(body));
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/markdown; charset=utf-8"),
    );
    response
}

fn escape_html(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(character),
        }
    }
    escaped
}

fn escape_markdown(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '\r' | '\n' => escaped.push(' '),
            '\\' | '[' | ']' | '(' | ')' | '*' | '_' | '`' | '#' | '!' | '<' | '>' => {
                escaped.push('\\');
                escaped.push(character);
            }
            _ => escaped.push(character),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app(category: HtmlAppCategory, title: &str, description: &str) -> HtmlAppMeta {
        HtmlAppMeta {
            category,
            slug: "demo".into(),
            title: title.into(),
            description: description.into(),
            filename: None,
            created_at: String::new(),
            updated_at: String::new(),
            url: format!("/{}/demo", category.as_str()),
        }
    }

    #[test]
    fn html_escapes_manifest_metadata() {
        let apps = [app(
            HtmlAppCategory::Tools,
            "<script>alert(1)</script>",
            "\" onmouseover=\"bad",
        )];
        let rendered = render_sections(&apps, CategoryFilter::All);
        assert!(!rendered.contains("<script>"));
        assert!(!rendered.contains("\" onmouseover=\"bad"));
        assert!(rendered.contains("&lt;script&gt;"));
        assert!(rendered.contains("&quot; onmouseover=&quot;bad"));
    }

    #[test]
    fn html_reuses_site_head_and_footer_without_rescanning_metadata() {
        let mut site = crate::model::SiteConfig::default();
        site.custom_head =
            Some(r#"<link rel="canonical" href="https://example.com/{{url_path}}">"#.into());
        site.footer_html = Some("<span>Site footer</span>".into());
        let apps = [app(
            HtmlAppCategory::Tools,
            "{{footer_html}}",
            "{{custom_head}}",
        )];
        let rendered = render_html(
            "<head>{{custom_head}}</head>{{app_sections}}{{footer_html}}",
            &apps,
            CategoryFilter::All,
            &site,
        );
        assert!(rendered.contains("https://example.com/apps"));
        assert!(
            rendered.contains("<footer class=\"site-footer\"><span>Site footer</span></footer>")
        );
        assert!(rendered.contains("{{footer_html}}"));
        assert!(rendered.contains("{{custom_head}}"));
    }

    #[test]
    fn filters_categories_in_both_formats() {
        let apps = [
            app(HtmlAppCategory::Tools, "Converter", "Convert things"),
            app(HtmlAppCategory::Games, "2048", "Play it"),
        ];
        let html = render_sections(&apps, CategoryFilter::Tools);
        assert!(html.contains("Converter"));
        assert!(!html.contains("2048"));
        let markdown = render_markdown(&apps, CategoryFilter::Games);
        assert!(!markdown.contains("Converter"));
        assert!(markdown.contains("2048"));
    }

    #[test]
    fn rejects_unknown_category() {
        assert_eq!(
            CategoryFilter::parse(Some("tools")),
            Ok(CategoryFilter::Tools)
        );
        assert!(CategoryFilter::parse(Some("notes")).is_err());
    }
}
