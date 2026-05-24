use axum::{
    Router,
    extract::Request,
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use tower_http::{
    services::ServeDir,
    set_header::SetResponseHeaderLayer,
};

pub fn router() -> Router {
    let assets_config = crate::config::load_assets_config();
    let mut css_service = ServeDir::new("./static/css");
    let mut js_service = ServeDir::new("./static/js");
    let mut fonts_service = ServeDir::new("./static/fonts");

    if assets_config.precompress {
        css_service = css_service.precompressed_zstd().precompressed_br().precompressed_gzip();
        js_service = js_service.precompressed_zstd().precompressed_br().precompressed_gzip();
        fonts_service = fonts_service.precompressed_zstd().precompressed_br().precompressed_gzip();
    }

    let root_files = Router::new()
        .route("/robots.txt", get(serve_robots))
        .route("/BingSiteAuth.xml", get(serve_bing_site_auth))
        .route("/sitemap.xml", get(serve_sitemap))
        .route("/favicon.ico", get(serve_favicon));

    let assets_daily = Router::new()
        .nest_service("/css", css_service)
        .nest_service("/js", js_service)
        .nest_service("/images", ServeDir::new("./static/images"))
        .layer(SetResponseHeaderLayer::overriding(
            header::CACHE_CONTROL,
            HeaderValue::from_static("public, max-age=86400"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::VARY,
            HeaderValue::from_static("Accept-Encoding"),
        ));

    let assets_weekly = Router::new()
        .nest_service("/fonts", fonts_service)
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

async fn serve_static_file(
    path: &str,
    content_type: &'static str,
    cache_control: &'static str,
    req: Request,
) -> Response {
    let metadata = match tokio::fs::metadata(path).await {
        Ok(m) => m,
        Err(_) => return StatusCode::NOT_FOUND.into_response(),
    };

    let modified = match metadata.modified() {
        Ok(m) => m,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    if let Some(ims) = req.headers().get(header::IF_MODIFIED_SINCE) {
        if let Some(ims_time) = ims.to_str().ok().and_then(|v| httpdate::parse_http_date(v).ok()) {
            let modified_secs = modified.duration_since(std::time::SystemTime::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
            let ims_secs = ims_time.duration_since(std::time::SystemTime::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
            if modified_secs <= ims_secs {
                let mut res = Response::new(axum::body::Body::empty());
                *res.status_mut() = StatusCode::NOT_MODIFIED;
                res.headers_mut()
                    .insert(header::CACHE_CONTROL, HeaderValue::from_static(cache_control));
                return res;
            }
        }
    }

    let contents = match tokio::fs::read(path).await {
        Ok(c) => c,
        Err(_) => return StatusCode::NOT_FOUND.into_response(),
    };

    let last_modified = httpdate::fmt_http_date(modified);
    let mut res = Response::new(axum::body::Body::from(contents));
    res.headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
    res.headers_mut().insert(
        header::LAST_MODIFIED,
        HeaderValue::from_str(&last_modified)
            .unwrap_or(HeaderValue::from_static("Thu, 01 Jan 1970 00:00:00 GMT")),
    );
    res.headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static(cache_control));
    res
}

async fn serve_robots(req: Request) -> Response {
    serve_static_file("./static/robots.txt", "text/plain", "public, max-age=3600", req).await
}

async fn serve_bing_site_auth(req: Request) -> Response {
    serve_static_file("./static/BingSiteAuth.xml", "application/xml", "public, max-age=3600", req).await
}

async fn serve_sitemap(req: Request) -> Response {
    serve_static_file("./static/sitemap.xml", "application/xml", "public, max-age=3600", req).await
}

async fn serve_favicon(req: Request) -> Response {
    serve_static_file("./static/favicon.ico", "image/x-icon", "public, max-age=3600", req).await
}
