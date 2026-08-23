# Architecture

## Layers and public identity

```text
Rust game
  -> cna::Microsoft::Xna::Framework::*   strict XNA projection
  -> private family modules              value/input/game/content/graphics
  -> crate-private native bridge         typed safe calls over dynamic symbols
  -> cna_sys                             raw ABI 0.7 declarations
  -> CNA stable C ABI                    cna_* only
  -> CNA C++
```

The published package is `cna-rust` and its library crate is `cna`. The raw
package is `cna-rust-sys` and its crate is `cna_sys`. Filesystem organization
does not create public namespaces: private `value`, `input`, `game`, `content`,
`graphics`, and `native` modules are re-exported into the exact XNA hierarchy.
CNA-only renderer, reflection-construction, collection-index, and opaque-window
concepts remain under `cna::extensions`.

## Native boundary and ABI evidence

`cna-sys` contains the reviewed ABI-0.7 slice: fixed-width aliases, exact
semantic handle typedefs, `repr(C)` structures, callbacks, constants, and 431
function-pointer declarations. The
safe bridge is grouped by concern:

- `native/api.rs`: exact version gate and symbol inventory;
- `native/loader.rs`: dynamic-library ownership and Unix loading;
- `native/game.rs`, `graphics.rs`, `display.rs`, `window.rs`, `input.rs`,
  `device_manager.rs`, and `storage.rs`: typed facade calls;
- `native/fault.rs`: feature-gated, test-only failure injection;
- `native/error.rs`: CNA error extraction.

The loader accepts exactly `0x00000700`; it does not silently accept ABI 0.8.
There is no fake backend fallback. Unix is runtime-tested and unsupported
loaders return a typed error.

The ABI verifier derives full C prototypes from Clang's view of canonical CNA
headers and compares them with every reviewed `cna-sys` function type. It
measures return and parameter types, scalar width/signedness, pointer depth and
constness, callback/struct pointers, and boolean/enum representations. The
current pass checks 431 functions and 1,509 prototype type positions.
Independent C and Rust probes make 936 measurements across 56 structures, five
callback signatures, scalar representations, and 243 constants, with zero
mismatches.

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

Textures, sprite objects, buffers, render targets, Effects, Models, queries, and bound
graphics-state descriptors retain a clone of the durable device wrapper. This
supports same-device validation and shared invalidation without giving a child
ownership of the native device. Stable shared slots also retain one logical
`PresentationParameters`, adapter, texture/sampler collection, graphics-state
object, vertex/index binding set, and render-target set per device. Repeated
property access aliases observable mutable state rather than creating
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
timing flags, one stable `ContentManager`, and durable device state. The public
`Game` trait composes this state through `GameStateAccess`; state is per game
and no global service, content, or component registry exists.

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
core callback context. The later frame-hook table has an independently mutable
context but cannot rebind Update/Draw/content callbacks. The safe binding
cannot let that pointer outlive a borrowed Rust game and reports the limitation
instead of manufacturing a `'static` borrow. A safe CNA route must atomically
replace the full core callback table/context while idle and guarantee release
of the prior context.

## Device management, Touch, Storage, and GamerServices

One optional `GraphicsDeviceManagerState` is registered with each Game. It is
published under manager and device-service `TypeId`s, retains the durable
Game-owned device wrapper, caches pre-run preferences, and owns a CNA manager
plus six event registrations only during the native session. It never owns or
constructs a second device. Native callbacks use a non-owning public manager
wrapper as sender; callback panic is recorded and returned after CNA reaches a
safe Rust operation boundary. Shared `GraphicsDeviceInformation` proposals
preserve CLR reference identity, while explicit XNA Clone deep-copies mutable
presentation state. CNA lacks candidate ranking, so that protected operation
is an explicit backend error.

`TouchPanel` is static at the XNA surface but takes the active
`GameContext<'_>` under the normative mapping. The bridge borrows the current
Game handle only for the call and copies capabilities, touch locations, and
gesture samples into managed values. It neither owns a registration nor
recognizes gestures in Rust.

Storage is independent of Game and has strict native ownership:
`StorageDevice -> StorageContainer -> StorageStream`. Begin operations invoke
CNA synchronously and return a concrete one-shot result retaining state and
origin identity. Every filesystem and stream operation remains behind CNA;
`std::fs` is not a backend. Rust validates XNA child-path containment before
native dispatch because ABI 0.7's `RelativePath` copies valid UTF-8 but does
not enforce all traversal rules. Container Dispose closes streams, observes
CNA's synchronous Disposing event exactly once, then unregisters/destroys in
child-first order.

`GamerServicesComponent` composes the existing `GameComponent` state and
participates in ordinary initialize/update/order/enabled/dispose behavior. No
Gamer, Guide, Avatar, network, achievement, or leaderboard service is in the
selected profile or fabricated by the component.

