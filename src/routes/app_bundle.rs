//! Immutable app revisions: publish the manifest only after every file is durable.
use super::html_apps::{APPS_ROOT, atomic_write, validate_html};
use crate::model::{HtmlAppBundle, HtmlAppCategory, HtmlAppFile};
use base64::{Engine, engine::general_purpose::STANDARD};
use rand::Rng;
use std::{collections::BTreeMap, path::PathBuf};

pub(super) const MAX_BUNDLE_BYTES: usize = 8 * 1024 * 1024;
pub(super) const MAX_FILES: usize = 64;

pub(super) fn mime(path: &str) -> Option<&'static str> {
    Some(
        match path.rsplit('.').next()?.to_ascii_lowercase().as_str() {
            "html" | "htm" => "text/html; charset=utf-8",
            "css" => "text/css; charset=utf-8",
            "js" | "mjs" => "text/javascript; charset=utf-8",
            "json" => "application/json",
            "txt" => "text/plain; charset=utf-8",
            "svg" => "image/svg+xml",
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "webp" => "image/webp",
            "ico" => "image/x-icon",
            "woff" => "font/woff",
            "woff2" => "font/woff2",
            "mp3" => "audio/mpeg",
            "wav" => "audio/wav",
            "ogg" => "audio/ogg",
            "mp4" => "video/mp4",
            "webm" => "video/webm",
            _ => return None,
        },
    )
}

pub(super) fn valid_path(path: &str) -> bool {
    if path.is_empty() || path.len() > 240 || path.split('/').count() > 8 || mime(path).is_none() {
        return false;
    }
    path.split('/').all(|part| {
        let stem = part
            .split('.')
            .next()
            .unwrap_or_default()
            .to_ascii_lowercase();
        let reserved = matches!(stem.as_str(), "con" | "prn" | "aux" | "nul")
            || (stem.len() == 4
                && (stem.starts_with("com") || stem.starts_with("lpt"))
                && matches!(stem.as_bytes()[3], b'1'..=b'9'));
        !part.is_empty()
            && !part.starts_with('.')
            && !part.ends_with('.')
            && !reserved
            && part
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'_' | b'-' | b'.'))
    })
}

pub(super) fn decode(files: Vec<HtmlAppFile>) -> Result<BTreeMap<String, Vec<u8>>, &'static str> {
    if files.is_empty() || files.len() > MAX_FILES {
        return Err("bundle must contain 1 to 64 files");
    }
    let mut result = BTreeMap::new();
    let mut folded = std::collections::HashSet::new();
    let mut total = 0usize;
    for file in files {
        if !valid_path(&file.path) || !folded.insert(file.path.to_ascii_lowercase()) {
            return Err("invalid or duplicate bundle path");
        }
        let bytes = match file.encoding.as_str() {
            "text" => file.content.into_bytes(),
            "base64" => STANDARD
                .decode(file.content)
                .map_err(|_| "invalid base64")?,
            _ => return Err("encoding must be text or base64"),
        };
        total += bytes.len();
        if bytes.len() > 2 * 1024 * 1024 || total > MAX_BUNDLE_BYTES {
            return Err("bundle exceeds 8 MiB or a file exceeds 2 MiB");
        }
        if mime(&file.path).is_some_and(|m| m.starts_with("text/html")) {
            validate_html(std::str::from_utf8(&bytes).map_err(|_| "HTML must be UTF-8")?)?;
        }
        result.insert(file.path, bytes);
    }
    if !result.contains_key("index.html") {
        return Err("bundle requires root index.html");
    }
    // A path cannot simultaneously name a file and a directory (on any OS).
    for path in result.keys() {
        for (index, _) in path.match_indices('/') {
            if folded.contains(&path[..index].to_ascii_lowercase()) {
                return Err("bundle has a file/directory conflict");
            }
        }
    }
    Ok(result)
}

pub(super) fn directory(
    category: HtmlAppCategory,
    slug: &str,
    bundle: &HtmlAppBundle,
) -> Option<PathBuf> {
    if bundle.revision.len() != 32 || !bundle.revision.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    Some(
        PathBuf::from(APPS_ROOT)
            .join(category.as_str())
            .join(slug)
            .join(&bundle.revision),
    )
}

