use std::fs;
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
        
        if let Ok(meta) = toml::from_str::<NoteFrontmatter>(frontmatter_str) {
            return Some((meta, markdown_content));
        }
    }
    None
}

pub fn load_all_notes() -> Vec<NoteSummary> {
    let mut notes = Vec::new();
    let dir = match fs::read_dir("notes") {
        Ok(d) => d,
        Err(_) => return notes,
    };

    for entry in dir.flatten() {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Some((meta, _)) = parse_note(&content) {
                    if let Some(os_name) = path.file_name() {
                        let filename = os_name.to_string_lossy().to_string();
                        notes.push(NoteSummary { meta, filename });
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
