use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use rand::Rng;
use sha2::{Digest, Sha256};
use tracing::{error, warn};

use crate::middlewares::constant_time_eq;
use crate::model::{AuthSecrets, SecurityConfig};

const SECRETS_FILE: &str = "secrets.toml";
const PLACEHOLDER_PASSWORD: &str = "CHANGE_YOUR_PASSWORD";
const PLACEHOLDER_ANSWERS: &[&str] = &["default1", "default2", "default3"];

#[derive(serde::Deserialize)]
struct SecretsFile {
    #[serde(default)]
    admin_password_hash: Option<String>,
    #[serde(default)]
    admin_security_answer_hashes: Vec<String>,
}

/// 生成 `sha256$<salt_hex>$<hash_hex>` 格式的加盐哈希。
pub fn hash_secret(secret: &str) -> String {
    let salt = random_bytes(16);
    let mut hasher = Sha256::new();
    hasher.update(&salt);
    hasher.update(secret.as_bytes());
    let digest = hasher.finalize();
    format!("sha256${}${}", to_hex(&salt), to_hex(&digest))
}

/// 使用恒定时间比较验证明文 secret 与存储的加盐哈希。
pub fn verify_secret(secret: &str, stored: &str) -> bool {
    let mut parts = stored.split('$');
    let algorithm = parts.next();
    let salt_hex = parts.next();
    let hash_hex = parts.next();
    if algorithm != Some("sha256") || salt_hex.is_none() || hash_hex.is_none() {
        return false;
    }
    let salt_hex = salt_hex.unwrap_or_default();
    let hash_hex = hash_hex.unwrap_or_default();
    let Some(salt) = decode_hex(salt_hex) else {
        return false;
    };

    let mut hasher = Sha256::new();
    hasher.update(&salt);
    hasher.update(secret.as_bytes());
    let candidate = to_hex(&hasher.finalize());
    constant_time_eq(&candidate, hash_hex)
}

/// 按优先级加载认证秘密：
/// 1. `LILY_ADMIN_PASSWORD_HASH` / `LILY_ADMIN_SECURITY_ANSWER_HASHES`
/// 2. `secrets.toml`
/// 3. 从 `config.toml` 的明文密码/密保答案迁移（占位值视为未初始化）
pub fn load_auth_secrets(security_config: &SecurityConfig) -> AuthSecrets {
    // 1. 环境变量优先
    if let Ok(password_hash) = std::env::var("LILY_ADMIN_PASSWORD_HASH")
        && !password_hash.trim().is_empty()
    {
        let answer_hashes = std::env::var("LILY_ADMIN_SECURITY_ANSWER_HASHES")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .map(|value| {
                value
                    .split(',')
                    .map(|item| item.trim().to_string())
                    .filter(|item| !item.is_empty())
                    .collect()
            })
            .unwrap_or_default();
        return AuthSecrets {
            admin_password_hash: Some(password_hash),
            admin_security_answer_hashes: answer_hashes,
        };
    }

    // 2. secrets.toml
    if Path::new(SECRETS_FILE).exists() {
        match fs::read_to_string(SECRETS_FILE) {
            Ok(content) => match toml::from_str::<SecretsFile>(&content) {
                Ok(file) => {
                    return AuthSecrets {
                        admin_password_hash: file
                            .admin_password_hash
                            .filter(|hash| !hash.trim().is_empty()),
                        admin_security_answer_hashes: file.admin_security_answer_hashes,
                    };
                }
                Err(e) => {
                    error!("解析 {} 失败: {}, 回退到 config.toml 明文", SECRETS_FILE, e);
                }
            },
            Err(e) => {
                error!("读取 {} 失败: {}, 回退到 config.toml 明文", SECRETS_FILE, e);
            }
        }
    }

    // 3. config.toml 明文回退（仅用于迁移）
    let mut secrets = AuthSecrets::default();
    let password = security_config.admin_password.as_deref().unwrap_or("");
    if password.is_empty() || password == PLACEHOLDER_PASSWORD {
        secrets.admin_password_hash = None;
    } else {
        secrets.admin_password_hash = Some(hash_secret(password));
        warn!(
            "已从 config.toml 明文 admin_password 生成哈希。请尽快运行 lily-nest set-password 将密码迁移到 secrets.toml"
        );
    }

    let answers = security_config
        .admin_security_answers
        .as_deref()
        .unwrap_or(&[]);
    if answers
        .iter()
        .any(|answer| PLACEHOLDER_ANSWERS.contains(&answer.as_str()))
    {
        secrets.admin_security_answer_hashes = Vec::new();
    } else if !answers.is_empty() {
        secrets.admin_security_answer_hashes =
            answers.iter().map(|answer| hash_secret(answer)).collect();
        warn!(
            "已从 config.toml 明文 admin_security_answers 生成哈希。请尽快迁移密保答案到 secrets.toml"
        );
    }

    secrets
}

/// 原子写入 secrets.toml，Unix 下权限收紧为 0600。
pub fn save_auth_secrets(secrets: &AuthSecrets) -> io::Result<()> {
    let mut content = String::new();
    if let Some(hash) = &secrets.admin_password_hash {
        content.push_str(&format!(
            "admin_password_hash = {}\n",
            toml::Value::String(hash.clone())
        ));
    }
    let answer_hashes = toml::Value::Array(
        secrets
            .admin_security_answer_hashes
            .iter()
            .map(|hash| toml::Value::String(hash.clone()))
            .collect(),
    );
    content.push_str(&format!(
        "admin_security_answer_hashes = {}\n",
        answer_hashes
    ));

    let tmp = format!(
        "{}.{}.{}.tmp",
        SECRETS_FILE,
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    );

    let mut file = fs::File::create(&tmp)?;
    file.write_all(content.as_bytes())?;
    file.flush()?;
    file.sync_all()?;
    drop(file);

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&tmp, fs::Permissions::from_mode(0o600))?;
    }

    if let Err(e) = fs::rename(&tmp, SECRETS_FILE) {
        let _ = fs::remove_file(&tmp);
        return Err(e);
    }
    Ok(())
}

/// 计算字节串的 SHA-256 十六进制摘要（用于 ETag 等非秘密用途）。
pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::Digest;
    let digest = Sha256::digest(bytes);
    to_hex(&digest)
}

fn random_bytes(len: usize) -> Vec<u8> {
    let mut bytes = vec![0u8; len];
    rand::rng().fill_bytes(&mut bytes);
    bytes
}

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{:02x}", byte)).collect()
}

fn decode_hex(hex: &str) -> Option<Vec<u8>> {
    if !hex.len().is_multiple_of(2) {
        return None;
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).ok())
        .collect()
}
