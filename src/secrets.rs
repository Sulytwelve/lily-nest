use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use rand::Rng;
use sha2::{Digest, Sha256};
use tracing::{error, warn};

use crate::middlewares::constant_time_eq;
use crate::model::{AuthSecrets, SecurityConfig};

const DEFAULT_SECRETS_FILE: &str = "secrets.toml";
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
    load_auth_secrets_with(security_config, Path::new(DEFAULT_SECRETS_FILE))
}

/// 与 [`load_auth_secrets`] 行为一致，但允许指定 secrets 文件路径（便于测试）。
pub fn load_auth_secrets_with(
    security_config: &SecurityConfig,
    secrets_path: &Path,
) -> AuthSecrets {
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
    if secrets_path.exists() {
        match fs::read_to_string(secrets_path) {
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
                    error!(
                        "解析 {} 失败: {}, 回退到 config.toml 明文",
                        secrets_path.display(),
                        e
                    );
                }
            },
            Err(e) => {
                error!(
                    "读取 {} 失败: {}, 回退到 config.toml 明文",
                    secrets_path.display(),
                    e
                );
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

/// 仅从 `secrets.toml` 读取（不走 `config.toml` 明文回退）。
///
/// 用途：CLI 子命令（如 `set-security-answers`）写回时，
/// 避免无意把 `config.toml` 里尚未迁移的明文密码提升进 `secrets.toml`。
/// 与 `load_auth_secrets` 的区别：不读 `LILY_ADMIN_PASSWORD_HASH` 等环境变量，
/// 也不读 `config.toml` 明文，缺失或解析失败时回退到 `AuthSecrets::default()`。
pub fn load_secrets_file_only() -> AuthSecrets {
    load_secrets_file(Path::new(DEFAULT_SECRETS_FILE))
}

/// 与 [`load_secrets_file_only`] 行为一致，但允许指定路径（便于测试）。
pub fn load_secrets_file(secrets_path: &Path) -> AuthSecrets {
    if !secrets_path.exists() {
        return AuthSecrets::default();
    }
    match fs::read_to_string(secrets_path) {
        Ok(content) => match toml::from_str::<SecretsFile>(&content) {
            Ok(file) => AuthSecrets {
                admin_password_hash: file
                    .admin_password_hash
                    .filter(|hash| !hash.trim().is_empty()),
                admin_security_answer_hashes: file.admin_security_answer_hashes,
            },
            Err(e) => {
                error!("解析 {} 失败: {}", secrets_path.display(), e);
                AuthSecrets::default()
            }
        },
        Err(e) => {
            error!("读取 {} 失败: {}", secrets_path.display(), e);
            AuthSecrets::default()
        }
    }
}

/// 原子写入 secrets.toml，Unix 下权限收紧为 0600。
pub fn save_auth_secrets(secrets: &AuthSecrets) -> io::Result<()> {
    save_auth_secrets_to(Path::new(DEFAULT_SECRETS_FILE), secrets)
}

/// 与 [`save_auth_secrets`] 行为一致，但允许指定路径（便于测试）。
pub fn save_auth_secrets_to(secrets_path: &Path, secrets: &AuthSecrets) -> io::Result<()> {
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
        secrets_path.display(),
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

    if let Err(e) = fs::rename(&tmp, secrets_path) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::SecurityConfig;
    use std::sync::Mutex;

    /// 测试用的全局 mutex：避免并发测试在环境变量或 cwd 上互相踩踏。
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    /// 在临时目录里生成一个唯一的 secrets.toml 路径。
    fn unique_tmp_path(label: &str) -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        path.push(format!(
            "lily-nest-test-{}-{}-{:x}.secrets.toml",
            label,
            std::process::id(),
            nanos
        ));
        path
    }

    #[test]
    fn hash_secret_format_and_roundtrip() {
        let h = hash_secret("hunter2");
        assert!(h.starts_with("sha256$"));
        // sha256$<32 hex salt>$<64 hex hash>
        let parts: Vec<&str> = h.split('$').collect();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0], "sha256");
        assert_eq!(parts[1].len(), 32);
        assert_eq!(parts[2].len(), 64);

        // 验证正/反向
        assert!(verify_secret("hunter2", &h));
        assert!(!verify_secret("Hunter2", &h));
        assert!(!verify_secret("", &h));
    }

    #[test]
    fn hash_secret_is_salted() {
        let a = hash_secret("same");
        let b = hash_secret("same");
        // 同一明文两次哈希应得到不同的 salt → 不同结果。
        assert_ne!(a, b);
        // 但两者都必须能验证明文。
        assert!(verify_secret("same", &a));
        assert!(verify_secret("same", &b));
    }

    #[test]
    fn verify_secret_rejects_malformed() {
        // 缺少算法段
        assert!(!verify_secret("x", ""));
        assert!(!verify_secret("x", "sha256$abcd"));
        // 错误的算法
        assert!(!verify_secret("x", "md5$aa$bb"));
        // salt 非 hex
        assert!(!verify_secret("x", "sha256$zz$bb"));
        // salt 奇数长度
        assert!(!verify_secret("x", "sha256$abc$bb"));
    }

    #[test]
    fn save_then_load_preserves_hashes() {
        let path = unique_tmp_path("save-load");
        // 确保起始干净
        let _ = fs::remove_file(&path);

        let answers = vec!["alpha".to_string(), "beta".to_string(), "gamma".to_string()];
        let password = "sup3rsecret";
        let secrets = AuthSecrets {
            admin_password_hash: Some(hash_secret(password)),
            admin_security_answer_hashes: answers.iter().map(|a| hash_secret(a)).collect(),
        };

        save_auth_secrets_to(&path, &secrets).expect("save should succeed");

        // 文件存在且包含哈希字面量
        let raw = fs::read_to_string(&path).expect("file should exist");
        assert!(raw.contains("admin_password_hash = \"sha256$"));
        assert!(raw.contains("admin_security_answer_hashes"));

        // 重新加载并校验每个明文仍能匹配对应哈希
        let loaded = load_secrets_file(&path);
        let password_hash = loaded.admin_password_hash.expect("password hash loaded");
        assert!(verify_secret(password, &password_hash));
        assert_eq!(loaded.admin_security_answer_hashes.len(), answers.len());
        for (plain, stored) in answers.iter().zip(loaded.admin_security_answer_hashes.iter()) {
            assert!(verify_secret(plain, stored));
        }

        // 清场
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn save_replaces_existing_answers_without_touching_password() {
        let path = unique_tmp_path("replace");
        let _ = fs::remove_file(&path);

        // 先写入 password + 旧 answers
        let initial = AuthSecrets {
            admin_password_hash: Some(hash_secret("original-password")),
            admin_security_answer_hashes: vec![hash_secret("old1"), hash_secret("old2")],
        };
        save_auth_secrets_to(&path, &initial).expect("save initial");

        // 模拟 CLI 流程：load_secrets_file → 仅替换 answers → 再保存
        let mut current = load_secrets_file(&path);
        assert_eq!(current.admin_security_answer_hashes.len(), 2);
        let preserved_password = current.admin_password_hash.clone();

        current.admin_security_answer_hashes =
            vec![hash_secret("new1"), hash_secret("new2"), hash_secret("new3")];
        save_auth_secrets_to(&path, &current).expect("save updated");

        // 重新加载，password hash 必须保持一致
        let reloaded = load_secrets_file(&path);
        assert_eq!(reloaded.admin_password_hash, preserved_password);
        assert!(verify_secret(
            "original-password",
            reloaded.admin_password_hash.as_deref().unwrap()
        ));
        assert_eq!(reloaded.admin_security_answer_hashes.len(), 3);
        assert!(verify_secret("new1", &reloaded.admin_security_answer_hashes[0]));
        assert!(verify_secret("new2", &reloaded.admin_security_answer_hashes[1]));
        assert!(verify_secret("new3", &reloaded.admin_security_answer_hashes[2]));

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn load_secrets_file_missing_returns_default() {
        let path = unique_tmp_path("missing");
        let _ = fs::remove_file(&path);
        let loaded = load_secrets_file(&path);
        assert!(loaded.admin_password_hash.is_none());
        assert!(loaded.admin_security_answer_hashes.is_empty());
    }

    #[test]
    fn load_auth_secrets_with_env_var_overrides_everything() {
        let _guard = ENV_LOCK.lock().unwrap();
        let path = unique_tmp_path("env-override");
        let _ = fs::remove_file(&path);

        // 先在磁盘上写一份「诱惑数据」
        let disk_secrets = AuthSecrets {
            admin_password_hash: Some(hash_secret("disk-password")),
            admin_security_answer_hashes: vec![hash_secret("disk-answer")],
        };
        save_auth_secrets_to(&path, &disk_secrets).expect("save disk");

        // 通过环境变量注入「更高优先级」的密码哈希
        let env_pw = "sha256$00$11";
        // SAFETY: 测试串行执行，env lock 保证不会有并发读 env 的测试。
        unsafe {
            std::env::set_var("LILY_ADMIN_PASSWORD_HASH", env_pw);
            std::env::set_var(
                "LILY_ADMIN_SECURITY_ANSWER_HASHES",
                "sha256$22$33,sha256$44$55",
            );
        }

        let mut cfg = SecurityConfig::default();
        // 同时在 cfg 里放明文：环境变量优先级最高，应被忽略。
        cfg.admin_password = Some("plain-from-cfg".to_string());
        cfg.admin_security_answers = Some(vec!["plain-answer".to_string()]);

        let loaded = load_auth_secrets_with(&cfg, &path);

        // SAFETY: 见上。
        unsafe {
            std::env::remove_var("LILY_ADMIN_PASSWORD_HASH");
            std::env::remove_var("LILY_ADMIN_SECURITY_ANSWER_HASHES");
        }

        assert_eq!(loaded.admin_password_hash.as_deref(), Some(env_pw));
        assert_eq!(
            loaded.admin_security_answer_hashes,
            vec!["sha256$22$33".to_string(), "sha256$44$55".to_string()]
        );

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn load_auth_secrets_falls_back_to_config_when_file_absent() {
        let _guard = ENV_LOCK.lock().unwrap();
        // 确保没有环境变量干扰
        // SAFETY: 见 ENV_LOCK 注释。
        unsafe {
            std::env::remove_var("LILY_ADMIN_PASSWORD_HASH");
            std::env::remove_var("LILY_ADMIN_SECURITY_ANSWER_HASHES");
        }

        let path = unique_tmp_path("fallback");
        let _ = fs::remove_file(&path);

        let mut cfg = SecurityConfig::default();
        cfg.admin_password = Some("plain-password".to_string());
        cfg.admin_security_answers = Some(vec!["plain1".to_string(), "plain2".to_string()]);

        let loaded = load_auth_secrets_with(&cfg, &path);

        // 明文密码被就地哈希
        let pw_hash = loaded.admin_password_hash.expect("password hashed");
        assert!(verify_secret("plain-password", &pw_hash));
        assert_eq!(loaded.admin_security_answer_hashes.len(), 2);
        assert!(verify_secret("plain1", &loaded.admin_security_answer_hashes[0]));
        assert!(verify_secret("plain2", &loaded.admin_security_answer_hashes[1]));
    }

    #[test]
    fn load_auth_secrets_placeholder_answers_become_empty() {
        let _guard = ENV_LOCK.lock().unwrap();
        // SAFETY: 见 ENV_LOCK 注释。
        unsafe {
            std::env::remove_var("LILY_ADMIN_PASSWORD_HASH");
            std::env::remove_var("LILY_ADMIN_SECURITY_ANSWER_HASHES");
        }

        let path = unique_tmp_path("placeholder");
        let _ = fs::remove_file(&path);

        let mut cfg = SecurityConfig::default();
        cfg.admin_password = Some("real-password".to_string());
        // 占位符 answers → 应被识别为未初始化，返回空数组
        cfg.admin_security_answers =
            Some(vec!["default1".to_string(), "default2".to_string()]);

        let loaded = load_auth_secrets_with(&cfg, &path);
        assert!(loaded.admin_password_hash.is_some());
        assert!(loaded.admin_security_answer_hashes.is_empty());
    }

    #[test]
    fn secrets_file_only_does_not_read_config_plaintext() {
        // load_secrets_file_only 即使 cfg 有明文，也必须完全不读 cfg。
        let path = unique_tmp_path("only");
        let _ = fs::remove_file(&path);

        // 写一份只有 password 的 secrets.toml
        let initial = AuthSecrets {
            admin_password_hash: Some(hash_secret("p")),
            admin_security_answer_hashes: Vec::new(),
        };
        save_auth_secrets_to(&path, &initial).expect("save");

        // 通过 load_secrets_file_only 读取，answers 必须为空（不应回退到 cfg）
        let mut cfg = SecurityConfig::default();
        cfg.admin_security_answers = Some(vec!["should-not-leak".to_string()]);
        let _ = cfg; // 仅用 cfg 走测试意图：load_secrets_file 不接收 cfg。

        let loaded = load_secrets_file(&path);
        assert!(loaded.admin_password_hash.is_some());
        assert!(loaded.admin_security_answer_hashes.is_empty());

        let _ = fs::remove_file(&path);
    }
}
