# CNA Rust implementation plan

**Status:** foundation scaffold in place

**Date:** 2026-08-16

**Sources:** `../cnabinding/analysis_binding.md`,
`../cnabinding/analysis_binding_sharp_runtime.md`, and
`../cna/analysis_binding_languages.md`

## Goal

Provide audited raw declarations in `cna-sys` and a safe, idiomatic `cna` crate
over CNA's canonical C++ engine. Preserve CNA/XNA concepts while using Rust
ownership, borrowing, RAII, `Result`, and local value types.

## Phase 0 — repository scaffold (this commit)

- [x] README, plan, architecture, license, notices, editor settings, ignores.
- [x] Dependency-free Cargo workspace with raw and safe crates.
- [x] Safe `Game` trait, `CnaError`, `GameTime`, `Vector2`, and `Color`.
- [x] Explicit native-unavailable behavior and no guessed raw ABI declarations.
- [x] Unit tests for the first local values.

## Phase 1 — canonical `cna-sys`

- [ ] Wait for ABI headers and implementation in `openeggbert/cna`.
- [ ] Generate/audit raw declarations and exact layout/link tests from those
      headers; keep build discovery configurable and diagnostics clear.
- [ ] Validate ABI version before use and retrieve structured native errors.
- [ ] Test UTF-8, nulls, stale/generation-mismatched handles, callback context,
      symbol visibility, wrong ABI versions, and shutdown order.

## Phase 2 — safe playable API

- [ ] Turn every owned handle into a non-cloneable RAII type with `Drop`; model
      borrowed handles with lifetimes and document `Send`/`Sync` decisions.
- [ ] Bridge the `Game` trait without aliasing `&mut` state across re-entrancy.
- [ ] Add graphics device, textures, sprite batching, content, and input.
- [ ] Run HelloGame: clear, load/draw a texture, read Escape, cleanly exit.

## Phase 3 — packaging and performance

- [ ] Batch SpriteBatch commands, cache lookups, and transfer slices in bulk.
- [ ] Support system and bundled-native discovery without silently mixing ABIs.
- [ ] Test Linux and Windows plus sanitizers on native boundary tests.
- [ ] Publish pre-1.0 paired crates only after the sample works end to end.

## Phase 4 — broader CNA/XNA concepts

- [ ] Complete Rust-local math, geometry, color, and input values.
- [ ] Add audio, fonts, effects, render targets, models, and 3D incrementally.
- [ ] Validate real games and publish an honest compatibility matrix.

## Invariants

1. CNA C++ stays canonical and only the CNA C ABI enters Rust.
2. `cna-sys` reflects canonical headers; `cna` contains all safety policy.
3. C++ exceptions and Sharp Runtime types never cross the boundary.
4. Ownership, thread affinity, re-entrancy, and callback roots are explicit.
5. Math stays local; input uses snapshots; high-frequency traffic batches.
