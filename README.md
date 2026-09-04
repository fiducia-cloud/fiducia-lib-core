# fiducia-lib-core

Shared runtime library for the `fiducia-cloud` organization—the one place for
validated configuration, request-serving policy, provider provenance, and
common non-persistence domain logic used by Fiducia runtimes.

```text
isomorph/   runs everywhere (browser, edge, server, desktop)   feature: isomorph
client/     browsers, Flutter FFI, desktop app, CLI             feature: client
server/     API/web/admin servers, workers, lambdas             feature: server
edge/       Cloudflare Workers and other edge runtimes          feature: edge
src/        crate root + RuntimeConfig (public/private split)
```

## Ownership boundaries

- `fiducia-interfaces` publishes the public TypeSpec and JSON Schema contract
  families and generated language interfaces. The two authoring lanes remain
  independent peers and discrepancies remain fail-closed.
- `fiducia-orm-core` owns SeaORM and Diesel projections, connection adapters,
  generated ORM SQL, runtime parity checks, and named database operations. It
  is backend-only. This crate does not open database connections or own ORM
  queries.
- `fiducia-lib-core` owns the shared request-serving profile: ORES middleware,
  ORES rate-limit policy, interface-version evidence, validated runtime roles,
  and the public/private configuration split.

## Standard Rust server profile

All four standard Rust servers—web, admin web, API, and admin API—enable the
`server` feature and call `fiducia_lib_core::server::install_axum`.

That function:

1. installs the portable lifecycle from `ORESoftware/ores-middleware`;
2. deliberately disables the middleware package's internal limiter through its
   audit adapter so there is exactly one quota authority;
3. installs the deterministic state machine from
   `ores-rate-limit/ores-rl-lib-core`;
4. binds the compiled profile to a generated `fiducia-interfaces` type;
5. preserves health/readiness/version probes outside admission quotas;
6. fails closed when middleware or limiter policy is invalid.

Default per-process fixed-window capacities are role-specific and intentionally
conservative: web 240/minute, API 120/minute, admin web/API 60/minute, worker
600/minute. They are a local admission boundary, not a claimed global quota.
A strict fleet-wide quota requires the rate-limit library's reviewed distributed
backend and separate deployment evidence.

## Runtime configuration

`RuntimeConfig::from_env()` is the process boundary;
`RuntimeConfig::from_map()` is deterministic. `PublicConfig` may reach browsers,
Flutter, and edge runtimes. `PrivateConfig` never leaves the server tier and
wraps secrets in `SecretValue`, whose `Debug` output is always redacted. Edge
targets are refused a direct database URL; API/worker roles require one.

## Immutable provider provenance

`platform-integrations.json` records the exact Git revisions compiled by the
server profile. `.zpkg.toml` declares the matching Zed package dependencies.
A release is not certified until `zed install --frozen` emits a reviewed lock;
the absence of that lock cannot be hidden by Cargo success.

## Validation

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
zed validate
zed install --frozen
```