## Managed Design converters

Design is a pure managed layer over the existing XNA value types. The twelve
concrete converters compose the shared `MathTypeConverterBase` contract; no
converter, property descriptor, culture, or reconstruction operation crosses
the CNA ABI. The strict namespace contains only the thirteen XNA converter
types. Crate-root `DesignType`, `DesignCulture`, `DesignValue`, ordered
property records, and executable reconstruction descriptors form the compact
Rust support vocabulary documented by the normative mapping.

Metadata is static and immutable. Property extraction produces an ordered
owned snapshot; nested Vector3 values are copied. Creation consumes an ordered
slice but performs explicit name lookup, so caller ordering cannot alter the
result and no hash iteration becomes observable. Reconstruction descriptors
select one of twelve closed constructor identities rather than retaining a
reflection object or resolving a dynamic symbol. The value union prevents
arbitrary `Any`, handles, raw pointers, and implementation objects from
crossing the public converter boundary.

Culture is explicit and contains only information the XNA IL actually uses:
decimal separator, list separator, and the three special Single symbols. The
format path intentionally implements XNA Windows legacy Single text instead
of inheriting Rust `Display`, while parsing targets `f32` directly. Six
converters accept component strings; the other six preserve XNA's unsupported
input behavior and use the inherited value-string fallback for output.

## Content and XNB

`ContentManager` owns an `Arc<ContentManagerInner>` with its service provider,
root/source, case-insensitive typed cache, ordered unique-disposable registry,
operation lock, and disposed state. A per-game manager receives the durable
device identity but never owns the native Game. `Unload` removes cache entries
and disposes unique resources in reverse registration order. It continues
after a failure and retains failed disposables so later `Dispose` can retry.
Partial reader failures roll back only resources recorded by that load.

`ContentReader` implements XNA's managed XNB layer: Windows header/version/
flags/size validation, compressed and uncompressed payload framing, seven-bit values, reader
table and versions, root reader index, existing instances, shared fixups,
external references, and trailing-data validation. The global registry stores
typed descriptors rather than a general application `Any` loader. Custom
readers follow the same activation/table path as built-ins. Primitive/value
readers remain pure Rust. Texture, SpriteFont, buffer, stock-effect, and Model
readers call the same private native bridge and ordinary resource constructors
used outside Content.

SpriteFont XNB ownership is one graph: the font retains the Texture2D atlas,
ContentManager records the font disposable identity, and SpriteBatch only
borrows both during glyph submission. The atlas is not independently recorded
for a second destruction. Effect XNB similarly uses the normal reader table;
the current HEADLESS backend rejects compiled effect bytecode with explicit
capability error 6, which is preserved as the inner `ContentLoadException`.
Model XNB uses the normal shared-resource table for VertexBuffer, IndexBuffer,
and Effect references. A private descriptor finalization hook publishes the
graph only after shared fixups resolve; failure rolls back each resource
recorded during the load. Model is registered last, so reverse unload
invalidates retained graph facades before effects and buffers are destroyed.
Texture3D and TextureCube readers admit only the reviewed exact Color encoding.

For compressed XNB v5, the 14-byte header supplies the exact decompressed
payload size. The frame layer accepts XNA's two-byte short header and
`0xff`-selected extended header, retains one 64 KiB LZX decoder across frames,
and requires exact frame output, exact total output, and an exact end or legal
zero marker/padding. The decoder implements verbatim, aligned, and
uncompressed blocks plus the complete XNA Huffman/repeated-offset/window
behavior; it does not apply CAB's optional Intel transform. Decompression
finishes before the ordinary reader graph begins, so fixups, external
references, rollback, cache identity, Unload, and reload use one unchanged
pipeline.

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

`Texture3D` follows the same resource and typed-transfer model. It validates
dimensions, complete mip counts, 3D boxes, start/count windows, exact element
encodings, disposal, and device lifetime before CNA dispatch. The qualified
HEADLESS renderer rejects volume storage at construction with error 6; the
binding preserves this backend boundary and does not claim native transfer
execution. `TextureCube` now also participates in the built-in XNB table.

`VertexDeclaration` is managed and retains exact XNA stride/elements. Built-in
vertices have verified C-compatible layouts; custom `VertexData` explicitly
encodes each field so Rust padding is never transferred. Vertex/index buffers
own native CNA handles and retain the device association. The device owns only
logical bindings. It refuses to destroy a bound buffer, unbinds during parent
shutdown, and forwards dynamic `Discard`/`NoOverwrite` without semantic
collapse.

DrawUser, bound, indexed, and instanced routes validate primitive counts,
vertex/index windows, declarations, strides, bindings, disposal, and device
identity before native dispatch. HEADLESS proves native submission by returning
the renderer's exact missing-applied-effect result; it does not prove visible
3D output. Back-buffer readback likewise reports the explicit HEADLESS
non-rasterizing error without mutating the destination.

