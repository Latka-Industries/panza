//! Static / SPA mount options for host frontends.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use axum::Router;
use axum::body::Body;
use axum::http::{HeaderValue, Request, StatusCode, header};
use axum::response::{IntoResponse, Response};
use tower_http::services::{ServeDir, ServeFile};

/// How (if at all) panza should serve a host frontend.
#[derive(Debug, Clone, Default)]
pub enum StaticMount {
    /// No static files (API + `/health` only).
    #[default]
    None,
    /// Serve files from a directory; missing paths fall back to `index.html` (SPA).
    Dir(PathBuf),
    /// In-memory assets keyed by URL path (`"index.html"`, `"assets/app.js"`, …).
    /// Missing paths fall back to `index.html` when present (SPA).
    Embedded(HashMap<String, Vec<u8>>),
}

impl StaticMount {
    /// Attach static serving to `router` when configured.
    pub(crate) fn apply(self, router: Router) -> Router {
        match self {
            Self::None => router,
            Self::Dir(dir) => mount_dir(router, &dir),
            Self::Embedded(assets) => mount_embedded(router, assets),
        }
    }
}

fn mount_dir(router: Router, dir: &Path) -> Router {
    let index = dir.join("index.html");
    let serve = ServeDir::new(dir).not_found_service(ServeFile::new(index));
    router.fallback_service(serve)
}

fn mount_embedded(router: Router, assets: HashMap<String, Vec<u8>>) -> Router {
    router.fallback(move |req: Request<Body>| {
        let assets = assets.clone();
        async move { embedded_response(&assets, req.uri().path()) }
    })
}

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

fn normalize_asset_key(path: &str) -> String {
    let trimmed = path.trim_start_matches('/');
    if trimmed.is_empty() {
        "index.html".to_owned()
    } else {
        trimmed.to_owned()
    }
}
