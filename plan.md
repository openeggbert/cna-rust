# CNA-Rust measured implementation plan

Status date: 2026-08-22

Strict target: XNA 4.0 Windows runtime Rust API projection

MSRV: Rust 1.74

## Current verified state

The supplied workspace began with 321 lines of Rust. Its Cargo dependency did
not resolve because package `cna-rust-sys` was requested as `cna-sys` without
an explicit package rename. The only raw declaration was
`BINDINGS_AVAILABLE = false`; `run` always returned `NativeUnavailable`; math,
graphics, input, and content contained apparent placeholder implementations.

The package/crate identities now are deliberate:

- package `cna-rust`, library `cna`;
- package `cna-rust-sys`, library `cna_sys`;
- dependency key `cna-sys` explicitly names package `cna-rust-sys`.

The workspace now has a real, safe native 2D slice and measured incomplete XNA
surface. It must not be described as XNA-complete.

## Compatibility definition and XNA to Rust mapping

- [x] Preserve authoritative XNA namespace/type/member/field casing wherever
  Rust permits it; Rust style is subordinate in the strict hierarchy.
- [x] Define the normative language mapping in `docs/xna-rust-mapping.md` and
  its executable subset in `tools/api-compat/mapping-rules.json`.
- [x] Keep crate-root host functions and `cna::extensions` out of the strict
  member comparison.
- [x] Keep the allowlist empty.
- [ ] Extend compiler comparison to every signature, generic, enum value,
  event/delegate, base/trait, ref/out, and disposal relation.

## Definition of done

The selected profile is complete only when all mapped types/members and all
currently unmeasured structural categories reach zero, raw/internal leaks stay
zero, ABI and behavior corpora pass, lifetime stress passes, the canonical CNA
library builds unmodified, 60/600-frame tests pass on every claimed platform,
and a freshly generated independent template builds and runs.

## Strict API verifier baseline

Authoritative reference assemblies: the seven exact SHA-256-pinned Windows
runtime assemblies in `tools/api-compat/profiles/xna40-windows-runtime.json`.

| Measurement | Current |
|---|---:|
| XNA reference types | 257 |
| XNA reference members | 2,964 |
| expected mapped Rust types (including 2 support types) | 259 |
| actual strict Rust types | 26 |
| total reviewed diagnostics | 1,066 |
| missing types | 233 |
| missing members | 833 |
| unexpected types | 0 |
| unexpected members | 0 |
| type-kind mismatches | 0 |
| internal type leaks | 0 |
| public unsafe APIs | 0 |
| allowlist entries | 0 |

Base/trait, parameter, return, generic, ref/out, enum, delegate/event, and
disposal comparisons are explicitly unmeasured, not zero. The normal verifier
exits nonzero while the 1,066 real findings remain.

## Workspace and package architecture

- [x] Fix Cargo identities and retain Rust 1.74.
- [x] Split raw declarations and safe projection at the existing two-crate
  boundary; avoid aesthetic crates.
- [x] Keep XNA and CNA-extension surfaces distinct.
- [ ] Complete `cargo package` contents, per-crate README/NOTICE review,
  docs.rs policy, versions, and release dependency versions.

## Native C ABI status and cna-sys

Canonical source: CNA `modules/c-api/include/CNA/C`, ABI 0.7.0. It contains 55
headers and 2,861 function declarations. `cna-sys` currently reviews 26
function-pointer declarations plus the required constants/structures/callbacks.

- [x] Verify all 26 names and arities against all headers.
- [x] Verify all 26 against a Linux ELF library and exact ABI version.
- [x] Add 14 `repr(C)` structure layout tests.
- [ ] Bind and audit the remaining ABI by coherent facade group.
- [ ] Add C-side generated layout/enum/bool/callback probes; current layout
  tests only assert the reviewed Rust expectations.
- [ ] Add PE and Mach-O export verification when those platforms exist.

The matching test library reports 2,861 ELF `cna_*` exports; zero bound symbols
are missing and zero arities mismatch. An available prebuilt ABI 0.8 library is
correctly rejected by the loader.

## Unsafe boundary and ownership/lifetime

- [x] Confine unsafe operations to `native.rs` and `game.rs` callback/FFI
  trampolines with stated invariants.
- [x] Deny `unsafe_op_in_unsafe_fn`.
- [x] Prevent raw pointer, native handle, and `cna_sys` leaks in safe public API.
- [x] Implement callback-borrowed `GraphicsDevice` and owned
  `Texture2D`/`SpriteBatch`.
- [x] Make explicit `Dispose` plus `Drop` idempotent.
- [x] Exercise explicit disposal followed by drop and double unload in native
  60/600-frame runs.
- [ ] Add isolated ownership-state types (`Owned`, `Borrowed`, `ParentOwned`,
  `Adopted`) as more resource families require them.
- [ ] Add failure-injection, parent/child order, repeated create/destroy,
  teardown callback, game shutdown, and process shutdown stress tests.

## Core and value API

- [x] Add signed tick-based `TimeSpan` and correct `GameTime` property names.
- [x] Replace identity/no-op matrix placeholders with real initial
  multiplication and common creation operations.
- [x] Establish Copy/value semantics and operator traits for the initial vector,
  quaternion, matrix, color, point/rectangle, plane/ray, and bounding subset.
