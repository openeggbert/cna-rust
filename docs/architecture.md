# Architecture

## Layers and public identity

```text
Rust game
  -> cna::Microsoft::Xna::Framework::*   strict XNA projection
  -> private family modules              value/input/game/graphics
  -> crate-private native bridge         typed safe calls over dynamic symbols
  -> cna_sys                             raw ABI 0.7 declarations
  -> CNA stable C ABI                    cna_* only
  -> CNA C++
```

The published package is `cna-rust` and its library crate is `cna`. The raw
package is `cna-rust-sys` and its crate is `cna_sys`. Filesystem organization
does not create public namespaces: private `value`, `input`, `game`,
`graphics`, and `native` modules are re-exported into the exact XNA hierarchy.
CNA-only renderer and opaque-window concepts remain under `cna::extensions`.

## Native boundary and ABI evidence

`cna-sys` contains the reviewed ABI-0.7 slice: fixed-width aliases, `repr(C)`
structures, callbacks, constants, and 104 function-pointer declarations. The
safe bridge is grouped by concern:

- `native/api.rs`: exact version gate and symbol inventory;
- `native/loader.rs`: dynamic-library ownership and Unix loading;
- `native/game.rs`, `graphics.rs`, `display.rs`, `window.rs`, and `input.rs`:
  typed facade calls;
- `native/fault.rs`: feature-gated, test-only failure injection;
- `native/error.rs`: CNA error extraction.

The loader accepts exactly `0x00000700`; it does not silently accept ABI 0.8.
There is no fake backend fallback. Unix is runtime-tested and unsupported
loaders return a typed error.

The ABI verifier derives full C prototypes from Clang's view of canonical CNA
headers and compares them with every reviewed `cna-sys` function type. It
measures return and parameter types, scalar width/signedness, pointer depth and
constness, callback/struct pointers, and boolean/enum representations. The
current pass checks 104 functions and 388 prototype type positions. Independent
C and Rust probes add layouts for 19 structures, three callback signatures,
and 129 constants, for 419 total C/Rust measurements and zero mismatches.

## Durable device identity

The game host creates one private `Arc<DeviceState>` for its logical
`GraphicsDevice`. Every `GameContext.GraphicsDevice()` clone points to that
same state, so repeated access and resource association preserve identity.
The `Arc` does not own CNA's native device: the native game remains its sole
owner.

Only the native device handle is callback-scoped. At callback entry the bridge
borrows CNA's current device handle into the private state; at callback exit it
clears that handle. Safe device operations outside a callback therefore return
a deterministic error instead of extending the native borrow or fabricating a
`'static` lifetime. No `transmute`, leak, public raw handle, or untracked
integer identity is used.

`Texture2D`, `SpriteBatch`, and bound graphics-state descriptors retain a clone
of the durable device wrapper. This supports same-device validation and shared
invalidation without giving a child ownership of the native device. Stable
shared slots also retain one logical `PresentationParameters`, adapter,
texture/sampler collection, and graphics-state object per device. Repeated
property access therefore aliases observable mutable state rather than creating
unrelated wrappers. `PresentationParameters.Clone` creates an independent
managed snapshot.

The host keeps weak registrations for owned native children and releases live
children in reverse registration order before parent destruction. It
invalidates the device after CNA finishes its shutdown callbacks and the
destroy attempt returns, including a reported destroy failure. After shutdown,
device/resource access fails safely and repeated child `Dispose` remains
idempotent. If CNA reports an untracked texture handle, the safe collection
returns an explicit unsupported error rather than inventing a second owner.

## Game object graph

Each host owns one `Arc<GameState>` containing its component collection,
typed service container, launch parameters, window state, game/device events,
timing flags, and durable device state. The public `Game` trait composes this
state through `GameStateAccess`; state is per game and no global service or
component registry exists.

`GameComponentCollection` stores shared trait objects and uses stable sorted
snapshots for update/draw traversal. Equal order preserves registration order;
changing order affects the next snapshot. Removal during traversal cannot
invalidate the current snapshot, and a component added after game
initialization is initialized immediately. Component and collection event
registries snapshot subscriptions, support removal/self-removal, and contain
handler panic.

`GameServiceContainer` uses Rust `TypeId` as the mapped CLR type token and
stores shared `Arc<dyn Any + Send + Sync>` provider identities. Duplicate
registration is rejected, missing lookup/removal is deterministic, and the
same provider identity is returned.

## Lifecycle and teardown

`Game` is the user lifecycle trait and `GameContext<'callback>` exposes the
host-owned portion needed during callbacks. CNA ABI 0.7's frame hooks now drive
`Initialize`, `BeginRun`, `BeginDraw`, `EndDraw`, and `EndRun` in addition to
the original content/update/draw/shutdown callbacks. A measured one-frame run
has this user-visible order:

```text
Initialize
LoadContent
BeginRun
Update
BeginDraw
Draw
EndDraw
OnExiting
EndRun
UnloadContent
Device Disposing
Dispose
Disposed event
```

