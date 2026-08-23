# Session evidence and next work

## 2026-08-23 — lifecycle/graphics foundation run

### Result

This run cleared all five pre-existing structural mismatches and completed the
durable device/resource foundation before broad API expansion. Curves, every
remaining packed-vector type, graphics resources, texture base behavior,
`Texture2D`, and the managed graphics-state group now have zero local strict
diagnostics. `SpriteBatch` is reduced to dependencies on real `Effect` and
`SpriteFont` families; those types were not replaced with shells.

```text
reference XNA types               257 -> 257
reference members                2964 -> 2964
expected mapped Rust types        259 -> 259
actual strict Rust types           51 -> 91

total diagnostics                 364 -> 263
missing types                     208 -> 168
missing members                   151 -> 95
parameter/signature mismatches      3 -> 0
disposal mismatches                 2 -> 0
```

All base/trait/interface, return, generic/bound, ref/out, enum/value, flags,
delegate, unexpected surface, unsafe/internal/raw-handle, allowlist, and
unmeasured counts remain zero. Current overlapping missing-category counts are:

```text
constructor / overload / property       2 / 18 / 30
event / parameter / disposal           20 / 0 / 0
```

The six requested scoreboards are exact:

```text
Game                 42 -> 36
GraphicsDevice       72 -> 51
GraphicsResource     11 -> 0
Texture               1 -> 0
Texture2D            14 -> 0
SpriteBatch          16 -> 8
```

Normal strict mode exits 1 for the remaining 263 real gaps. Report-only records
the same scoreboard and leak-only exits 0 with no findings.

### Behavior and lifecycle

The neutral XNA-derived corpus grew from 82 to 105 named observations (106
assertions including the count gate). New groups are curve (10), packed vector
(7), and graphics-state defaults (6). It found and fixed half conversion,
tie-even quantization, curve loop/tangent, and CNA `BlendFunction.Min/Max`
translation errors. Final corpus failures are zero.

The real CNA one-frame lifecycle order is:

```text
Initialize
LoadContent
BeginRun
Update
BeginDraw
Draw
EndDraw
Exiting
EndRun
UnloadContent
Dispose
```

`BeginDraw=false` was also measured: two begin attempts produced exactly one
`Draw` and one `EndDraw`. User `UnloadContent` occurs exactly once. Before CNA
destroy, the host privately releases registered native children in reverse
order because ABI 0.7 rejects a live-child parent destroy. That internal pass
does not invoke `UnloadContent` or public resource events. At CNA shutdown,
resources are already disposed, then CNA sends `Exiting`, `EndRun`, and the one
user `UnloadContent`; Rust sends `Dispose` after the destroy attempt.

### Device and resource ownership

`GraphicsDevice` now wraps a stable `Arc<DeviceState>`. Repeated
`GameContext.GraphicsDevice` access and every child association share that
identity. Only the private CNA device handle is borrowed at callback entry and
cleared at callback exit. Resources share identity/validity, not native device
ownership; the game is the sole native owner.

The host invalidates the shared device after parent destruction, including a
reported failure. Device and child access after shutdown fail safely; child
`Dispose` after shutdown is idempotent. Tests cover multiple resources on one
device, same-device draws through distinct wrappers, invalidation, live-child
shutdown, and disposal after parent shutdown.

`GraphicsResource.Disposing` covers registration order, removal, self-removal,
single emission, event-visible state, double dispose, `Drop`, and contained
handler panic. The internal parent cleanup deliberately emits no user event.

### Native ABI and fault evidence

```text
reviewed ABI functions                 34 -> 53
full prototype functions checked           53
prototype type positions                  188
all C/Rust ABI measurements           135 -> 313
layout types / callbacks / constants   14 / 2 / 98
prototype/probe mismatches                     0
missing header/library symbols                 0
ABI tested                               0x00000700
```

Full compiler-derived prototypes now cover return types, parameter count,
width/signedness, pointer depth/constness, callback and structure pointer types,
and bool/enum/flag representation for every reviewed route.

The isolated suite passes 143 successful game lifetimes and constructs 93
native child handles. It covers double `Dispose`, `Dispose` plus `Drop`, active
batch disposal, live children at shutdown, partial texture rollback, a
contained callback panic plus recreation, injected game-create failure,
texture-information failure plus recreation, and a reported game-destroy
failure plus recreation. Native crashes and observed double-free/use-after-free
events are zero. No sanitizer was actually executed; `tools/native-stress/`
provides an opt-in runner for a separately ASan/UBSan-instrumented exact
ABI-0.7 CNA library.

### Canonical CNA and template

Canonical read-only CNA HEAD is
`1bb2145d99ed572dd4eb15009c34e2e5f410fcf0`. A clean unmodified HEADLESS C API
configure succeeds, but the build still stops at:

```text
modules/c-api/src/CnaCApiCoreExt.cpp:250
renderer identity assertion: 49 == 50
```

The CNA checkout was not modified. The loader remains exact ABI 0.7 and does
not accept ABI 0.8. Native evidence uses the experimental ABI-0.7 HEADLESS
artifact and remains labelled accordingly.

The Linux x86-64 template builds/tests and freshly completes 60 and 600
HEADLESS frames. It now demonstrates a real state-bearing
`SpriteBatch.Begin(Deferred, AlphaBlend)` route. A fresh generated consumer
vendors both binding crates, contains no developer/sibling path, builds/tests,
and completes 60 frames.

### Quality gates

Rust 1.74 format, check, workspace all-feature tests, docs, and Clippy all exit
zero. The Clippy audit reduced library warnings from 432 to 368 and the native
stress test warnings from 16 to two by fixing unchecked conversions, redundant
locks/closures, precedence, and temporary stock-state borrows. Remaining
warnings are chiefly exact XNA naming, exact float equality, intentional packed
numeric conversion, large differential/stress functions, and mapped overload
argument counts; they are not globally silenced.

### Next coherent slice

1. Implement components/services/window with their real events and ordering.
2. Finish the remaining `Game` state/events/run controls on that object model.
3. Establish stable state-collection, presentation-parameter, and adapter
   identities before buffers/render targets.
4. Add real `Effect`, then the two remaining `SpriteBatch.Begin` overloads;
   add real `SpriteFont`, then the six draw-string overloads.
5. Run the optional sanitizer path once an exact instrumented ABI-0.7 CNA
   artifact can be produced without hiding the canonical HEAD blocker.
6. Defer XNB/content, models, audio/XACT, and media/storage until those lifetime
   dependencies are complete.
