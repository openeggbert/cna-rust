# CNA Rust implementation plan

**Status:** XNA namespace scaffold in place

**Date:** 2026-08-16

## Phase 0 — repository scaffold

- [x] Keep `cna-sys` as the raw ABI crate and `cna` as the safe crate.
- [x] Establish `cna::Microsoft::Xna::Framework` plus `Graphics`, `Input`, and
      `Content` compatibility modules.
- [x] Keep binding-specific errors and runner utilities at crate root.
- [x] Remove the invalid invented `cna::CNA::Framework` module tree.

## Phase 1 — canonical ABI

- [ ] Generate and audit `cna-sys` only from headers owned by `openeggbert/cna`.
- [ ] Add layout/link tests, version checks, UTF-8, errors, callbacks, ownership,
      thread affinity, and shutdown.

## Phase 2 — playable compatibility slice

- [ ] Add graphics device, texture, sprite batch, content, and keyboard types.
- [ ] Run a CNA-backed XNA-style game loop.

## Invariants

1. XNA types follow the `Microsoft::Xna::Framework` hierarchy.
2. No public module is invented without a native counterpart.
3. CNA C++ remains canonical and only its stable C ABI crosses the boundary.
