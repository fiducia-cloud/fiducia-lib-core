//! Edge-runtime primitives (Cloudflare Workers via `fiducia-infra` / `ores-middleware`).
//! Edge code never holds a database URL; it talks to the HTTPS data API only.

/// Header used to carry the request id from the edge into the origin (ores-otel correlation).
pub const REQUEST_ID_HEADER: &str = "x-ores-request-id";
