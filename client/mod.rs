//! Client-side primitives (browsers, Flutter FFI, desktop, CLI). May only see `PublicConfig`.

use crate::config::PublicConfig;

/// Everything a client needs to bootstrap: no secrets, safe to embed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientBootstrap {
    pub api_base_url: String,
    pub auth_issuer: String,
    pub auth_audience: String,
}

impl From<&PublicConfig> for ClientBootstrap {
    fn from(p: &PublicConfig) -> Self {
        Self {
            api_base_url: p.public_base_url.clone(),
            auth_issuer: p.auth_issuer.clone(),
            auth_audience: p.auth_audience.clone(),
        }
    }
}
