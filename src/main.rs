mod app;
mod compressor;
mod config;
mod middlewares;
mod model;
mod routes;
pub mod note_loader;
pub mod render;
pub mod state;

use axum_server::tls_rustls::RustlsConfig;
use std::net::SocketAddr;
use tracing::{info, warn};
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;

const HTTP2_MAX_HEADER_LIST_SIZE: u32 = 32 * 1024;

#[tokio::main]
async fn main() {
    // 初始化 tracing，将日志输出到控制台
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::new("info"))
        .init();

    // 预压缩资源文件
    let assets_config = config::load_assets_config();
    if assets_config.precompress {
        compressor::ensure_precompressed_assets(&assets_config);
    }

    // 构建应用（路由、静态资源等）
    let app = app::build_app().await;

    let server_config = config::load_server_config();

    if cfg!(any(debug_assertions, feature = "force-http")) {
        if !cfg!(debug_assertions) && cfg!(feature = "force-http") {
            warn!("[security] 警告: 当前正在 Release 模式下强制使用未加密的 HTTP 服务! 管理后台密码在传输时可能会被明文拦截，存在极大的安全风险!");
        }

        // debug：直接 HTTP port
        let addr_str = format!("[::]:{}", server_config.http_port);
        let addr: SocketAddr = addr_str.parse().expect("解析地址失败");
        info!(">> 梨窝 已启动 (dev): http://{}", addr);
        let mut server = axum_server::bind(addr);
        server.http_builder().http2().max_header_list_size(HTTP2_MAX_HEADER_LIST_SIZE);
        server
            .serve(app.into_make_service())
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
        server.http_builder().http2().max_header_list_size(HTTP2_MAX_HEADER_LIST_SIZE);
        server
            .serve(app.into_make_service())
            .await
            .expect("Server error");
    }
}
