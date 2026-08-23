# Session evidence and next work

## 2026-08-23 — Game/Graphics persistent-object run

### Strict result

```text
reference XNA types                257 -> 257
reference members                2964 -> 2964
expected mapped Rust types         259 -> 259
actual strict Rust types            91 -> 117

total diagnostics                  263 -> 178
missing types                      168 -> 142
missing members                     95 -> 36

constructor / overload / property   2 / 18 / 30 -> 1 / 17 / 4
event mapping mismatches                      20 -> 0
```

Parameter/signature, return, base/trait/interface, generic/bound, ref/out,
enum/value, flags, delegate, disposal, unexpected surface, type-kind,
internal/raw-handle/public-unsafe leaks, allowlist, and unmeasured-category
counts all remain zero. Normal strict mode exits 1 only for the 178 genuine
remaining gaps; leak-only exits 0.

```text
Game                 36 -> 2
GraphicsDevice       51 -> 26
SpriteBatch           8 -> 8
```

All 26 new component/service/window/presentation/display/collection dependency
types have zero local diagnostics. `Game` waits only for its real content
manager. `GraphicsDevice` waits for buffers/draw/reset/render-target and
back-buffer transfer routes. `SpriteBatch` remains unchanged because its two
`Effect` and six `SpriteFont` dependencies were not replaced with placeholders.

Fresh missing-type families are Graphics 63, Media 24, Audio 19, Design 13,
Content 12, Framework/core 4, Input 3, Storage 3, and GamerServices 1.

### Game object model and lifecycle

The implementation now has real per-game component, service, launch-parameter,
window, lifecycle-state, and event objects. Components initialize before the
run and immediately when added after initialization. Update/draw snapshots are
stable for equal order, order changes affect subsequent snapshots, and removal
during a snapshot does not invalidate iteration. Enabled/visible/order events
support ordered subscription, removal, and self-removal; handler panic is
contained.

`GameServiceContainer` uses per-instance type tokens and returns the same
shared provider identity. Duplicate add, missing get/remove, and invalid
providers are deterministic. No global service registry exists.

The measured native one-frame shutdown order is:

```text
Initialize
LoadContent
BeginRun
Update
BeginDraw
Draw
EndDraw
Exiting event / OnExiting
EndRun
UnloadContent
Device Disposing
Game Dispose
Game Disposed event
```

User `UnloadContent` occurs exactly once. The host first performs an internal
reverse-order release of registered native children because ABI 0.7 rejects
parent destruction with live children. That internal operation emits neither
user `UnloadContent` nor public resource events. CNA destroy then emits the
user shutdown callbacks; shared device invalidation and public game disposal
follow. A discovered early-invalidation regression was fixed so the CNA
`UnloadContent` callback still has a valid callback-scoped device borrow.

`Activated`, `Deactivated`, and the three window events are subscribed through
real CNA routes and safely detached before destruction. HEADLESS does not
fabricate platform transitions. Repeated arbitrary `Game.RunOneFrame`/`Tick`
is still a precise host/ABI blocker: CNA retains a creation-time callback
context, so a borrowed game cannot safely outlive one host session or be
rebound by the current ABI.

### Graphics identity and lifecycle

One host-owned `Arc<DeviceState>` is the durable logical `GraphicsDevice`.
Resources and persistent properties share that identity but never own the
native device. The CNA handle remains callback-scoped. Shutdown invalidates the
shared state deterministically; access after parent shutdown fails safely and
resource `Dispose` remains idempotent.

Repeated access preserves logical identity for `Game.GraphicsDevice`,
`PresentationParameters`, `Adapter`, textures, samplers, vertex textures,
vertex samplers, and graphics-state properties. Presentation refresh mutates
the shared object in place; `Clone` is independent. Texture/sampler collections
validate bounds and device association. A CNA-reported texture handle without
a tracked safe wrapper returns explicit unsupported behavior instead of
creating a second native owner.

Resource-created/destroyed events remain safely non-emitting because CNA's raw
handle callbacks cannot reconstruct stable safe wrapper/tag identity. Lost and
reset events likewise require a real backend transition; HEADLESS state is not
treated as such. `GraphicsAdapter.MonitorHandle` is explicitly unsupported
where CNA cannot expose a safe opaque identity.

### Behavior, ABI, and safety evidence

```text
XNA-derived observations          105 -> 123
assertions including count        106 -> 124
failures                                    0

reviewed ABI functions             53 -> 104
prototype type positions          188 -> 388
total C/Rust measurements         313 -> 419
layouts / callbacks / constants   19 / 3 / 129
mismatches                                  0

native game lifetime cycles       143 -> 146
native child handles               93 -> 103
native crashes                              0
observed double-free/UAF                    0
```

New corpus groups cover component/service behavior and authoritative XNA
`PresentationParameters` defaults/clone semantics. Native stress additionally
covers stable property identities, multiple resources sharing a device,
texture/sampler slots, disposed parents, component ordering/mutation,
device/game events, callback/event-handler panic plus recreation, double
dispose/drop, live-child cleanup, and injected create/info/reported-destroy
failures.

All 104 reviewed functions have compiler-derived return/parameter prototypes,
including pointer depth, constness where representable, scalar width and
signedness, callback/struct pointer types, and bool/enum/flag representation.
ABI tested is exactly `0x00000700`.

No sanitizer was executed. An exact ABI-0.7 sanitized CNA artifact could not be
produced while canonical CNA HEAD still fails at
`CnaCApiCoreExt.cpp:250` with renderer identity `49 == 50`. Canonical revision
is `1bb2145d99ed572dd4eb15009c34e2e5f410fcf0`; it was not modified.

### Template and quality gates

Linux x86-64 experimental CNA HEADLESS template tests and fresh 60/600-frame
runs pass. The canary retains real PNG, `Texture2D`, texture `SpriteBatch`,
keyboard/mouse/gamepad, clean shutdown, and now verifies per-game service
identity. A fresh generated consumer vendors both binding crates, contains no
developer/sibling path, builds/tests, and completes 60 frames.

Rust 1.74 format, check, workspace all-feature tests, docs, and Clippy exit
zero. New genuine Clippy findings (unchecked enum/index conversions, trait
object identity comparison, needless closure/borrow, and fallible cache setup)
were fixed. Narrow compatibility allowances remain for exact XNA naming,
mapped high-arity overloads, exact numerical behavior, and the native stress
driver.

### Next coherent slice

1. Implement real Content/XNB ownership sufficient for `Game.Content`.
2. Solve repeated `RunOneFrame`/`Tick` with an owned/rebindable callback
   context or a reviewed CNA ABI addition.
3. Add vertex declarations and buffers with safe bound-buffer destruction,
   then drawing, render targets, reset, and back-buffer transfer.
4. Implement real `Effect` ownership/reflection/execution before the two
   effect-bearing `SpriteBatch.Begin` overloads.
5. Implement genuine `SpriteFont` state, measurement, and atlas rendering
   before the six draw-string overloads.
6. Execute optional ASan/UBSan only against an exact ABI-0.7 instrumented CNA
   artifact; do not infer leak freedom from crash absence.
