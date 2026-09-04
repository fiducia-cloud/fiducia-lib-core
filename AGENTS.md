# Agent rules — fiducia-lib-core

- Read and follow `ORESoftware/my-ai/AGENTS.md` (the fleet-wide rules) before changing this repository.
- Organization: `fiducia-cloud` (Fiducia). Sibling repositories follow the standard `fiducia-*` layout
  (`-interfaces`, `-lib-core`, `-orm-core`, `-clients`, `-sync`, `-flutter`, `-desktop-app.rs`,
  `-lambdas`, `-infra`, `-web-server.rs`, `-api-server.rs`, `-admin-*-server.rs`, `-monorepo`).
- This repository owns shared **client / server / edge / isomorph** runtime primitives, the single
  validated `RuntimeConfig`, provider contracts (`platform-integrations.json`), and common domain logic.
- `fiducia-interfaces` owns the TypeSpec and JSON Schema authorities; `fiducia-orm-core` owns all ORM,
  schema, and migration code and is **private to the backend**. Never add ORM or schema ownership here.
- Every deployable Fiducia runtime reads configuration through `RuntimeConfig` and the root
  `.cli-flags.toml` (flags-2-env). Do not read ad hoc environment variables in service crates.
- Public/private configuration split: `PublicConfig` may reach browsers, Flutter, and edge workers;
  `PrivateConfig` never leaves the server tier and is always `SecretValue`-wrapped.
- Shared providers are exact packages, never local substitutes: `shared-auth`, `opto-sync`,
  `oresoftware/ores-middleware`, `ores-otel`, `ores-rate-limit`, `flags-2-env`, `ores-sops`,
  `declarative-migrations`, `zed-pkg`. Resolve them with `zed install --frozen` and commit `.zpkg.lock`.
- Rust first for systems and scripting work. No webviews or React in native apps.
- Never migrate a database at process boot. Never commit, log, or print credentials.
- Resolve cross-repository changes in dependency order: interfaces, lib-core, orm-core,
  clients/sync/services/lambdas, infra, then monorepo.
- Use focused branches and reviewed pull requests. Resolve merge conflicts semantically
  (merge both sides conceptually, never pick a side blindly), then grep for conflict markers.
