//! Clap serve flags, router assembly, and the blocking [`run`] entrypoint.
//!
//! Prefer [`run`] from a binary’s `serve` subcommand. Use [`serve_router`] in tests or when
//! the host already owns the `TcpListener` / axum serve loop.

use std::net::{IpAddr, SocketAddr};
use std::time::Instant;

use axum::Router;
use axum::routing::get;
use clap::Parser;
use log::{info, warn};

use crate::health::{HealthBody, ServeMeta};
use crate::static_mount::StaticMount;

/// Clap flags for a host `serve` subcommand.
///
/// Flatten into the host’s args with `#[command(flatten)]` so every Latka CLI shares the same
/// `--host` / `--port` / `--open` surface:
///
/// ```ignore
/// #[derive(clap::Parser)]
/// struct ServeCli {
///     #[command(flatten)]
///     serve: panza::ServeArgs,
///     // host-specific args (e.g. DIR) go here
/// }
/// ```
#[derive(Parser, Debug, Clone)]
pub struct ServeArgs {
    /// Interface to bind (IP address). Default is loopback only.
    #[arg(long, default_value = "127.0.0.1")]
    pub host: String,

    /// TCP port to listen on.
    #[arg(long, short = 'p', default_value_t = 8787)]
    pub port: u16,

    /// After a successful bind, open the listen URL in the default browser.
    #[arg(long)]
    pub open: bool,
}

impl ServeArgs {
    /// Parse `host` + `port` into a [`SocketAddr`].
    ///
    /// # Errors
    ///
    /// Returns `Err` when `host` is not a valid IP address (hostnames are not resolved).
    pub fn socket_addr(&self) -> Result<SocketAddr, anyhow::Error> {
        let ip: IpAddr = self
            .host
            .parse()
            .map_err(|e| anyhow::anyhow!("invalid --host {:?}: {e}", self.host))?;
        Ok(SocketAddr::from((ip, self.port)))
    }
}

/// Build the merged router: kit `/health`, then host `api`, then optional static fallback.
///
/// Layering:
/// 1. Panza registers `GET /health` (see [`HealthBody`]).
/// 2. `api` is [`Router::merge`]d in — host domain routes.
/// 3. [`StaticMount`] may attach a fallback for SPA / asset serving.
///
/// [`HealthBody::uptime_secs`] is relative to **this call** (`Instant::now()` here), not to
/// process start or first request. Call once per server instance.
///
/// # Examples
///
/// ```
/// use panza::{ServeMeta, StaticMount, serve_router};
///
/// let app = serve_router(
///     ServeMeta {
///         service: "demo",
///         version: "0.0.0",
///     },
///     axum::Router::new(),
///     StaticMount::None,
/// );
/// # let _ = app;
/// ```
pub fn serve_router(meta: ServeMeta, api: Router, static_mount: StaticMount) -> Router {
    let started = Instant::now();
    let kit = Router::new().route(
        "/health",
        get(move || async move { axum::Json(HealthBody::from_meta(meta, started)) }),
    );
    static_mount.apply(kit.merge(api))
}

/// Bind [`ServeArgs`], serve until Ctrl-C, then shut down gracefully.
///
/// - Resolves [`ServeArgs::socket_addr`].
/// - Warns (via `log`) when the bind address is **not** loopback — panza does not add auth.
/// - Builds the app with [`serve_router`], binds, logs the listen URL, optionally `--open`s it.
///
/// # Errors
///
/// Returns `Err` when the host string is invalid, bind fails, or the axum serve loop errors.
pub async fn run(
    meta: ServeMeta,
    args: ServeArgs,
    api: Router,
    static_mount: StaticMount,
) -> Result<(), anyhow::Error> {
    let addr = args.socket_addr()?;
    if !addr.ip().is_loopback() {
        warn!(
            "binding to non-loopback address {addr}; this exposes the API on the network (no auth)"
        );
    }

    let app = serve_router(meta, api, static_mount);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let local = listener.local_addr()?;
    let url = format!("http://{local}");
    info!(
        "panza listening on {url} (service={}, version={})",
        meta.service, meta.version
    );

    if args.open {
        open_url(&url);
    }

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

/// Wait for Ctrl-C (used as axum’s graceful shutdown signal).
async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}

/// Best-effort “open this URL in the default browser” for `--open`.
///
/// Failures are logged and ignored so a missing browser never kills the server.
fn open_url(url: &str) {
    #[cfg(target_os = "macos")]
    {
        if let Err(e) = std::process::Command::new("open").arg(url).spawn() {
            warn!("failed to open browser: {e}");
        }
    }
    #[cfg(target_os = "linux")]
    {
        if let Err(e) = std::process::Command::new("xdg-open").arg(url).spawn() {
            warn!("failed to open browser: {e}");
        }
    }
    #[cfg(target_os = "windows")]
    {
        if let Err(e) = std::process::Command::new("cmd")
            .args(["/C", "start", url])
            .spawn()
        {
            warn!("failed to open browser: {e}");
        }
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        warn!("--open is not supported on this platform; open {url} manually");
    }
}
