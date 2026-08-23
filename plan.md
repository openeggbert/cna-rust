# CNA-Rust measured implementation plan

Status date: 2026-08-23

Strict target: XNA 4.0 Windows runtime Rust API projection

MSRV: Rust 1.74

## Governing constraints

- Preserve authoritative XNA identifiers and casing below
  `cna::Microsoft::Xna::Framework`.
- Keep implementation modules private, the compatibility allowlist empty, and
  every structural zero as a permanent regression gate.
- Complete dependency/ownership families with real behavior before adding
  broad unrelated types.
- Expose only safe facades over reviewed, exact ABI-0.7 CNA routes.

The generated `typeScoreboard` is the authoritative work queue. This file
records the final measured state and dependency order; it does not maintain a
second hand-written contract inventory.

## Current strict API measurement

| Measurement | Run baseline | Current |
|---|---:|---:|
| XNA reference types | 257 | 257 |
| XNA reference members | 2,964 | 2,964 |
| expected mapped Rust types | 259 | 259 |
| actual strict Rust types | 91 | 117 |
| total diagnostics | 263 | 178 |
| missing types | 168 | 142 |
| missing members | 95 | 36 |
| constructor mapping mismatches | 2 | 1 |
| overload mapping mismatches | 18 | 17 |
| property mapping mismatches | 30 | 4 |
| event mapping mismatches | 20 | 0 |

Parameter/signature, return, base projection, trait, interface, generic,
generic-bound, ref/out, enum/value, flags, delegate, disposal, unexpected type,
unexpected member, type-kind, internal-type leak, raw-handle leak, public
unsafe, allowlist, and unmeasured-category counts are all zero.

The 142 missing types are grouped from the fresh generated report as:

```text
Graphics          63
Media             24
Audio             19
Design            13
Content           12
Framework/core     4
Input               3
Storage             3
GamerServices       1
```

## Current three-type scoreboard

| Strict type | Run baseline | Current |
|---|---:|---:|
| `Game` | 36 | 2 |
| `GraphicsDevice` | 51 | 26 |
| `SpriteBatch` | 8 | 8 |

`Game` now waits only for `Content` and `SetContent`, which require the real
`ContentManager`. `GraphicsDevice` waits for its primary constructor plus
buffer binding/data, render targets, reset/back-buffer transfer, and drawing
routes. `SpriteBatch` deliberately still waits for two real `Effect`-bearing
`Begin` overloads and six real `SpriteFont` draw-string overloads.

Every new dependency type introduced in this run has zero local diagnostics:

- Game/core: `DisplayOrientation`, `IGameComponent`, `IUpdateable`,
  `IDrawable`, `GameComponent`, `DrawableGameComponent`,
  `GameComponentCollection`, `GameComponentCollectionEventArgs`,
  `GameServiceContainer`, `LaunchParameters`, `GameWindow`,
  `FrameworkDispatcher`, and `TitleContainer`.
- Graphics: `DepthFormat`, `DisplayMode`, `DisplayModeCollection`,
  `GraphicsAdapter`, `GraphicsDeviceStatus`, `GraphicsProfile`,
  `PresentInterval`, `PresentationParameters`, `RenderTargetUsage`,
  `ResourceCreatedEventArgs`, `ResourceDestroyedEventArgs`,
  `SamplerStateCollection`, and `TextureCollection`.

## Completed in this run

- [x] Regenerated the strict work queue and dependency-family inventory.
- [x] Added the complete selected-profile component/service/window declaration
  family with stable component ordering, mutation snapshots, change events,
  initialization timing, per-game typed services, window managed state, and
  real CNA lifecycle/window subscriptions where the backend exposes them.
- [x] Completed all currently representable `Game` lifecycle/state/event/run
  members except its content-manager property pair.
- [x] Preserved exactly one user `UnloadContent` and corrected shutdown order:
  internal child release precedes CNA destroy; CNA emits `Exiting`, `EndRun`,
  and the sole `UnloadContent`; device invalidation and public disposal follow.
- [x] Extended the durable `Arc<DeviceState>` graph with stable shared
  `PresentationParameters`, adapter, texture/sampler collection, and graphics
  state identities. Repeated access aliases the same logical state.
- [x] Added real CNA query/apply routes for presentation/display/window/device
  state and safe real event subscriptions. Unsupported HEADLESS transitions
  are never fabricated.
- [x] Kept `Effect` and `SpriteFont` pending instead of adding signature-only
  placeholders, so `SpriteBatch` remains behaviorally honest.
- [x] Expanded full C-prototype verification to every newly reviewed ABI route.

## Behavior, ABI, safety, and template evidence

| Measurement | Run baseline | Current |
|---|---:|---:|
| named XNA-derived observations | 105 | 123 |
| corpus assertions including count | 106 | 124 |
| reviewed ABI functions | 53 | 104 |
| prototype type positions | 188 | 388 |
| total C/Rust ABI measurements | 313 | 419 |
| prototype/layout/callback/constant mismatches | 0 | 0 |
| native game lifetime cycles exercised | 143 | 146 |
| native child handles constructed | 93 | 103 |
| native crashes observed | 0 | 0 |

New behavior groups cover component/service semantics and
`PresentationParameters` defaults/clone independence. The corpus passes all
123 observations and 124 assertions. Native stress covers component ordering
and mutation, lifecycle and event-handler panic containment, stable device and
collection identity, resource association, disposed-parent behavior, child
cleanup, fault injection, repeated disposal, and recreation.

Linux x86-64 experimental ABI-0.7 HEADLESS template tests and fresh 60/600
frame runs pass. A fresh generated consumer vendors both crates, contains no
developer path, builds/tests, and completes 60 frames. It exercises real PNG,
`Texture2D`, texture `SpriteBatch`, input, per-game service identity, and clean
shutdown.

Canonical read-only CNA HEAD
`1bb2145d99ed572dd4eb15009c34e2e5f410fcf0` still fails an unmodified C API
build at `modules/c-api/src/CnaCApiCoreExt.cpp:250`; the renderer identity
assertion is `49 == 50`. The Rust loader remains exact ABI 0.7. No exact
ABI-0.7 ASan/UBSan CNA artifact could be produced, so sanitizers remain
unmeasured and absence of a native crash is not reported as leak freedom.

## Next dependency-ordered work

1. Implement the real content foundation needed by `Game.Content` and
   `Game.SetContent`; PNG decoding remains separate from XNB.
2. Resolve repeated `Game.RunOneFrame`/`Tick` hosting without leaving CNA's
   retained callback context pointing at a borrowed Rust game. The present
   one-session host is safe but does not claim arbitrary repeated XNA ticks.
3. Add vertex declarations and vertex/index buffers with safe binding lifetime
   rules, then complete device draw routes and data transfers.
4. Add render targets and reset/back-buffer behavior only through real backend
   routes or explicit unsupported errors.
5. Implement the full `Effect` ownership/reflection/execution family before
   clearing the two effect-bearing `SpriteBatch.Begin` overloads.
6. Implement real `SpriteFont` state/measurement/rendering before clearing the
   six draw-string overloads.
7. Run optional ASan/UBSan verification when an exact instrumented ABI-0.7 CNA
   library can be built without changing canonical semantics.
8. Defer models, broad content/XNB, audio/XACT, media, and storage until these
   ownership dependencies are complete.

## Definition of complete compatibility

The selected profile is complete only when all mapped types and members and
every structural category reach zero; all public-surface safety gates remain
zero; behavior is XNA-derived; native prototypes/layouts are compiler-verified;
ownership and sanitizer evidence pass; canonical CNA builds unmodified; and
every claimed platform has fresh runtime evidence.
