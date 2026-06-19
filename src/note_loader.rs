
use crate::model::{NoteFrontmatter, NoteSummary};

pub fn parse_note(content: &str) -> Option<(NoteFrontmatter, String)> {
    if !content.starts_with("---") {
        return None;
    }
    
    // Split by "---", limiting to 3 parts: ("", "frontmatter", "content")
    let parts: Vec<&str> = content.splitn(3, "---").collect();
    if parts.len() >= 3 {
        let frontmatter_str = parts[1];
        let markdown_content = parts[2].trim_start().to_string();
        
        match toml::from_str::<NoteFrontmatter>(frontmatter_str) {
            Ok(mut meta) => {
                if meta.excerpt.is_none() {
                    let plain_text = markdown_content
                        .replace(['#', '*', '`', '>', '[', ']', '\n'], " ");
                    
                    let plain_text = plain_text.split_whitespace().collect::<Vec<_>>().join(" ");
                    let excerpt = if plain_text.chars().count() > 100 {
                        plain_text.chars().take(100).collect::<String>() + "..."
                    } else {
                        plain_text
                    };
                    meta.excerpt = Some(excerpt);
                }
                return Some((meta, markdown_content));
            }
            Err(e) => {
                tracing::warn!("笔记元数据解析失败: {}", e);
            }
        }
    }
    None
}

pub async fn load_all_notes() -> Vec<NoteSummary> {
    let mut notes = Vec::new();
    let mut dir = match tokio::fs::read_dir("notes").await {
        Ok(d) => d,
        Err(_) => return notes,
    };

    while let Ok(Some(entry)) = dir.next_entry().await {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
            if let Ok(content) = tokio::fs::read_to_string(&path).await {
                if let Some((meta, markdown_body)) = parse_note(&content) {
                    if let Some(os_name) = path.file_name() {
                        let filename = os_name.to_string_lossy().to_string();
                        notes.push(NoteSummary { meta, filename, content: markdown_body });
                    } else {
                        tracing::warn!("无法解析文件名跳过加载: {:?}", path);
                    }
                }
            }
        }
    }
    
    notes.sort_by(|a, b| b.meta.date.cmp(&a.meta.date));
    notes
}
