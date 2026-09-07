# Independent schema-authority CI fixture

`main.tsp` and `authored.schema.json` are independently maintained peer authorities for this repository's parity-gate fixture. Neither file is generated from, ranked below, or allowed to overwrite the other.

CI generates JSON Schema B from TypeSpec only into `.typespec-json-schema-validator/positive/generated/`, validates both JSON Schema lanes as Draft 2020-12, compares top-level declarations and normalized semantics, and executes bidirectional instance probes. The generated witness and deterministic receipt are evidence only.

The same exact-head workflow also executes a negative mapping-integrity canary against the immutable validator merge commit `8584720715e4e90573535e14b16cb3a24c14ca63`:

- `stale.mapping.json` deliberately references the absent TypeSpec declaration `OreSchemaAuthority.MissingRegistration`;
- the validator must exit non-zero with report status `stopped_for_evaluation`;
- `check-stale-mapping-receipt.mjs` requires exactly one attributable `mapping-typespec-declaration-missing` finding with a stable lowercase SHA-256 fingerprint;
- a generic `run-failed` finding is rejected; and
- Git must report no mutation of either authored authority or the mapping fixture.

Both positive and negative receipts, SARIF projections, and generated comparison witnesses are retained separately. A negative lane that unexpectedly passes, fails without a receipt, emits the wrong rule, or changes an authored input fails the job.

A passing fixture proves this repository executes the fail-closed gate. It does not move product or domain schema ownership here and does not certify unrelated contracts. Fiducia product authorities remain in `fiducia-cloud/fiducia-interfaces`; any unexplained mismatch stops evaluation and blocks promotion.
