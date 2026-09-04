//! Single validated runtime configuration boundary (flags-2-env `.cli-flags.toml`).
//!
//! `RuntimeConfig::from_env()` is the process boundary; `RuntimeConfig::from_map()` is
//! deterministic and testable. Configuration is split into a [`PublicConfig`] half that may be
//! shipped to browsers, Flutter, and edge workers, and a [`PrivateConfig`] half that never leaves
//! the server tier. No secret has a checked-in default.

use std::collections::HashMap;
use std::fmt;

/// A secret that is always redacted in `Debug` output.
#[derive(Clone, PartialEq, Eq)]
pub struct SecretValue(String);

impl SecretValue {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
    /// Expose the secret. Call sites should be few and reviewed.
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for SecretValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SecretValue(<redacted>)")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeTarget {
    Server,
    Edge,
    Lambda,
    Desktop,
    Cli,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServiceRole {
    Api,
    Web,
    AdminApi,
    AdminWeb,
    Worker,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConfigError {
    Missing(&'static str),
    Invalid { key: &'static str, reason: String },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Missing(k) => write!(f, "missing required configuration `{k}`"),
            ConfigError::Invalid { key, reason } => {
                write!(f, "invalid configuration `{key}`: {reason}")
            }
        }
    }
}

impl std::error::Error for ConfigError {}

/// Configuration that may be shipped to untrusted clients.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicConfig {
    pub public_base_url: String,
    pub auth_issuer: String,
    pub auth_audience: String,
    pub auth_jwks_url: String,
}

/// Configuration that must never leave the server tier.
#[derive(Clone, Debug)]
pub struct PrivateConfig {
    pub bind_addr: String,
    pub database_url: Option<SecretValue>,
    pub otel_exporter_endpoint: Option<String>,
    pub rate_limit_fail_closed: bool,
}

#[derive(Clone, Debug)]
pub struct RuntimeConfig {
    pub target: RuntimeTarget,
    pub role: ServiceRole,
    pub public: PublicConfig,
    pub private: PrivateConfig,
}

fn required<'a>(
    map: &'a HashMap<String, String>,
    key: &'static str,
) -> Result<&'a str, ConfigError> {
    match map.get(key).map(String::as_str) {
        Some(v) if !v.trim().is_empty() => Ok(v),
        _ => Err(ConfigError::Missing(key)),
    }
}

fn optional<'a>(map: &'a HashMap<String, String>, key: &str) -> Option<&'a str> {
    map.get(key)
        .map(String::as_str)
        .filter(|v| !v.trim().is_empty())
}

impl RuntimeConfig {
    /// Process boundary: read from the environment (as declared in `.cli-flags.toml`).
    pub fn from_env() -> Result<Self, ConfigError> {
        let map: HashMap<String, String> = std::env::vars().collect();
        Self::from_map(&map)
    }

