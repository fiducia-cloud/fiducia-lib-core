//! Server-only primitives (api/web/admin servers, workers, lambdas). Has access to `PrivateConfig`.
//!
//! Persistence goes through `fiducia-orm-core`; this module never opens a database connection itself.

use crate::config::{RuntimeConfig, ServiceRole};

/// Which of the four web<->api avenues a server may use (see my-ai AGENTS.md):
/// 1. direct read-only db query, 2. stateless HTTP, 3. stateful TCP, 4. NATS/mq async.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Avenue {
    DirectReadOnlyDb,
    StatelessHttp,
    StatefulTcp,
    NatsAsync,
}

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
