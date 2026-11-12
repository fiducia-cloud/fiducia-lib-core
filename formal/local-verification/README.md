# Full local verification — 2026-11-12

Source commit: 9dc68d7eb060eac57bfddb2a28e4cf471df32b9f.

Executed with Rust/Cargo 1.90.0: **11 all-feature tests; documentation target contains zero tests passed**, no failures or ignored tests. Source and dependency files were read at exact commits through the GitHub connection and checked against every recorded Git blob SHA. See snapshot-results.json for the complete provenance and local log digests.

The staging Cargo manifests redirect git dependencies to those verified local paths. Fiducia also requires the pinned transitive logging package. The logging workspace was narrowed to its consumed Rust SDK for this staging build. No implementation source or production dependency pin was changed. The saved lockfile describes this local path-resolution graph and must not replace the production root lockfile without a separate dependency review.

Reproduction: fetch the recorded revisions; verify blob identities; redirect only the listed Git dependencies to their matching package paths in disposable manifest copies; use the saved lockfile in that staging root; run cargo test --locked --all-targets --all-features and cargo test --locked --doc --all-features.

Hosted runtime-refinement still requires its configured dependency-read secret. This report records executed local full-crate tests, not a green hosted job or cryptographic execution attestation.
