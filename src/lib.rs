//! **panza** — the squire for your CLI.
//!
//! Shared clap `serve` shell for Latka (and other) binaries: bind an address, expose
//! [`GET /health`](HealthBody), optionally mount a host frontend, and merge in the host’s
//! domain [`axum::Router`]. Host crates own routes, DB access, and UI; panza only serves.
//!
//! # Typical host wiring
//!
//! Flatten [`ServeArgs`] into a clap `serve` subcommand, build your API router, then call
//! [`run`] (blocks until Ctrl-C) or [`serve_router`] if you need a custom bind loop.
//!
//! ```
//! use panza::{ServeMeta, StaticMount, serve_router};
//!
//! let app = serve_router(
//!     ServeMeta {
//!         service: "my-tool",
//!         version: env!("CARGO_PKG_VERSION"),
//!     },
//!     axum::Router::new(), // host routes here
//!     StaticMount::None,   // or Dir / Embedded for a SPA
//! );
//! # let _ = app;
//! ```
//!
//! ```ignore
//! // Full CLI entrypoint (needs a runtime + bind):
//! use clap::Parser;
//! use panza::{ServeArgs, ServeMeta, StaticMount, run};
//!
//! #[derive(Parser)]
//! struct ServeCli {
//!     #[command(flatten)]
//!     serve: ServeArgs,
//! }
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let cli = ServeCli::parse();
//!     run(
//!         ServeMeta {
//!             service: "my-tool",
//!             version: env!("CARGO_PKG_VERSION"),
//!         },
//!         cli.serve,
//!         axum::Router::new(),
//!         StaticMount::None,
//!     )
//!     .await
//! }
//! ```
//!
//! # What panza does not do
//!
//! - Domain JSON / SQL / business logic
//! - Auth, TLS, or multi-tenant hosting (non-loopback binds only log a warning)
//! - Owning a shared frontend — each host brings its own `StaticMount`

#![deny(missing_docs)]

mod health;
mod run;
mod static_mount;

pub use health::{HealthBody, ServeMeta};
pub use run::{ServeArgs, run, serve_router};
pub use static_mount::StaticMount;
