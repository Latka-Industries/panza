//! Integration smoke tests for panza.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;

use axum::Router;
use axum::routing::get;
use panza::{ServeMeta, StaticMount, serve_router};
use tokio::net::TcpListener;

async fn spawn_app(app: Router) -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().expect("local_addr");
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve");
    });
    // Tiny yield so the accept loop is ready.
    tokio::time::sleep(Duration::from_millis(20)).await;
    addr
}

#[tokio::test]
async fn health_returns_meta() {
    let app = serve_router(
        ServeMeta {
            service: "panza-test",
            version: "0.0.0-test",
        },
        Router::new(),
        StaticMount::None,
    );
    let addr = spawn_app(app).await;
    let body: serde_json::Value = reqwest::get(format!("http://{addr}/health"))
        .await
        .expect("get")
        .json()
        .await
        .expect("json");
    assert_eq!(body["ok"], true);
    assert_eq!(body["service"], "panza-test");
    assert_eq!(body["version"], "0.0.0-test");
    assert!(body["uptime_secs"].as_u64().is_some());
}

#[tokio::test]
async fn health_uptime_increases() {
    let app = serve_router(
        ServeMeta {
            service: "panza-test",
            version: "0.0.0-test",
        },
        Router::new(),
        StaticMount::None,
    );
    let addr = spawn_app(app).await;
    let first: serde_json::Value = reqwest::get(format!("http://{addr}/health"))
        .await
        .expect("get")
        .json()
        .await
        .expect("json");
    tokio::time::sleep(Duration::from_secs(1)).await;
    let second: serde_json::Value = reqwest::get(format!("http://{addr}/health"))
        .await
        .expect("get")
        .json()
        .await
        .expect("json");
    assert!(second["uptime_secs"].as_u64().unwrap() > first["uptime_secs"].as_u64().unwrap());
}

#[tokio::test]
async fn host_api_routes_merge() {
    let api = Router::new().route("/ping", get(|| async { "pong" }));
    let app = serve_router(
        ServeMeta {
            service: "panza-test",
            version: "0.0.0-test",
        },
        api,
        StaticMount::None,
    );
    let addr = spawn_app(app).await;
    let text = reqwest::get(format!("http://{addr}/ping"))
        .await
        .expect("get")
        .text()
        .await
        .expect("text");
    assert_eq!(text, "pong");
}

#[tokio::test]
async fn embedded_static_spa_fallback() {
    let mut assets = HashMap::new();
    assets.insert("index.html".into(), b"<html>hi</html>".to_vec());
    assets.insert("assets/app.js".into(), b"console.log(1)".to_vec());

    let app = serve_router(
        ServeMeta {
            service: "panza-test",
            version: "0.0.0-test",
        },
        Router::new(),
        StaticMount::Embedded(assets),
    );
    let addr = spawn_app(app).await;

    let js = reqwest::get(format!("http://{addr}/assets/app.js"))
        .await
        .expect("get js")
        .text()
        .await
        .expect("text");
    assert_eq!(js, "console.log(1)");

    let spa = reqwest::get(format!("http://{addr}/some/client/route"))
        .await
        .expect("get spa")
        .text()
        .await
        .expect("text");
    assert_eq!(spa, "<html>hi</html>");
}

#[tokio::test]
async fn dir_static_serves_index() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("index.html"), b"<html>dir</html>").expect("write");

    let app = serve_router(
        ServeMeta {
            service: "panza-test",
            version: "0.0.0-test",
        },
        Router::new(),
        StaticMount::Dir(dir.path().to_path_buf()),
    );
    let addr = spawn_app(app).await;
    let html = reqwest::get(format!("http://{addr}/"))
        .await
        .expect("get")
        .text()
        .await
        .expect("text");
    assert_eq!(html, "<html>dir</html>");
}
