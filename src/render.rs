use crate::config::{load_about_items, load_changelog, load_projects, load_site_data, load_cloudflare_config};
use std::fs;
use tracing::error;

const CF_BEACON_TEMPLATE: &str = r#"<script defer src="https://static.cloudflareinsights.com/beacon.min.js" data-cf-beacon='{"token": "__CF_BEACON_TOKEN__"}'></script>"#;

/// 从 templates/fragments/ 读取片段文件，找不到则返回空字符串并记录错误
fn load_fragment(name: &str) -> String {
    let path = format!("templates/fragments/{}", name);
    fs::read_to_string(&path).unwrap_or_else(|e| {
        error!("无法加载片段模板 {}: {}", path, e);
        String::new()
    })
}

pub fn render_index() -> String {
    let (profile_data, site_config) = load_site_data();
    let projects_data = load_projects();
    let about_data = load_about_items();
    let cf_data = load_cloudflare_config();

    let mut html = fs::read_to_string("templates/index.html").unwrap_or_else(|_| {
        "<!doctype html><html><body><h1>templates/index.html not found</h1></body></html>"
            .to_string()
    });

    // 加载片段模板
    let member_tpl = load_fragment("member_button.html");
    let project_tpl = load_fragment("project_item.html");
    let divider_tpl = load_fragment("project_divider.html");
    let about_tpl = load_fragment("about_item.html");

    // 1. 组装成员
    let members_html = profile_data
        .team_members
        .iter()
        .map(|m| member_tpl.replace("{name}", &html_escape(m)))
        .collect::<String>();

    // 2. 组装项目预览
    let projects_html = projects_data
        .items
        .iter()
        .enumerate()
        .map(|(i, proj)| {
            // 如果不是第一个元素，在前面加一个分割线
            let divider = if i > 0 { divider_tpl.as_str() } else { "" };
            let rendered = project_tpl
                .replace("{url}", &html_escape(sanitize_url(&proj.url)))
                .replace("{name}", &html_escape(&proj.name))
                .replace("{desc}", &html_escape(&proj.desc));
            format!("{divider}{rendered}")
        })
        .collect::<String>();

    // 3. 关于我
    let about_items_html = about_data
        .items
        .iter()
        .map(|item| {
            about_tpl
                .replace("{icon}", &html_escape(sanitize_url(&item.icon_url)))
                .replace("{title}", &html_escape(&item.title))
                .replace("{content}", &html_escape(&item.content))
        })
        .collect::<String>();

    // 4. 更新日志（最多展示 10 条）
    let changelog_tpl = load_fragment("changelog_item.html");
    let changelog_data = load_changelog();
    let changelog_html = changelog_data
        .items
        .iter()
        .take(10)
        .map(|item| {
            let tag_text = item.tag.as_deref().unwrap_or("");
            let tag_style = if tag_text.trim().is_empty() { "display:none" } else { "" };
            let since_text = item.since.as_deref().unwrap_or("");
            let since_style = if since_text.trim().is_empty() { "display:none" } else { "" };
            changelog_tpl
                .replace("{date}", &html_escape(&item.date))
                .replace("{title}", &html_escape(&item.title))
                .replace("{content}", &html_escape(&item.content))
                .replace("{tag}", &html_escape(tag_text))
                .replace("{tag_style}", tag_style)
                .replace("{since}", &html_escape(since_text))
                .replace("{since_style}", since_style)
        })
        .collect::<String>();

    // 替换占位符
    html = html.replace("{{profile_title}}", &html_escape(&profile_data.current_identity));
    html = html.replace("{{index_title}}", &html_escape(&site_config.index_title));
    html = html.replace("{{meta_desc}}", &html_escape(&site_config.meta_desc));
    html = html.replace(
        "{{avatar}}",
        &html_escape(sanitize_url(&profile_data.avatar_url)),
    );
    html = html.replace("{{bg}}", &html_escape(sanitize_url(&profile_data.bg_url)));
    html = html.replace("{{ver}}", &html_escape(&profile_data.site_version));
    html = html.replace("{{members_html}}", &members_html);
    html = html.replace("{{intro}}", &html_escape(&profile_data.intro));
    let blog_url_escaped = html_escape(sanitize_url(&profile_data.blog_url));
    let blog_disabled = if profile_data.blog_enable { "" } else { "disabled" };
    html = html.replace("{{blog_url}}", &blog_url_escaped);
    html = html.replace("{{blog_disabled}}", blog_disabled);
    // 注入项目 HTML
    html = html.replace("{{projects_html}}", &projects_html);
    html = html.replace("{{about_items_html}}", &about_items_html);
    html = html.replace("{{changelog_html}}", &changelog_html);

    // 注入 Cloudflare Web Analytics 脚本
    let script = if let Some(ref token) = cf_data.web_analytics_token {
        let clean_token = token.trim();
        if clean_token.is_empty() || !clean_token.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            "".to_string()
        } else {
            CF_BEACON_TEMPLATE.replace("__CF_BEACON_TOKEN__", clean_token)
        }
    } else {
        "".to_string()
    };
    html = html.replace("{{web_analytics_script}}", &script);

    html
}

pub fn sanitize_url(url: &str) -> &str {
    let url = url.trim();
    if (url.starts_with("http://") || url.starts_with("https://"))
        || (url.starts_with('/') && !url.starts_with("//"))
    {
        url
    } else {
        "#projects"
    }
}

// 简单转义：用于插入到 HTML 文本节点/属性里
pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
