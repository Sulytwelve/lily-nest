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

    let js_css_cc = format!("public, max-age={}", assets_config.js_css_cache_seconds);
    let js_css_cc_val = HeaderValue::try_from(js_css_cc)
        .unwrap_or_else(|_| HeaderValue::from_static("public, max-age=86400"));

    let image_cc = format!("public, max-age={}", assets_config.image_cache_seconds);
    let image_cc_val = HeaderValue::try_from(image_cc)
        .unwrap_or_else(|_| HeaderValue::from_static("public, max-age=86400"));

    let font_cc = format!("public, max-age={}", assets_config.font_cache_seconds);
    let font_cc_val = HeaderValue::try_from(font_cc)
        .unwrap_or_else(|_| HeaderValue::from_static("public, max-age=604800"));

    let css_js_router = Router::new()
        .nest_service("/css", css_service)
        .nest_service("/js", js_service)
        .layer(SetResponseHeaderLayer::overriding(
            header::CACHE_CONTROL,
            js_css_cc_val,
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::VARY,
            HeaderValue::from_static("Accept-Encoding"),
        ));

    let images_router = Router::new()
        .nest_service("/images", ServeDir::new("./static/images"))
        .layer(SetResponseHeaderLayer::overriding(
            header::CACHE_CONTROL,
            image_cc_val,
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::VARY,
            HeaderValue::from_static("Accept-Encoding"),
        ));

    let assets_weekly = Router::new()
        .nest_service("/fonts", fonts_service)
        .layer(SetResponseHeaderLayer::overriding(
            header::CACHE_CONTROL,
            font_cc_val,
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::VARY,
            HeaderValue::from_static("Accept-Encoding"),
        ));

    root_files
        .merge(css_js_router)
        .merge(images_router)
        .merge(assets_weekly)
}

async fn serve_static_file(
    path: &str,
    content_type: &'static str,
    cache_control: &str,
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

    let cc_val = HeaderValue::try_from(cache_control)
        .unwrap_or_else(|_| HeaderValue::from_static("public, max-age=3600"));

    if let Some(ims) = req.headers().get(header::IF_MODIFIED_SINCE) {
        if let Some(ims_time) = ims.to_str().ok().and_then(|v| httpdate::parse_http_date(v).ok()) {
            let modified_secs = modified.duration_since(std::time::SystemTime::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
            let ims_secs = ims_time.duration_since(std::time::SystemTime::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
            if modified_secs <= ims_secs {
                let mut res = Response::new(axum::body::Body::empty());
                *res.status_mut() = StatusCode::NOT_MODIFIED;
                res.headers_mut()
                    .insert(header::CACHE_CONTROL, cc_val);
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
        .insert(header::CACHE_CONTROL, cc_val);
    res
}

async fn serve_robots(req: Request) -> Response {
    let assets_config = crate::config::load_assets_config();
    let cc = format!("public, max-age={}", assets_config.other_cache_seconds);
    serve_static_file("./static/robots.txt", "text/plain", &cc, req).await
}

async fn serve_bing_site_auth(req: Request) -> Response {
    let assets_config = crate::config::load_assets_config();
    let cc = format!("public, max-age={}", assets_config.other_cache_seconds);
    serve_static_file("./static/BingSiteAuth.xml", "application/xml", &cc, req).await
}

async fn serve_sitemap(req: Request) -> Response {
    let assets_config = crate::config::load_assets_config();
    let cc = format!("public, max-age={}", assets_config.other_cache_seconds);
    serve_static_file("./static/sitemap.xml", "application/xml", &cc, req).await
}

async fn serve_favicon(req: Request) -> Response {
    let assets_config = crate::config::load_assets_config();
    let cc = format!("public, max-age={}", assets_config.other_cache_seconds);
    serve_static_file("./static/favicon.ico", "image/x-icon", &cc, req).await
}
