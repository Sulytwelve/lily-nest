use axum::{
    Router,
    http::{HeaderValue, header},
};
use tower_http::{
    services::{ServeDir, ServeFile},
    set_header::SetResponseHeaderLayer,
};

pub fn router() -> Router {
    // Root-level files: 1 hour cache
    let root_files = Router::new()
        .route_service("/robots.txt", ServeFile::new("./static/robots.txt"))
        .route_service(
            "/BingSiteAuth.xml",
            ServeFile::new("./static/BingSiteAuth.xml"),
        )
        .route_service("/sitemap.xml", ServeFile::new("./static/sitemap.xml"))
        .route_service("/favicon.ico", ServeFile::new("./static/favicon.ico"))
        .layer(SetResponseHeaderLayer::overriding(
            header::CACHE_CONTROL,
            HeaderValue::from_static("public, max-age=3600"),
        ));

    // CSS/JS/Images: 1 day cache + Vary: Accept-Encoding for Cloudflare
    let assets_daily = Router::new()
        .nest_service(
            "/css",
            ServeDir::new("./static/css")
                .precompressed_zstd()
                .precompressed_br()
                .precompressed_gzip(),
        )
        .nest_service(
            "/js",
            ServeDir::new("./static/js")
                .precompressed_zstd()
                .precompressed_br()
                .precompressed_gzip(),
        )
        .nest_service("/images", ServeDir::new("./static/images"))
        .layer(SetResponseHeaderLayer::overriding(
            header::CACHE_CONTROL,
            HeaderValue::from_static("public, max-age=86400"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::VARY,
            HeaderValue::from_static("Accept-Encoding"),
        ));

    // Fonts: 7 day cache + Vary: Accept-Encoding for Cloudflare
    let assets_weekly = Router::new()
        .nest_service(
            "/fonts",
            ServeDir::new("./static/fonts")
                .precompressed_zstd()
                .precompressed_br()
                .precompressed_gzip(),
        )
        .layer(SetResponseHeaderLayer::overriding(
            header::CACHE_CONTROL,
            HeaderValue::from_static("public, max-age=604800"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::VARY,
            HeaderValue::from_static("Accept-Encoding"),
        ));

    root_files.merge(assets_daily).merge(assets_weekly)
}
