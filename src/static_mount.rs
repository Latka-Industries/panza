//! Optional static / SPA mounting for host frontends.
//!
//! Panza never ships UI assets. Hosts pass a [`StaticMount`] into [`crate::run`] /
//! [`crate::serve_router`]:
//!
//! - [`StaticMount::None`] — JSON API + `/health` only (default).
//! - [`StaticMount::Dir`] — files from a build output directory (`dist/`, etc.).
//! - [`StaticMount::Embedded`] — in-memory map (e.g. `include_bytes!` / `rust-embed` unpacked).
//!
//! Both `Dir` and `Embedded` use an SPA-style fallback to `index.html` for unknown paths so
//! client-side routers work. Domain API routes registered on the host router take precedence
//! because static serving is attached as a **fallback**.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use axum::Router;
use axum::body::Body;
use axum::http::{HeaderValue, Request, StatusCode, header};
use axum::response::{IntoResponse, Response};
use tower_http::services::{ServeDir, ServeFile};

/// How (if at all) panza should serve a host frontend beside the API.
///
/// Applied after `/health` and the host `api` router are merged, as a fallback service.
#[derive(Debug, Clone, Default)]
pub enum StaticMount {
    /// No static files — only `/health` and host API routes.
    #[default]
    None,
    /// Serve files from `dir`; unknown paths fall back to `{dir}/index.html` when present.
    Dir(PathBuf),
    /// In-memory assets keyed by URL path without a leading slash
    /// (`"index.html"`, `"assets/app.js"`, …).
    ///
    /// Unknown paths fall back to `"index.html"` when that key exists; otherwise `404`.
    Embedded(HashMap<String, Vec<u8>>),
}

impl StaticMount {
    /// Attach this mount as a fallback on `router`, or return `router` unchanged for [`None`].
    pub(crate) fn apply(self, router: Router) -> Router {
        match self {
            Self::None => router,
            Self::Dir(dir) => mount_dir(router, &dir),
            Self::Embedded(assets) => mount_embedded(router, assets),
        }
    }
}

/// `ServeDir` with SPA `index.html` not-found fallback.
fn mount_dir(router: Router, dir: &Path) -> Router {
    let index = dir.join("index.html");
    let serve = ServeDir::new(dir).not_found_service(ServeFile::new(index));
    router.fallback_service(serve)
}

/// Fallback handler that looks up bytes in `assets` (SPA-aware).
fn mount_embedded(router: Router, assets: HashMap<String, Vec<u8>>) -> Router {
    router.fallback(move |req: Request<Body>| {
        let assets = assets.clone();
        async move { embedded_response(&assets, req.uri().path()) }
    })
}

/// Resolve `path` against `assets`, falling back to `index.html`, with a guessed `Content-Type`.
fn embedded_response(assets: &HashMap<String, Vec<u8>>, path: &str) -> Response {
    let key = normalize_asset_key(path);
    let (key, bytes) = if let Some(b) = assets.get(&key) {
        (key, b.as_slice())
    } else if let Some(b) = assets.get("index.html") {
        ("index.html".to_owned(), b.as_slice())
    } else {
        return (StatusCode::NOT_FOUND, "not found").into_response();
    };

    let mime = mime_guess::from_path(&key).first_or_octet_stream();
    let mut res = Response::new(Body::from(bytes.to_vec()));
    *res.status_mut() = StatusCode::OK;
    if let Ok(val) = HeaderValue::from_str(mime.as_ref()) {
        res.headers_mut().insert(header::CONTENT_TYPE, val);
    }
    res
}

/// Map a request path to an asset map key (`/` → `index.html`, strip leading `/`).
fn normalize_asset_key(path: &str) -> String {
    let trimmed = path.trim_start_matches('/');
    if trimmed.is_empty() {
        "index.html".to_owned()
    } else {
        trimmed.to_owned()
    }
}