`RenderTarget2D`, `RenderTargetCube`, and `RenderTargetBinding` retain their
underlying native resource and device identity. Set/Get routes preserve stable
logical binding identity only after CNA succeeds and reject duplicate,
disposed, wrong-device, or incompatible target sets first. HEADLESS may reject
a target operation as unsupported; no binding success is fabricated.

`Effect` owns one native effect handle and durable device state. Annotation,
parameter, pass, technique, and collection wrappers own only their CNA view
handles while retaining the parent ResourceState; they can never destroy the
Effect. Collections cache wrappers by native handle so repeated lookup is
stable. Parameter routes are typed and textures retain safe tracked wrappers.
`EffectPass.Apply` is a real CNA call. CNA's empty-effect tooling starts with a
default technique, and its manual-graph clone can return an empty native graph;
extension-created Effects retain a typed blueprint and rebuild only for that
exact observed fallback. Compiled Effects always use the native clone path.

BasicEffect, AlphaTestEffect, DualTextureEffect, EnvironmentMapEffect, and
SkinnedEffect each own their distinct CNA stock-effect handle; they are not
modeled as an unrelated manual Effect graph. A private common wrapper shares
only genuine resource/technique behavior. Texture properties retain safe
tracked texture identities. DirectionalLight handles are stable parent-owned
views that share the parent's ResourceState, are destroyed once by parent
cleanup, and fail after parent disposal. IEffectFog, IEffectLights, and
IEffectMatrices delegate directly to concrete stock-effect state.

`Model` is a managed owner of stable bone, mesh, mesh-part, collection, and
resource facades. Strong edges run from Model to its graph. Bone parent and
mesh-part sibling back-links use `Weak`, and the shared lifetime has no facade
back-link, so the graph is collectable. Shared buffers and effects retain one
native owner. Model.Draw walks the public graph, sets matrices through the
IEffectMatrices-capable stock-effect implementations, rejects an incompatible
generic Effect, binds the part buffers, applies each pass, and calls the
ordinary indexed GraphicsDevice route; there is no native Model shortcut.

`OcclusionQuery` owns one query handle and enforces idle/active/ended/disposed
transitions before using CNA create, begin, end, completion, pixel-count, and
destroy operations. PixelCount is never synthesized by Rust.

`SpriteFont` owns a native font handle and retains exactly one atlas. Its XNB
reader validates parallel glyph/crop/character/kerning tables and constructs
the native font. Measurement, spacing/default-character mutation, and glyph
submission use real CNA routes.

`SpriteBatch` now implements every selected texture and DrawString overload and
both Effect-bearing state Begin overloads. It validates state, parent device,
Effect/font/atlas disposal, and marks itself active only after native Begin
succeeds. An Effect is never accepted and ignored. Invalid begin/draw/end/
dispose transitions and recovery after a failed Begin are tested.

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

The XNA-derived managed corpus has 185 named observations plus a final count
assertion (186 assertions total). The 40-observation Design group covers
capabilities, ordered metadata/extraction, component cultures, malformed and
wrong inputs, creation, Matrix's Translation asymmetry, nested snapshots, and
all reconstruction identities. The earlier platform-neutral groups cover
Gesture flags/samples and device-information default/reference/clone behavior. The
native suite uses isolated child processes for 209 created game lifetimes and
1,012 owned child-handle constructions, including ten buffer binding cycles,
ten SpriteFont atlas/content cycles, ten Effect parent/child cycles, ten
complete compressed Model/XNB cycles, ten stock-effect/Texture3D/
OcclusionQuery cycles, ten combined Framework/Touch/Storage/GamerServices
cycles, repeated dispose/drop, live-child parent shutdown, stable identities,
callback self-removal/panic/recreation, transfer validation, and injected
failures. Absence of a crash is not a leak proof.
`tools/native-stress/run-sanitized.sh` is an optional path requiring a
separately instrumented exact ABI-0.7 CNA library. Sanitizer status is
`not-run`; no sanitizer pass is claimed for the current run.

## Current blockers and dependency order

Canonical CNA HEAD `1bb2145d99ed572dd4eb15009c34e2e5f410fcf0` still fails
its unmodified C API build at `CnaCApiCoreExt.cpp:250`: the renderer identity
assertion reduces to `49 == 50`. Runtime evidence therefore uses the clearly
labelled experimental ABI-0.7 HEADLESS library.

Graphics, Framework/core, Input, Storage, GamerServices, and Design have zero
missing types. Every remaining strict diagnostic is a whole missing type in
Audio (19) or Media (24); each is a separate future milestone requiring its
own regenerated dependency/ownership review. LZX is complete for XNA 4.0
Windows framing. Repeated frame hosting still needs a CNA core-callback-context
rebinding route. PNG decoding remains a texture route, not an alias for XNB
content.
