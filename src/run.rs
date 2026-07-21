//! Bind, clap args, and the main `run` entrypoint.

use std::net::{IpAddr, SocketAddr};

use axum::Router;
use axum::routing::get;
use clap::Parser;
use log::{info, warn};

use crate::health::{HealthBody, ServeMeta};
use crate::static_mount::StaticMount;

/// Clap flags for a host `serve` subcommand (`#[command(flatten)]`).
#[derive(Parser, Debug, Clone)]
pub struct ServeArgs {
    /// Bind address (default: localhost).
    #[arg(long, default_value = "127.0.0.1")]
    pub host: String,

    /// TCP port.
    #[arg(long, short = 'p', default_value_t = 8787)]
    pub port: u16,

    /// Open the listen URL in the default browser after bind.
    #[arg(long)]
    pub open: bool,
}

impl ServeArgs {
    /// Parse `host` + `port` into a socket address.
    ///
    /// # Errors
    ///
    /// Returns `Err` when `host` is not a valid IP address.
    pub fn socket_addr(&self) -> Result<SocketAddr, anyhow::Error> {
        let ip: IpAddr = self
            .host
            .parse()
            .map_err(|e| anyhow::anyhow!("invalid --host {:?}: {e}", self.host))?;
        Ok(SocketAddr::from((ip, self.port)))
    }
}

/// Build the merged router: kit `/health`, then host `api`, then optional static fallback.
pub fn serve_router(meta: ServeMeta, api: Router, static_mount: StaticMount) -> Router {
    let health = HealthBody::from_meta(meta);
    let kit = Router::new().route("/health", get(move || async move { axum::Json(health) }));
    static_mount.apply(kit.merge(api))
}

/// Bind and serve until Ctrl-C.
///
/// Logs a warning when the bind host is not a loopback address.
///
/// # Errors
///
/// Returns `Err` on invalid host, bind failure, or serve errors.
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

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}

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
