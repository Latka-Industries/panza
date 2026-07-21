//! Health endpoint types.

use std::time::Instant;

use serde::Serialize;

/// Service identity included in `GET /health` JSON.
#[derive(Debug, Clone, Copy)]
pub struct ServeMeta {
    /// Short service name (e.g. `"ublx"`).
    pub service: &'static str,
    /// Version string (typically `env!("CARGO_PKG_VERSION")`).
    pub version: &'static str,
}

/// Default `/health` response body.
#[derive(Debug, Clone, Serialize)]
pub struct HealthBody {
    /// Always `true` when the process is accepting connections.
    pub ok: bool,
    /// From [`ServeMeta::service`].
    pub service: &'static str,
    /// From [`ServeMeta::version`].
    pub version: &'static str,
    /// Seconds since this server instance started accepting requests.
    pub uptime_secs: u64,
}

impl HealthBody {
    /// Build a healthy response from meta and a start clock.
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
