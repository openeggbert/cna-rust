# CNA Rust implementation plan

**Status:** corrected namespace scaffold in place

**Date:** 2026-08-16

## Phase 0 — namespace scaffold

- [x] Keep `cna-sys` as the raw ABI crate and `cna` as the safe crate.
- [x] Establish `cna::CNA::Framework` and
      `cna::Microsoft::Xna::Framework` public module roots.
- [x] Reserve matching `Graphics`, `Input`, and `Content` modules.
- [x] Add initial `Game`, `GameTime`, `Vector2`, and `Color` shapes.

## Phase 1 — canonical ABI

- [ ] Generate and audit `cna-sys` only from headers owned by `openeggbert/cna`.
- [ ] Add layout/link tests, ABI-version checks, UTF-8, structured errors,
      callbacks, ownership, thread affinity, and shutdown.

## Phase 2 — first playable XNA-style loop

- [ ] Add safe graphics device, texture, sprite batch, content, and keyboard
      types under both public module trees.
- [ ] Run a CNA-backed game that clears, loads/draws a texture, reads Escape,
      and shuts down cleanly.

## Invariants

1. Public module hierarchy follows CNA and `Microsoft::Xna::Framework`.
2. CNA C++ remains the only engine implementation.
3. `cna-sys` is raw; `cna` owns every safety and ownership guarantee.
4. Sharp Runtime and C++ ABI details remain native implementation details.
