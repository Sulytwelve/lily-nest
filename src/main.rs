mod app;
mod compressor;
mod config;
mod middlewares;
mod model;
pub mod note_loader;
pub mod render;
mod routes;
mod secrets;
pub mod state;
pub mod utils;

use axum_server::tls_rustls::RustlsConfig;
use std::net::SocketAddr;
use tracing::{info, warn};
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;

const HTTP2_MAX_HEADER_LIST_SIZE: u32 = 32 * 1024;

#[tokio::main]
async fn main() {
    // 初始化 tracing，将日志输出到控制台；M8：尊重 RUST_LOG，默认 info
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(env_filter)
        .init();

    // 子命令：set-password —— 初始化/修改管理员密码（不启动服务器）
    if std::env::args().nth(1).as_deref() == Some("set-password") {
        handle_set_password_command();
        std::process::exit(0);
    }

    // 子命令：set-security-answers —— 交互式设置密保答案（哈希后写入 secrets.toml）
    if std::env::args().nth(1).as_deref() == Some("set-security-answers") {
        handle_set_security_answers_command();
        std::process::exit(0);
    }

    // 预压缩资源文件
    let assets_config = config::load_assets_config();
    if assets_config.precompress {
        compressor::ensure_precompressed_assets(&assets_config);
    }

    // 确保 notes/ 目录存在，避免首次创建笔记时失败
    if let Err(e) = tokio::fs::create_dir_all("notes").await {
        warn!("无法创建 notes/ 目录: {}", e);
    }

    // 构建应用（路由、静态资源等）；assets_config 同时用于预压缩与应用状态，
    // 避免同一份 config.toml 在启动阶段被重复解析（M2）。
    let app = app::build_app(assets_config).await;

    let server_config = config::load_server_config();

    if cfg!(any(debug_assertions, feature = "force-http")) {
        if !cfg!(debug_assertions) && cfg!(feature = "force-http") {
            warn!(
                "[security] 警告: 当前正在 Release 模式下强制使用未加密的 HTTP 服务! 管理后台密码在传输时可能会被明文拦截，存在极大的安全风险!"
            );
        }

        // debug：直接 HTTP port
        let addr_str = format!("[::]:{}", server_config.http_port);
        let addr: SocketAddr = addr_str.parse().expect("解析地址失败");
        info!(">> 梨窝 已启动 (dev): http://{}", addr);
        let mut server = axum_server::bind(addr);
        server
            .http_builder()
            .http2()
            .max_header_list_size(HTTP2_MAX_HEADER_LIST_SIZE);
        server
            .serve(app.into_make_service_with_connect_info::<std::net::SocketAddr>())
            .await
            .expect("Server error");
    } else {
        // release：必须有证书，否则 panic
        let tls = config::load_tls_config().expect("release 模式下必须配置 TLS，缺少证书配置");

        assert!(
            std::path::Path::new(&tls.cert_path).exists(),
            "release 模式下证书文件不存在: {}",
            tls.cert_path
        );
        assert!(
            std::path::Path::new(&tls.key_path).exists(),
            "release 模式下私钥文件不存在: {}",
            tls.key_path
        );

        let config = RustlsConfig::from_pem_file(&tls.cert_path, &tls.key_path)
            .await
            .expect("加载 TLS 证书失败");

        let addr_str = format!("[::]:{}", server_config.https_port);
        let addr: SocketAddr = addr_str.parse().expect("解析地址失败");
        info!(">> 梨窝 已启动: https://{}", addr);
        let mut server = axum_server::bind_rustls(addr, config);
        server
            .http_builder()
            .http2()
            .max_header_list_size(HTTP2_MAX_HEADER_LIST_SIZE);
        server
            .serve(app.into_make_service_with_connect_info::<std::net::SocketAddr>())
            .await
            .expect("Server error");
    }
}

/// `lily-nest set-password`：初始化或修改管理员密码并写入 secrets.toml。
fn handle_set_password_command() {
    let args: Vec<String> = std::env::args().collect();
    let generate = args.iter().any(|arg| arg == "--generate");

    let password = if generate {
        let password = generate_random_password(16);
        println!("Generated admin password: {}（请立即保存）", password);
        password
    } else {
        let password = match rpassword::prompt_password("New admin password: ") {
            Ok(value) => value,
            Err(e) => {
                eprintln!("读取密码失败: {}", e);
                std::process::exit(1);
            }
        };
        let confirm = match rpassword::prompt_password("Confirm password: ") {
            Ok(value) => value,
            Err(e) => {
                eprintln!("读取确认密码失败: {}", e);
                std::process::exit(1);
            }
        };
        if password != confirm {
            eprintln!("两次输入的密码不一致");
            std::process::exit(1);
        }
        password
    };

    if password.chars().count() < 8 {
        eprintln!("密码至少 8 个字符");
        std::process::exit(1);
    }

    let security_config = config::load_security_config();
    let mut secrets = secrets::load_auth_secrets(&security_config);
    secrets.admin_password_hash = Some(secrets::hash_secret(&password));

    if let Err(e) = secrets::save_auth_secrets(&secrets) {
        eprintln!("写入 secrets.toml 失败: {}", e);
        std::process::exit(1);
    }
    println!("管理员密码已写入 secrets.toml（已哈希加盐）");
    std::process::exit(0);
}

