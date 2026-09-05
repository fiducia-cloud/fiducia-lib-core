//! Server-only primitives for API, web, admin, worker, and lambda binaries.
//!
//! Persistence goes through `fiducia-orm-core`; this module never opens a
//! database connection itself. Request lifecycle, interface markers, and
//! rate-limit policy are centralized here so the four standard servers do not
//! drift.

pub mod platform;

use crate::config::{RuntimeConfig, ServiceRole};

pub use platform::{install_axum, PlatformError, ServiceProfile, SharedRateLimiter};

/// Which of the four web↔API avenues a server may use.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Avenue {
    DirectReadOnlyDb,
    StatelessHttp,
    StatefulTcp,
    NatsAsync,
}

#[must_use]
pub fn allowed_avenues(cfg: &RuntimeConfig) -> &'static [Avenue] {
    match cfg.role {
        ServiceRole::Web | ServiceRole::AdminWeb => &[
            Avenue::DirectReadOnlyDb,
            Avenue::StatelessHttp,
            Avenue::StatefulTcp,
            Avenue::NatsAsync,
        ],
        ServiceRole::Api | ServiceRole::AdminApi | ServiceRole::Worker => &[
            Avenue::StatelessHttp,
            Avenue::StatefulTcp,
            Avenue::NatsAsync,
        ],
    }
}
