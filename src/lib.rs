//! # fiducia-lib-core
//!
//! Shared runtime primitives for the `fiducia-cloud` organization. Contracts
//! live in `fiducia-interfaces`; persistence lives in `fiducia-orm-core`.
//! Backend-neutral fenced lock identities live here so servers do not invent
//! independent lock namespaces or recursively depend on Fiducia as the only
//! lease authority.

#![forbid(unsafe_code)]

pub mod config;
pub mod locks;

#[path = "../isomorph/mod.rs"]
pub mod isomorph;

#[cfg(feature = "client")]
#[path = "../client/mod.rs"]
pub mod client;

#[cfg(feature = "server")]
#[path = "../server/mod.rs"]
pub mod server;

#[cfg(feature = "edge")]
#[path = "../edge/mod.rs"]
pub mod edge;

pub use config::{
    ConfigError, PrivateConfig, PublicConfig, RuntimeConfig, RuntimeTarget, SecretValue,
    ServiceRole,
};
