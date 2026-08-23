# CNA-Rust measured implementation plan

Status date: 2026-08-23

Strict target: XNA 4.0 Windows runtime Rust API projection

MSRV: Rust 1.74

## Governing constraints

- Preserve authoritative XNA identifiers and casing under
  `cna::Microsoft::Xna::Framework`.
- Keep filesystem modules private and the compatibility allowlist empty.
- Treat every measured zero as a permanent gate; do not reclassify real gaps.
- Complete coherent behavior/ownership families before broad shallow types.
- Expose only safe facades over reviewed, exact ABI-0.7 CNA routes.

The generated `typeScoreboard` remains the detailed work queue. This plan
records measured truth and dependency order rather than duplicating metadata.

## Current strict API measurement

| Measurement | Requested-run baseline | Current |
|---|---:|---:|
| XNA reference types | 257 | 257 |
| XNA reference members | 2,964 | 2,964 |
| expected mapped Rust types | 259 | 259 |
| actual strict Rust types | 51 | 91 |
| total diagnostics | 364 | 263 |
| missing types | 208 | 168 |
| missing members | 151 | 95 |
| parameter/signature mismatches | 3 | 0 |
| disposal mismatches | 2 | 0 |
| unexpected types/members | 0 / 0 | 0 / 0 |
| type-kind mismatches | 0 | 0 |
| internal/raw-handle/public-unsafe leaks | 0 / 0 / 0 | 0 / 0 / 0 |
| allowlist entries | 0 | 0 |
| unmeasured categories | 0 | 0 |

All other structural mismatches remain zero: base, trait, interface, return,
generic, generic-bound, ref/out, enum/value, flags, and delegate. Current
overlapping missing-category counts are two constructors, 18 overloads, 30
properties, and 20 events.

### Six-type core scoreboard

| Strict type | Before | Current |
|---|---:|---:|
| `Game` | 42 | 36 |
| `GraphicsDevice` | 72 | 51 |
| `GraphicsResource` | 11 | 0 |
| `Texture` | 1 | 0 |
| `Texture2D` | 14 | 0 |
| `SpriteBatch` | 16 | 8 |

`SpriteBatch` waits only for two effect-bearing begin overloads and six
`SpriteFont` draw-string overloads. `Game` and `GraphicsDevice` remain the
nonzero lifecycle/graphics foundation types; their missing component/window,
event, presentation, collection, buffer, and draw dependencies are not hidden.

## Completed in this run

- [x] Fixed all three generated signature and both disposal mismatches, with
  verifier regression tests and no mapping relaxation.
- [x] Completed `Curve`, `CurveKey`, `CurveKeyCollection`, loop/continuity and
  tangent behavior with exact declarations and XNA-derived observations.
- [x] Completed all 17 selected-profile packed-vector formats, typed/untyped
  contracts, exact packing behavior, and differential tests.
- [x] Replaced callback-lifetime `GraphicsDevice` identity with a durable
  shared state whose native handle borrow remains callback-scoped.
- [x] Added resource registration, deterministic pre-parent child release,
  same-device validation, shared invalidation, and post-shutdown safety tests.
- [x] Completed `GraphicsResource`, its normative `Disposing` event, and the
  `Texture`/`Texture2D` base relationships at zero local diagnostics.
- [x] Completed all mapped `Texture2D` constructors, stream overloads,
  metadata, transfers, encoders, validation, rollback, and lifetime behavior.
- [x] Implemented every texture `SpriteBatch.Draw` overload and both
  non-effect state-bearing `Begin` routes with invalid-state tests.
- [x] Completed the managed blend/depth/rasterizer/sampler state families and
  exact enums/flags, stock defaults, resource semantics, and real CNA apply
  routes.
- [x] Deepened `GraphicsDevice` with viewport/scissor, blend factor,
  multisample mask, reference stencil, state setters, and no-argument present.
