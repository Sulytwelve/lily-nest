use crate::config::{load_about_items, load_projects, load_site_data};
use std::fs;

pub fn render_index() -> String {
    let (profile_data, site_config) = load_site_data();
    let projects_data = load_projects();
    let about_data = load_about_items();

    let mut html = fs::read_to_string("templates/index.html").unwrap_or_else(|_| {
        "<!doctype html><html><body><h1>templates/index.html not found</h1></body></html>"
            .to_string()
    });
    // 1. 组装成员
    let members_html = profile_data
        .team_members
        .iter()
        .map(|m| format!(r#"<md-text-button>{}</md-text-button>"#, html_escape(m)))
        .collect::<String>();

    // 2. 组装项目预览
    let projects_html = projects_data
        .items
        .iter()
        .enumerate()
        .map(|(i, proj)| {
            // 如果不是第一个元素，在前面加一个分割线
            let divider = if i > 0 {
                "<md-divider></md-divider>"
            } else {
                ""
            };

            format!(
                r#"{divider}
                  <md-list-item type="button" href="{url}" target="_blank" rel="noopener">
                    <md-icon slot="start">
                      <svg style="height: 48px; width: 48px" viewBox="0 -960 960 960">
                        <path
                          d="M320-240 80-480l240-240 57 57-184 184 183 183-56 56Zm320 0-57-57 184-184-183-183 56-56 240 240-240 240Z"
                        />
                      </svg>
                    </md-icon>
                    <div slot="headline">{name}</div>
                    <div slot="supporting-text">{desc}</div>
                    <md-icon slot="end">
                      <svg style="height: 48px; width: 48px" viewBox="0 -960 960 960">
                        <path
                          d="M200-120q-33 0-56.5-23.5T120-200v-560q0-33 23.5-56.5T200-840h280v80H200v560h560v-280h80v280q0 33-23.5 56.5T760-120H200Zm188-212-56-56 372-372H560v-80h280v280h-80v-144L388-332Z"
                        />
                      </svg>
                    </md-icon>
                  </md-list-item>
                "#,
                divider = divider,
                url = html_escape(sanitize_url(&proj.url)),
                name = html_escape(&proj.name),
                desc = html_escape(&proj.desc)
            )
        })
        .collect::<String>();

    // 3. 关于我
    let about_items_html = about_data
        .items
        .iter()
        .map(|item| {
            format!(
                r#"
        <md-list-item>
            <img slot="start" src="{icon}" style="width: 24px; height: 24px; border-radius: 50%;" alt="{title}">
            <div slot="headline">{title}</div>
            <div slot="supporting-text">{content}</div>
        </md-list-item>
        "#,
                icon = html_escape(sanitize_url(&item.icon_url)),
                title = html_escape(&item.title),
                content = html_escape(&item.content)
            )
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
    // 注入项目 HTML
    html = html.replace("{{projects_html}}", &projects_html);
    html = html.replace("{{about_items_html}}", &about_items_html);

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
