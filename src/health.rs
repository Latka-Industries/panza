//! [`ServeMeta`] identity and the JSON body for `GET /health`.
//!
//! The kit mounts `/health` inside [`crate::serve_router`]. Hosts can still define their own
//! health route if they merge a conflicting path (axum’s merge rules apply); prefer letting
//! panza own `/health` so `ok` / `service` / `version` / `uptime_secs` stay consistent.

use std::time::Instant;

use serde::Serialize;

/// Service identity stamped into every [`HealthBody`].
///
/// Pass this into [`crate::run`] / [`crate::serve_router`]. Keep `service` short and stable
/// (crate or binary name); `version` is usually `env!("CARGO_PKG_VERSION")`.
///
/// This type is intentionally free of clocks or request state — uptime is computed when
/// building each [`HealthBody`].
#[derive(Debug, Clone, Copy)]
pub struct ServeMeta {
    /// Short service name (e.g. `"ublx"`, `"nefaxer"`).
    pub service: &'static str,
    /// Version string (typically `env!("CARGO_PKG_VERSION")`).
    pub version: &'static str,
}

/// JSON body returned by panza’s `GET /health`.
///
/// Example:
///
/// ```json
/// { "ok": true, "service": "ublx", "version": "0.1.10", "uptime_secs": 42 }
/// ```
///
/// `uptime_secs` is whole seconds since the [`Instant`] passed to [`HealthBody::from_meta`]
/// (panza uses the instant when [`crate::serve_router`] is called).
#[derive(Debug, Clone, Serialize)]
pub struct HealthBody {
    /// Always `true` when this handler runs (process is accepting connections).
    pub ok: bool,
    /// From [`ServeMeta::service`].
    pub service: &'static str,
    /// From [`ServeMeta::version`].
    pub version: &'static str,
    /// Whole seconds elapsed since `started` (see [`HealthBody::from_meta`]).
    pub uptime_secs: u64,
}

impl HealthBody {
    /// Build a healthy response from identity + a start clock.
    ///
    /// `started` should be the instant this server instance began (panza records it at
    /// [`crate::serve_router`] construction). `uptime_secs` uses [`Instant::elapsed`] and
    /// truncates to whole seconds.
    #[must_use]
    pub fn from_meta(meta: ServeMeta, started: Instant) -> Self {
        Self {
            ok: true,
            service: meta.service,
            version: meta.version,
            uptime_secs: started.elapsed().as_secs(),
        }
    }
}
