# Shared limiter implementation refinement

The production limiter and error type live in `server/rate_limiter.rs` and are re-exported from `server::platform`, preserving the public API. Axum still calls the same elapsed-time admission method. The transition and mutex-protected update are unchanged.

Run `cargo test --features server --lib server::platform:: -- --nocapture` with access to the existing pinned private dependencies. This includes the four pre-existing platform tests and the two limiter tests.

The new tests exercise 16 rounds of 16 simultaneous cloned handles against a budget of three, rejection before the window boundary, admission at the next window, and fail-closed behavior after lock poisoning. These are executed implementation checks, not an exhaustive schedule proof.

For an isolated local check, compile the exact pinned ores-rl-lib-core revision `31df606d26280611e2d2d684328a804644eb4255` with its default (empty) feature set as an rlib. Compile a Rust test wrapper that includes the unchanged rate_limiter.rs as a child module and links that rlib. This checks the production limiter but excludes Axum and the four platform integration tests; it cannot certify the full server build.