    /// Deterministic constructor used by tests and by non-process hosts (edge, lambda).
    pub fn from_map(map: &HashMap<String, String>) -> Result<Self, ConfigError> {
        let target = match optional(map, "FIDUCIA_RUNTIME_TARGET").unwrap_or("server") {
            "server" => RuntimeTarget::Server,
            "edge" => RuntimeTarget::Edge,
            "lambda" => RuntimeTarget::Lambda,
            "desktop" => RuntimeTarget::Desktop,
            "cli" => RuntimeTarget::Cli,
            other => {
                return Err(ConfigError::Invalid {
                    key: "FIDUCIA_RUNTIME_TARGET",
                    reason: format!("unknown target `{other}`"),
                })
            }
        };
        let role = match optional(map, "FIDUCIA_SERVICE_ROLE").unwrap_or("api") {
            "api" => ServiceRole::Api,
            "web" => ServiceRole::Web,
            "admin-api" => ServiceRole::AdminApi,
            "admin-web" => ServiceRole::AdminWeb,
            "worker" => ServiceRole::Worker,
            other => {
                return Err(ConfigError::Invalid {
                    key: "FIDUCIA_SERVICE_ROLE",
                    reason: format!("unknown role `{other}`"),
                })
            }
        };

        let public = PublicConfig {
            public_base_url: optional(map, "FIDUCIA_PUBLIC_BASE_URL")
                .unwrap_or("http://localhost:8080")
                .to_string(),
            auth_issuer: required(map, "FIDUCIA_AUTH_ISSUER")?.to_string(),
            auth_audience: required(map, "FIDUCIA_AUTH_AUDIENCE")?.to_string(),
            auth_jwks_url: required(map, "FIDUCIA_AUTH_JWKS_URL")?.to_string(),
        };
        if !public.auth_issuer.starts_with("https://")
            && !public.auth_issuer.starts_with("http://localhost")
        {
            return Err(ConfigError::Invalid {
                key: "FIDUCIA_AUTH_ISSUER",
                reason: "must be https (or http://localhost for dev)".into(),
            });
        }

        let database_url = optional(map, "FIDUCIA_DATABASE_URL").map(SecretValue::new);
        let needs_db = matches!(target, RuntimeTarget::Server | RuntimeTarget::Lambda)
            && matches!(
                role,
                ServiceRole::Api | ServiceRole::AdminApi | ServiceRole::Worker
            );
        if needs_db && database_url.is_none() {
            return Err(ConfigError::Missing("FIDUCIA_DATABASE_URL"));
        }
        if matches!(target, RuntimeTarget::Edge) && database_url.is_some() {
            return Err(ConfigError::Invalid {
                key: "FIDUCIA_DATABASE_URL",
                reason: "edge runtimes must use the HTTPS data API, never a direct database URL"
                    .into(),
            });
        }

        let rate_limit_fail_closed =
            match optional(map, "FIDUCIA_RATE_LIMIT_MODE").unwrap_or("fail-closed") {
                "fail-closed" => true,
                "fail-open" => false,
                other => {
                    return Err(ConfigError::Invalid {
                        key: "FIDUCIA_RATE_LIMIT_MODE",
                        reason: format!("unknown mode `{other}`"),
                    })
                }
            };

        Ok(Self {
            target,
            role,
            public,
            private: PrivateConfig {
                bind_addr: optional(map, "FIDUCIA_BIND_ADDR")
                    .unwrap_or("127.0.0.1:8080")
                    .to_string(),
                database_url,
                otel_exporter_endpoint: optional(map, "FIDUCIA_OTEL_EXPORTER_ENDPOINT")
                    .map(str::to_string),
                rate_limit_fail_closed,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> HashMap<String, String> {
        [
            ("FIDUCIA_AUTH_ISSUER", "https://auth.fiducia.cloud"),
            ("FIDUCIA_AUTH_AUDIENCE", "fiducia"),
            (
                "FIDUCIA_AUTH_JWKS_URL",
                "https://auth.fiducia.cloud/.well-known/jwks.json",
            ),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
    }

    #[test]
    fn api_server_requires_database_url() {
        let err = RuntimeConfig::from_map(&base()).unwrap_err();
        assert_eq!(err, ConfigError::Missing("FIDUCIA_DATABASE_URL"));
    }

    #[test]
    fn web_role_does_not_require_database_url() {
        let mut m = base();
        m.insert("FIDUCIA_SERVICE_ROLE".into(), "web".into());
        let cfg = RuntimeConfig::from_map(&m).unwrap();
        assert_eq!(cfg.role, ServiceRole::Web);
        assert!(cfg.private.rate_limit_fail_closed);
    }

    #[test]
    fn edge_rejects_direct_database_url() {
        let mut m = base();
        m.insert("FIDUCIA_RUNTIME_TARGET".into(), "edge".into());
        m.insert("FIDUCIA_SERVICE_ROLE".into(), "web".into());
        m.insert("FIDUCIA_DATABASE_URL".into(), "postgres://x".into());
        assert!(matches!(
            RuntimeConfig::from_map(&m),
            Err(ConfigError::Invalid {
                key: "FIDUCIA_DATABASE_URL",
                ..
            })
        ));
    }

    #[test]
    fn secrets_are_redacted_in_debug() {
        let mut m = base();
        m.insert(
            "FIDUCIA_DATABASE_URL".into(),
            "postgres://user:pw@db/fiducia".into(),
        );
        let cfg = RuntimeConfig::from_map(&m).unwrap();
        let dbg = format!("{:?}", cfg);
        assert!(!dbg.contains("pw@db"));
        assert!(dbg.contains("<redacted>"));
    }
}