pub(super) async fn stage(
    category: HtmlAppCategory,
    slug: &str,
    files: BTreeMap<String, Vec<u8>>,
) -> std::io::Result<HtmlAppBundle> {
    let mut random = [0u8; 16];
    rand::rng().fill_bytes(&mut random);
    let bundle = HtmlAppBundle {
        revision: random.iter().map(|b| format!("{b:02x}")).collect(),
        files: files.keys().cloned().collect(),
    };
    let root = directory(category, slug, &bundle).unwrap();
    // Never reuse a revision or follow a preexisting staging directory.
    tokio::fs::create_dir_all(root.parent().unwrap()).await?;
    tokio::fs::create_dir(&root).await?;
    for (path, bytes) in files {
        if let Err(error) = atomic_write(&root.join(path), &bytes).await {
            let _ = tokio::fs::remove_dir_all(&root).await;
            return Err(error);
        }
    }
    Ok(bundle)
}

pub(super) async fn read(
    category: HtmlAppCategory,
    slug: &str,
    bundle: &HtmlAppBundle,
    path: &str,
) -> Option<Vec<u8>> {
    if !valid_path(path) || !bundle.files.iter().any(|p| p == path) {
        return None;
    }
    let root = tokio::fs::canonicalize(directory(category, slug, bundle)?)
        .await
        .ok()?;
    let file = tokio::fs::canonicalize(root.join(path)).await.ok()?;
    if !file.starts_with(&root) {
        return None;
    }
    tokio::fs::read(file).await.ok()
}

pub(super) async fn remove(
    category: HtmlAppCategory,
    slug: &str,
    bundle: &HtmlAppBundle,
) -> std::io::Result<()> {
    let path = directory(category, slug, bundle)
        .ok_or_else(|| std::io::Error::other("invalid revision"))?;
    match tokio::fs::remove_dir_all(path).await {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        result => result,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn file(path: &str, content: &str) -> HtmlAppFile {
        HtmlAppFile {
            path: path.into(),
            content: content.into(),
            encoding: "text".into(),
        }
    }
    #[test]
    fn rejects_traversal_reserved_paths_and_collisions() {
        for path in [
            "../a.js",
            "/a.js",
            "a\\b.js",
            "a//b.js",
            ".agent.key",
            "a/NUL.js",
            "a.js.",
            "a%2f.js",
            "a:evil.js",
        ] {
            assert!(!valid_path(path), "{path}");
        }
        assert!(valid_path("assets/my-file.min.js"));
        assert!(
            decode(vec![
                file("index.html", "ok"),
                file("A.js", "x"),
                file("a.js", "y")
            ])
            .is_err()
        );
        assert!(
            decode(vec![
                file("index.html", "ok"),
                file("a.js", "x"),
                file("a.js/b.js", "y")
            ])
            .is_err()
        );
        assert!(decode(vec![file("a.js", "x")]).is_err());
    }
    #[test]
    fn validates_binary_limits_and_html() {
        let mut png = file("assets/a.png", "AAEC");
        png.encoding = "base64".into();
        let files = decode(vec![file("index.html", "ok"), png]).unwrap();
        assert_eq!(files["assets/a.png"], [0, 1, 2]);
        assert!(decode(vec![file("index.html", "")]).is_err());
        assert!(decode(vec![file("index.html", &"x".repeat(2 * 1024 * 1024 + 1))]).is_err());
        let mut bad = file("a.png", "?!");
        bad.encoding = "base64".into();
        assert!(decode(vec![file("index.html", "ok"), bad]).is_err());
    }
    #[tokio::test]
    async fn revisions_are_complete_isolated_and_replaceable() {
        let slug = format!(
            "bundle-test-{}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap()
        );
        let first = stage(
            HtmlAppCategory::Tools,
            &slug,
            decode(vec![
                file("index.html", "old"),
                file("assets/a.js", "script"),
            ])
            .unwrap(),
        )
        .await
        .unwrap();
        let second = stage(
            HtmlAppCategory::Tools,
            &slug,
            decode(vec![file("index.html", "new")]).unwrap(),
        )
        .await
        .unwrap();
        assert_ne!(first.revision, second.revision);
        assert_eq!(
            read(HtmlAppCategory::Tools, &slug, &first, "index.html")
                .await
                .unwrap(),
            b"old"
        );
        assert_eq!(
            read(HtmlAppCategory::Tools, &slug, &second, "index.html")
                .await
                .unwrap(),
            b"new"
        );
        assert!(
            read(HtmlAppCategory::Tools, &slug, &second, "assets/a.js")
                .await
                .is_none()
        );
        assert!(
            read(HtmlAppCategory::Tools, &slug, &first, "../manifest.toml")
                .await
                .is_none()
        );
        remove(HtmlAppCategory::Tools, &slug, &first).await.unwrap();
        remove(HtmlAppCategory::Tools, &slug, &second)
            .await
            .unwrap();
        tokio::fs::remove_dir(PathBuf::from(APPS_ROOT).join("tools").join(slug))
            .await
            .unwrap();
    }
}
