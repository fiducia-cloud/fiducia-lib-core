# fiducia-lib-core

Shared runtime library for the `fiducia-cloud` (Fiducia) organization — the one place for validated
configuration, provider contracts, and common domain logic used by every Fiducia runtime.

```
isomorph/   runs everywhere (browser, edge, server, desktop)     feature: isomorph (default)
client/     browsers, Flutter FFI, desktop app, CLI                feature: client
server/     api/web/admin servers, workers, lambdas                feature: server
edge/       Cloudflare Workers and other edge runtimes             feature: edge
src/        crate root + RuntimeConfig (public/private split)
```

## Ownership boundaries

- `fiducia-interfaces` owns the TypeSpec and JSON Schema authorities.
- `fiducia-orm-core` owns Diesel/SeaORM projections, schema SQL, and migration generation. It is
  **private to the backend**; nothing in `client/` or `edge/` may depend on it.
- This crate owns `RuntimeConfig` (`.cli-flags.toml` via flags-2-env), the public/private
  configuration split, `SecretValue` redaction, and the shared-provider contract in
  `platform-integrations.json` (shared-auth, opto-sync, ores-middleware, ores-otel,
  ores-rate-limit, ores-sops, declarative-migrations).

## Runtime configuration

`RuntimeConfig::from_env()` is the process boundary; `RuntimeConfig::from_map()` is deterministic.
`PublicConfig` may reach browsers/Flutter/edge; `PrivateConfig` never leaves the server tier and
wraps secrets in `SecretValue` (always redacted in `Debug`). Edge targets are refused a direct
database URL; API/worker roles require one.

## Commands

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
zed install --frozen   # pins the shared providers listed in platform-integrations.json
```
