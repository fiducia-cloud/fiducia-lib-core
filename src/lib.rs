//! # fiducia-lib-core
//!
//! Shared runtime primitives for the `fiducia-cloud` (Fiducia) organization, split into the four
//! standard folders:
//!
//! | folder      | feature    | may run in                                   |
//! | ----------- | ---------- | -------------------------------------------- |
//! | `isomorph/` | `isomorph` | everywhere (browser, edge, server, desktop)  |
//! | `client/`   | `client`   | browsers, Flutter FFI, desktop, CLI          |
//! | `server/`   | `server`   | api/web/admin servers, workers, lambdas      |
//! | `edge/`     | `edge`     | Cloudflare Workers / edge runtimes           |
//!
//! Persistence lives in `fiducia-orm-core` (private to the backend). Contracts live in
//! `fiducia-interfaces`. This crate never owns schema or migrations.

#![forbid(unsafe_code)]

pub mod config;

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