- [x] Added real `BeginRun`, `BeginDraw`, `EndDraw`, and `EndRun` frame hooks.
- [x] Separated one user `UnloadContent` from internal pre-destroy child
  cleanup and measured the callback order explicitly.
- [x] Expanded every reviewed function from symbol/arity checks to compiler-
  derived full C prototype comparison.
- [x] Added isolated test-bridge faults for create, texture information and
  reported destroy failure, plus an opt-in ASan/UBSan runner.

## Behavior and native evidence

| Measurement | Baseline | Current |
|---|---:|---:|
| named XNA-derived observations | 82 | 105 |
| corpus assertions including count | 83 | 106 |
| reviewed ABI functions | 34 | 53 |
| C/Rust ABI measurements | 135 | 313 |
| prototype functions/type positions | unmeasured | 53 / 188 |
| prototype/layout/callback/constant mismatches | 0 | 0 |
| successful native game lifetimes | 127 | 143 |
| native child handles constructed | not separately recorded | 93 |
| native crashes observed | 0 | 0 |

New managed groups are curve (10 observations), packed vectors (7), and
graphics-state defaults (6). The implementation work found and fixed XNA
behavior differences in half conversion, tie-to-even packing, curve loop/
tangent semantics, and the private XNA-to-CNA `BlendFunction.Min/Max`
translation. The corpus now passes with zero failures.

The native suite covers 100 minimal lifetimes, 25 double-dispose/drop resource
games, ten transfer/event games, lifecycle/device identity, a suppressed draw,
callback panic plus recreation, create failure, texture-info rollback plus
recreation, and reported destroy failure plus recreation. No sanitizer was
actually executed, so the run does not claim leak freedom from crash absence.

## Runtime and template evidence

- [x] Linux x86-64 experimental CNA ABI-0.7 HEADLESS template tests pass.
- [x] Fresh template runs complete 60 and 600 successful draw frames.
- [x] The canary uses real PNG decoding, `Texture2D`, state-bearing
  `SpriteBatch`, clear, keyboard/mouse/gamepad, and deterministic shutdown.
- [x] A fresh generated consumer vendors both crates, has no developer or
  sibling paths, builds/tests, and completes 60 HEADLESS frames.

Canonical CNA HEAD `1bb2145d99ed572dd4eb15009c34e2e5f410fcf0` still fails an
unmodified C API build at `CnaCApiCoreExt.cpp:250`; the renderer identity
assertion reduces to `49 == 50`. The Rust loader remains pinned to exactly ABI
0.7. Runtime evidence therefore remains experimental and does not imply an ABI
0.8 migration.

## Next dependency-ordered work

1. Implement the authoritative component/service/window group: `IGameComponent`,
   `IUpdateable`, `IDrawable`, component base types/collection/events,
   `GameServiceContainer`, `LaunchParameters`, and `GameWindow`.
2. Complete remaining `Game` properties, events, run-one-frame/tick/reset/
   suppress/exit semantics over that object model.
3. Resolve stable wrapper identity for device state collections,
   `PresentationParameters`, and adapter/profile/status before adding buffers.
4. Complete graphics device events and the buffer/vertex declaration group,
   then render targets and drawing routes using reviewed CNA prototypes.
5. Implement `Effect` coherently, then clear the two remaining state-bearing
   `SpriteBatch.Begin` overloads; implement real `SpriteFont` before the six
   draw-string overloads.
6. Produce and run an exact ABI-0.7 ASan/UBSan CNA artifact when the upstream
   build blocker permits it; keep sanitizer tooling optional for Rust 1.74.
7. Begin models and XNB/content only after the lifetime/identity graph above is
   complete. PNG decoding is never content-pipeline support.

## Definition of complete compatibility

The selected profile is complete only when all mapped types/members and every
structural category reach zero, all public-surface safety gates remain zero,
behavior is XNA-derived, native prototypes/layouts are compiler-verified,
ownership plus sanitizer evidence passes, canonical CNA builds unmodified, and
every claimed platform has fresh runtime evidence.