`BeginDraw == false` suppresses `Draw` and `EndDraw`; the next update/frame can
still proceed. Panic in either ordinary lifecycle callbacks or `BeginDraw` is
caught before returning through C and becomes `CnaError::Callback`.

CNA ABI 0.7 rejects `cna_game_destroy` while owned child handles remain, but
CNA itself emits the shutdown lifecycle during destruction. The host therefore
performs an internal child-release pass after `cna_game_run` and before
`cna_game_destroy`. That pass invokes neither user `UnloadContent` nor public
resource `Disposing` events. CNA then emits exactly one `OnExiting`, one
`EndRun`, and one `UnloadContent`; only afterward does the host invalidate the
device and emit device disposal, invoke user `Game.Dispose`, and emit the game
disposed event. This order keeps the callback-scoped device borrow valid during
the CNA-supplied `UnloadContent`. User lifecycle notification and native
dependency cleanup are separate mechanisms, so normal `UnloadContent` code
need not be idempotent merely because of the host.

Activated/deactivated and client-size/orientation/screen-device events attach
to real CNA callbacks and detach before native destruction. HEADLESS supplies
no artificial transition. Reset/lost and resource-created/destroyed device
events are safe registries but deliberately do not emit without a real backend
transition or a stable safe wrapper for the reported native resource handle.

The current `RunOneFrame` entry executes one owned host session. Arbitrary
repeated XNA-style ticking remains blocked by CNA retaining the creation-time
callback context: the safe binding cannot let that pointer outlive a borrowed
Rust game or rebind it through the current ABI. It reports this limitation
instead of manufacturing a `'static` borrow.

## Graphics resource foundation

`GraphicsResource` is the shared safe trait for device association, name, tag,
disposed state, disposal, finalization projection, string representation, and
the normative `Disposing` event. Event dispatch snapshots subscription order,
supports removal and self-removal, catches handler panic, and never unwinds
into native code. Explicit `Dispose(true)` emits while the resource is still
observable as not disposed, then releases it exactly once. `Drop` and internal
pre-destroy cleanup do not synthesize the user event.

`Texture` composes `GraphicsResource`. `Texture2D` completes the selected
profile's mapped constructors, stream overloads, bounds/format/level metadata,
generic full/rectangle/mip transfers, PNG/JPEG encoding, and disposal. Data
routes accept only layouts CNA ABI 0.7 can represent exactly; they validate
dimensions, mip/rectangle/window bounds, format/type compatibility, disposed
parents/children, bad streams, and construction rollback.

`SpriteBatch` implements all texture draw overloads and the non-effect
state-bearing begin routes. Its remaining eight diagnostics are two
effect-bearing `Begin` overloads and six `SpriteFont.DrawString` overloads;
those wait for real `Effect` and `SpriteFont` families rather than shallow
signature shells. Invalid begin/draw/end/dispose transitions are tested.

Managed `BlendState`, `DepthStencilState`, `RasterizerState`, and
`SamplerState` descriptors have complete XNA properties, defaults, stock
states, resource behavior, and real CNA application routes. CNA and XNA assign
opposite C numeric identities to `BlendFunction.Min`/`Max`; the private bridge
performs an explicit translation while the public XNA enum values remain exact.

## Verification and fault evidence

The API verifier hashes all seven XNA 4.0 Windows runtime assemblies, extracts
neutral CLR metadata with Mono, applies the normative mapping, and inspects
compiler-produced rustdoc JSON. Schema 2 measures every declared structural
category and emits the deterministic type scoreboard. `unmeasuredCategories`
and the allowlist are empty; the leak-only public-surface gate is zero.

The XNA-derived managed corpus has 123 named observations plus a final count
assertion (124 assertions total). New groups in this run cover authoritative
component/service behavior and `PresentationParameters` defaults/clone
semantics. The native suite uses isolated child processes for 146 game
lifetime cycles and 103 resource constructions, repeated dispose/drop,
live-child parent shutdown, stable persistent identities, component mutation,
callback and event-handler panic, transfer validation, and test-bridge
create/info/reported-destroy failures. Absence of a crash is not a leak proof.
`tools/native-stress/run-sanitized.sh` is an optional path requiring a
separately instrumented exact ABI-0.7 CNA library; no sanitizer pass is claimed
for the current run.

## Current blockers and dependency order

Canonical CNA HEAD `1bb2145d99ed572dd4eb15009c34e2e5f410fcf0` still fails
its unmodified C API build at `CnaCApiCoreExt.cpp:250`: the renderer identity
assertion reduces to `49 == 50`. Runtime evidence therefore uses the clearly
labelled experimental ABI-0.7 HEADLESS library.

The remaining strict work is dependency-ordered, not optimized for missing
type count: add the content foundation for the last two `Game` members; solve
safe repeated frame hosting; add vertex/index buffers, draw routes, and render
targets; then real effects and `SpriteFont`, models, broader XNB/content,
audio/XACT, and media/storage. PNG decoding remains a texture route, not
content-pipeline support.
