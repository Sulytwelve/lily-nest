mod app;
mod compressor;
mod config;
mod middlewares;
mod model;
mod routes;
pub mod render;
pub mod state;



use axum_server::tls_rustls::RustlsConfig;
use std::net::SocketAddr;
use tracing::info;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;

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
    let app = app::build_app();

    if cfg!(debug_assertions) {
        // debug：直接 HTTP 8880
        let addr: SocketAddr = "[::]:8880".parse().expect("解析地址失败");
        info!(">> 梨窝 已启动 (dev): http://{}", addr);
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .expect("Failed to bind address");
        axum::serve(listener, app).await.expect("Server error");
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

        let addr: SocketAddr = "[::]:8443".parse().expect("解析地址失败");
        info!(">> 梨窝 已启动: https://{}", addr);
        axum_server::bind_rustls(addr, config)
            .serve(app.into_make_service())
            .await
            .expect("Server error");
    }
}
