# Atomic read/decide/commit admission

Property: Atomic read/decide/commit admission.

## Scope and source mapping

Reviewed boundary: [server/platform.rs](https://github.com/fiducia-cloud/fiducia-lib-core/blob/139c012ef0c7570bd12495e04e20761d8b887c1c/server/platform.rs).

Two callers and one fixed window of capacity one. Models the mutex scope around SharedRateLimiter::decide_at. Does not execute ores-rate-limit or model refill, clock rollback, or cross-process limits. Mutation removes mutual exclusion.


This is a finite design model. No source extraction, implementation trace replay, solver proof, production execution, or unbounded theorem is claimed. Production refinement remains a separate obligation.

## Reproduce

Requires Rust 1.75 or newer; standard library only. From the repository root:

```sh
mkdir -p tmp/formal-safety
rustc --edition=2021 --test -D warnings formal/safety-interleavings/check.rs -o tmp/formal-safety/check
tmp/formal-safety/check --nocapture --test-threads=1
```

The checker explores the reachable graph to a fixed point. Each invariant is checked at initialization and every successor. Breadth-first traversal emits a shortest action trace on violation. Exceeding 200,000 states fails rather than reporting a partial exploration as proof. Positive tests pin exact state/edge counts to catch accidental loss of reachable behavior. Negative controls must reach an invariant violation; compiler errors, panics, and exhausted exploration are not accepted counterexamples.

## Executed evidence

Local rustc 1.75.0, warnings denied: two tests passed. Positive exploration: **9 states / 8 edges**. One unsafe mutant detected. Hosted CI is separate evidence and must execute actual steps before being called passed.

## Refinement work

Map observed implementation events into these transitions; replay successful and rejected operations; demonstrate the same counterexamples against controlled implementation mutations. Preserve these bounds and assumptions when reporting results. Passing this model alone must not promote runtime, database, compiler, or backend conformance status.
