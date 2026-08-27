use crate::config::{
    load_about_items, load_changelog, load_cloudflare_config, load_projects, load_site_data,
};
use crate::utils::{html_escape, sanitize_url};
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
    let (profile_data, site_config, _) = load_site_data();
    let projects_data = load_projects();
    let about_data = load_about_items();
    let cf_data = load_cloudflare_config();

    let html = fs::read_to_string("templates/index.html").unwrap_or_else(|_| {
        "<!doctype html><html><body><h1>templates/index.html not found</h1></body></html>"
            .to_string()
    });

    // 加载片段模板
    let member_tpl = load_fragment("member_button.html");
    let project_tpl = load_fragment("project_item.html");
    let divider_tpl = load_fragment("project_divider.html");
    let about_tpl = load_fragment("about_item.html");

    // 1. 组装成员（N1：单遍渲染，避免值中的占位符被二次替换）
    let members_html = profile_data
        .team_members
        .iter()
        .map(|m| crate::utils::render_once(&member_tpl, &[("{name}", html_escape(m).as_str())]))
        .collect::<String>();

    // 2. 组装项目预览
    let projects_html = projects_data
        .items
        .iter()
        .enumerate()
        .map(|(i, proj)| {
            // 如果不是第一个元素，在前面加一个分割线
            let divider = if i > 0 { divider_tpl.as_str() } else { "" };
            let url = html_escape(sanitize_url(&proj.url));
            let name = html_escape(&proj.name);
            let desc = html_escape(&proj.desc);
            let rendered = crate::utils::render_once(
                &project_tpl,
                &[
                    ("{url}", url.as_str()),
                    ("{name}", name.as_str()),
                    ("{desc}", desc.as_str()),
                ],
            );
            format!("{divider}{rendered}")
        })
        .collect::<String>();

    // 3. 关于我
    let about_items_html = about_data
        .items
        .iter()
        .map(|item| {
            let icon = html_escape(sanitize_url(&item.icon_url));
            let title = html_escape(&item.title);
            let content = html_escape(&item.content);
            crate::utils::render_once(
                &about_tpl,
                &[
                    ("{icon}", icon.as_str()),
                    ("{title}", title.as_str()),
                    ("{content}", content.as_str()),
                ],
            )
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
            let tag_style = if tag_text.trim().is_empty() {
                "display:none"
            } else {
                ""
            };
            let since_text = item.since.as_deref().unwrap_or("");
            let since_style = if since_text.trim().is_empty() {
                "display:none"
            } else {
                ""
            };
            let date = html_escape(&item.date);
            let title = html_escape(&item.title);
            let content = html_escape(&item.content);
            let tag = html_escape(tag_text);
            let since = html_escape(since_text);
            crate::utils::render_once(
                &changelog_tpl,
                &[
                    ("{date}", date.as_str()),
                    ("{title}", title.as_str()),
                    ("{content}", content.as_str()),
                    ("{tag}", tag.as_str()),
                    ("{tag_style}", tag_style),
                    ("{since}", since.as_str()),
                    ("{since_style}", since_style),
                ],
            )
        })
        .collect::<String>();

    // 计算全部插入值：用户可配置文本先做 HTML 转义，raw HTML 片段原样保留
    let profile_title = html_escape(&profile_data.current_identity);
    let index_title = html_escape(&site_config.index_title);
    let meta_desc = html_escape(&site_config.meta_desc);
    let avatar = html_escape(sanitize_url(&profile_data.avatar_url));
    let bg = html_escape(sanitize_url(&profile_data.bg_url));
    let ver = html_escape(&profile_data.site_version);
    let intro = html_escape(&profile_data.intro);
    let note_url_escaped = html_escape(sanitize_url(&profile_data.note_url));
    let note_disabled = if profile_data.note_enable {
        ""
    } else {
        "disabled"
    };

    // 注入 Cloudflare Web Analytics 脚本
    let script = if let Some(ref token) = cf_data.web_analytics_token {
        let clean_token = token.trim();
        if clean_token.is_empty()
            || !clean_token
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            "".to_string()
        } else {
            CF_BEACON_TEMPLATE.replace("__CF_BEACON_TOKEN__", clean_token)
        }
    } else {
        "".to_string()
    };

    let raw_head = site_config
        .custom_head
        .as_deref()
        .unwrap_or_default()
        .trim();
    let custom_head =
        crate::utils::render_once(&raw_head.replace('\n', "\n    "), &[("{{url_path}}", "")]);

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

    crate::utils::render_once(
        &html,
        &[
            ("{{profile_title}}", profile_title.as_str()),
            ("{{index_title}}", index_title.as_str()),
            ("{{meta_desc}}", meta_desc.as_str()),
            ("{{avatar}}", avatar.as_str()),
            ("{{bg}}", bg.as_str()),
            ("{{ver}}", ver.as_str()),
            ("{{members_html}}", members_html.as_str()),
            ("{{intro}}", intro.as_str()),
            ("{{note_url}}", note_url_escaped.as_str()),
            ("{{note_disabled}}", note_disabled),
            ("{{projects_html}}", projects_html.as_str()),
            ("{{about_items_html}}", about_items_html.as_str()),
            ("{{changelog_html}}", changelog_html.as_str()),
            ("{{web_analytics_script}}", script.as_str()),
            ("{{custom_head}}", custom_head.as_str()),
            ("{{footer_html}}", footer_html.as_str()),
        ],
    )
}

pub fn render_index_markdown() -> String {
    let (profile_data, site_config, _) = load_site_data();
    let projects_data = load_projects();
    let about_data = load_about_items();
    let changelog_data = load_changelog();

    let mut md = String::with_capacity(4096);

    // 1. 标题与简介
    md.push_str(&format!("# {}\n\n", site_config.index_title));
    md.push_str(&format!("> {}\n>\n", profile_data.intro));

    let members = profile_data.team_members.join(", ");
    md.push_str(&format!(
        "> **当前版本**：{} | **团队成员**：{}\n\n",
        profile_data.site_version, members
    ));
    md.push_str("---\n\n");

    // 2. 项目概览
    md.push_str("## 💻 项目概览\n\n");
    for proj in &projects_data.items {
        md.push_str(&format!(
            "*   **[{}]({})**：{}\n",
            proj.name, proj.url, proj.desc
        ));
    }
    md.push_str("\n---\n\n");

    // 3. 关于我
    md.push_str("## 🛠️ 关于我\n\n");
    for item in &about_data.items {
        md.push_str(&format!("*   **{}**：{}\n", item.title, item.content));
    }
    md.push_str("\n---\n\n");

    // 4. 更新日志
    md.push_str("## ⏱️ 更新日志\n\n");
    for item in &changelog_data.items {
        let tag = item.tag.as_deref().unwrap_or("");
        let since = item.since.as_deref().unwrap_or("");
        let meta = match (tag.is_empty(), since.is_empty()) {
            (false, false) => format!(" `[{}]` (since `{}`)", tag, since),
            (false, true) => format!(" `[{}]`", tag),
            (true, false) => format!(" (since `{}`)", since),
            (true, true) => String::new(),
        };
        md.push_str(&format!("### {} — {}{}\n", item.date, item.title, meta));
        md.push_str(&format!("{}\n\n", item.content));
    }

    md
}