- [ ] Complete the authoritative members for MathHelper, Vector2/3/4, Matrix,
  Quaternion, Color, Point/Rectangle, Plane/Ray, bounds/frustum, curves, and
  packed vectors.
- [ ] Build and pass the neutral XNA differential corpus including NaN,
  infinity, signed zero, rounding, and exception cases.

## Game, device, and window

- [x] Establish `Game` lifecycle trait plus callback-scoped `GameContext`.
- [x] Run actual CNA initialization, callbacks, update/draw, exit, cleanup, and
  shutdown.
- [ ] Complete the strict `Game` contract: services, content, window,
  components, properties, events, and `Run` mapping.
- [ ] Implement GameWindow, components/collections/services,
  GraphicsDeviceManager, GraphicsAdapter, and PresentationParameters as one
  lifecycle-tested group.

## Graphics

- [x] Implement real clear, viewport, encoded `Texture2D`, SpriteBatch begin /
  draw / end, and renderer-query extension paths.
- [x] Model `GraphicsResource` and `Texture` base relationships as traits.
- [ ] Complete Texture variants, render targets, buffers, declarations,
  effects/BasicEffect, states, sprite overloads, and parent-borrowed reflected
  objects.
- [ ] Verify resize/window behavior with a non-headless backend.

## Input and touch

- [x] Implement real native keyboard capture and pure `KeyboardState` queries.
- [ ] Complete authoritative `Keys`, keyboard constructors/queries, mouse,
  gamepad, player index, and touch with transitions and enum tests.

## Content

- [x] Remove the fake `ContentManager::Load<T>() -> Err(())` implementation;
  the namespace is visibly unimplemented.
- [x] Keep raw encoded image loading distinct from XNB.
- [ ] Define `ContentLoadable`, implement manager caching/unload and XNB reader
  tables, versions, shared resources, built-in/custom readers, and normalized
  lookup.

## Models

- [ ] Implement Model and collections using stable indices/parent ownership,
  with no self-referential structures or parent handle leaks.

## Audio and XACT

- [ ] Implement the measured profile using actual CNA ABI, with safe callback
  and dynamic-buffer lifetimes. Record native multi-listener or other blockers
  rather than faking behavior.

## Media, storage, and GamerServices

- [ ] Project the full selected profile. Historical external services may use
  deterministic unsupported errors, but types/members remain verifier
  requirements.

## CNA extensions

- [x] Put actual renderer facts under `cna::extensions::graphics`.
- [x] Remove inferred/fake renderer name and capability data.
- [ ] Add further extensions only when they correspond to real CNA concepts.

## Template

- [x] Replace the Rust-only three-frame loop with real `--smoke-test` (60),
  `--stability-test` (600), and `--frames N` CNA runs.
- [x] Exercise lifecycle, game time, native device, real PNG decode,
  Texture2D, SpriteBatch, clear/draw, keyboard, movement, capability query,
  explicit dispose, and shutdown.
- [x] Remove fake cube/3D and renderer claims.
- [ ] Add window resize and non-headless renderer evidence.
- [ ] Make project naming/package metadata parameterized through a tested
  cargo-generate-compatible or equivalently standard generation path.
- [ ] Replace the local path dependency with a released crate version when one
  exists.

## Platform matrix

| Platform | Classification |
|---|---|
| Linux x86-64 HEADLESS | experimental; 60/600 native frames passed with two temporary CNA compile-gate corrections |
| Linux windowed/GPU | planned; no CNA-Rust runtime evidence |
| Windows | planned; loader and runtime unverified |
| macOS | planned; runtime unverified |
| WebAssembly | unsupported; no compatible upstream CNA C ABI build verified |
| Android | unsupported; no native lifecycle/window/input integration verified |

## CI and quality gates

- [x] Rust 1.74 `fmt`, check, Clippy, tests, and docs.
- [x] Strict API report, leak guard, header/library ABI report.
- [x] Template build and 60/600 Linux headless runtime tests.
- [ ] Add CI-owned XNA reference acquisition policy and native artifact.
- [ ] Add canonical unmodified CNA build gate and sanitizers.
- [ ] Test the declared MSRV in CI, not only locally.

## Upstream CNA blockers

The checked-out CNA source does not build `cna_c_api` unmodified: C++ added the
50th `NanoVg` renderer, but ABI 0.7 `graphics.h` and both C mapping tables still
end at the 49th `PixiJs` identity. The compile-time guards correctly fail. The
runtime test library used two equivalent temporary `/tmp` corrections: preserve
the ABI 0.7 table size and map the post-ABI C++ `NanoVg` value to unknown. These
must be fixed and versioned upstream before release runtime support is claimed.

CNA ABI 0.7 teardown also checks for C-owned children before its shutdown path
invokes `unload_content`. CNA-Rust supplies a pre-destroy cleanup call and keeps
cleanup idempotent; the upstream ordering should be reviewed.

## Later profiles

- [ ] Separately inventory GamerServices/network assemblies not in the first
  seven-assembly runtime set, Xbox, Windows Phone, and content-pipeline/build
  assemblies.
- [ ] Consider FNA, MonoGame, and CNA extension profiles only after the strict
  XNA Windows runtime structure is healthy. Never merge them into a random
  union under the strict namespace.