/// `lily-nest set-security-answers`：交互式设置密保答案并写入 secrets.toml。
///
/// 流程：
/// 1. 从 `config.toml` 的 `[security] admin_security_questions` 读取题目（非秘密）；
/// 2. 检查题目不是占位符、且与 `auth_ext_secq` 设置一致；
/// 3. 逐题用 `rpassword` 隐藏输入答案（不做二次确认，避免冗长）；
/// 4. 把答案加盐哈希后写入 `secrets.toml` 的 `admin_security_answer_hashes`，
///    保留已存在的 `admin_password_hash`（避免无意把 config.toml 明文密码提升进来）。
fn handle_set_security_answers_command() {
    let security_config = config::load_security_config();

    // 1. 题目必须存在且非占位符
    let questions = match security_config.admin_security_questions.as_ref() {
        Some(questions) if !questions.is_empty() => questions.clone(),
        _ => {
            eprintln!(
                "config.toml 的 [security] 段没有配置 admin_security_questions。\n\
                 请先在 config.toml 里添加题目（非秘密，可明文），再运行本命令。\n\
                 示例：\n  \
                   admin_security_questions = [\"你的第一只宠物叫什么？\", \"你在哪里出生？\", \"你最喜欢的老师是谁？\"]"
            );
            std::process::exit(1);
        }
    };

    const PLACEHOLDER_QUESTIONS: &[&str] = &["default1", "default2", "default3"];
    if questions
        .iter()
        .any(|q| PLACEHOLDER_QUESTIONS.contains(&q.as_str()))
    {
        eprintln!(
            "config.toml 的 admin_security_questions 仍是占位符 (default1/default2/default3)。\n\
             请先在 config.toml 里把它替换为真实题目，再运行本命令。"
        );
        std::process::exit(1);
    }

    // 2. auth_ext_secq 关闭时给出提醒，但不阻止写入（运维可能想先哈希保存）
    if security_config.auth_ext_secq != Some(true) {
        eprintln!(
            "[提醒] config.toml 中 auth_ext_secq = false —— 密保题在登录流程中不会被使用。\n\
             仍然继续把答案哈希并写入 secrets.toml。\n"
        );
    }

    // 3. 逐题交互式输入答案
    println!("题目（来自 config.toml，共 {} 题）：", questions.len());
    let mut new_hashes = Vec::with_capacity(questions.len());
    for (idx, question) in questions.iter().enumerate() {
        println!("{}. {}", idx + 1, question);
        let answer = loop {
            let input = match rpassword::prompt_password("   Answer: ") {
                Ok(value) => value,
                Err(e) => {
                    eprintln!("读取答案失败: {}", e);
                    std::process::exit(1);
                }
            };
            if input.is_empty() {
                eprintln!("答案不能为空，请重新输入。");
                continue;
            }
            break input;
        };
        new_hashes.push(secrets::hash_secret(&answer));
    }

    // 4. 合并：保留现有 password hash（仅读 secrets.toml，不读 config.toml 明文，
    //    避免把还没迁移的明文密码意外提升进 secrets.toml）。
    let mut secrets = secrets::load_secrets_file_only();
    let old_answer_count = secrets.admin_security_answer_hashes.len();
    secrets.admin_security_answer_hashes = new_hashes;

    if let Err(e) = secrets::save_auth_secrets(&secrets) {
        eprintln!("写入 secrets.toml 失败: {}", e);
        std::process::exit(1);
    }

    println!(
        "✅ 已写入 secrets.toml（已哈希加盐，共 {} 个密保答案；旧的 {} 个答案哈希已替换）",
        secrets.admin_security_answer_hashes.len(),
        old_answer_count
    );
    if secrets.admin_password_hash.is_none() {
        println!(
            "[提醒] secrets.toml 中暂无 admin_password_hash，请运行 `lily-nest set-password` 设置密码。"
        );
    }
}

fn generate_random_password(len: usize) -> String {
    use rand::RngExt;

    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::rng();
    (0..len)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}
