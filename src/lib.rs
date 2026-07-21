//! **panza** — the squire for your CLI.
//!
//! Shared clap `serve` shell: bind, `/health`, optional static/SPA mount.
//! Host crates supply domain routes and their own frontend.
//!
//! ```ignore
//! use clap::Parser;
//! use panza::{ServeArgs, ServeMeta, StaticMount, run};
//!
//! #[derive(Parser)]
//! struct ServeCli {
//!     #[command(flatten)]
//!     serve: ServeArgs,
//! }
//!
//! # async fn example(cli: ServeCli, api: axum::Router) -> anyhow::Result<()> {
//! run(
//!     ServeMeta {
//!         service: "my-tool",
//!         version: env!("CARGO_PKG_VERSION"),
//!     },
//!     cli.serve,
//!     api,
//!     StaticMount::None,
//! )
//! .await?;
//! # Ok(())
//! # }
//! ```

#![deny(missing_docs)]

mod health;
mod run;
mod static_mount;

pub use health::{HealthBody, ServeMeta};
pub use run::{ServeArgs, run, serve_router};
pub use static_mount::StaticMount;
