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
    let mut css_dir = ServeDir::new("./static/css");
    let mut css_vendor = ServeDir::new("./static/css/vendor");
    let mut js_dir = ServeDir::new("./static/js");
    let mut js_vendor = ServeDir::new("./static/js/vendor");
    let mut fonts_dir = ServeDir::new("./static/fonts");
    let mut fonts_vendor = ServeDir::new("./static/fonts/vendor");

    if assets_config.precompress {
        css_dir = css_dir.precompressed_zstd().precompressed_br().precompressed_gzip();
        css_vendor = css_vendor.precompressed_zstd().precompressed_br().precompressed_gzip();
        js_dir = js_dir.precompressed_zstd().precompressed_br().precompressed_gzip();
        js_vendor = js_vendor.precompressed_zstd().precompressed_br().precompressed_gzip();
        fonts_dir = fonts_dir.precompressed_zstd().precompressed_br().precompressed_gzip();
        fonts_vendor = fonts_vendor.precompressed_zstd().precompressed_br().precompressed_gzip();
    }

    let css_service = css_dir.fallback(css_vendor);
    let js_service = js_dir.fallback(js_vendor);
    let fonts_service = fonts_dir.fallback(fonts_vendor);

    let root_files = Router::new()
        .route("/robots.txt", get(serve_robots))
        .route("/BingSiteAuth.xml", get(serve_bing_site_auth))
        .route("/sitemap.xml", get(serve_sitemap))
        .route("/favicon.ico", get(serve_favicon))
        .route("/.well-known/security.txt", get(serve_security_txt))
        .route("/{baidu_verify_codeva}", get(serve_baidu_verify));

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
        if let Some(ims_time) = ims.to_str().ok().and_then(crate::utils::parse_http_date) {
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

    let last_modified = crate::utils::fmt_http_date(modified);
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

async fn serve_baidu_verify(
    axum::extract::Path(baidu_verify_codeva): axum::extract::Path<String>,
    req: Request,
) -> Response {
    // 严格安全限制：仅允许以 baidu_ 开头、以 .html 结尾的文件名，且中间校验码必须是合法的安全字符
    if baidu_verify_codeva.starts_with("baidu_") && baidu_verify_codeva.ends_with(".html") {
        let code = baidu_verify_codeva
            .trim_start_matches("baidu_")
            .trim_end_matches(".html");
        if code.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            let path = format!("./static/{}", baidu_verify_codeva);
            let assets_config = crate::config::load_assets_config();
            let cc = format!("public, max-age={}", assets_config.other_cache_seconds);
            serve_static_file(&path, "text/html; charset=utf-8", &cc, req).await
        } else {
            StatusCode::BAD_REQUEST.into_response()
        }
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

async fn serve_security_txt(req: Request) -> Response {
    let assets_config = crate::config::load_assets_config();
    let cc = format!("public, max-age={}", assets_config.other_cache_seconds);
    serve_static_file(
        "./static/.well-known/security.txt",
        "text/plain; charset=utf-8",
        &cc,
        req,
    ).await
}
