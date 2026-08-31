# Architecture

## Layers and public identity

```text
Rust game
  -> cna::Microsoft::Xna::Framework::*   strict XNA projection
  -> private family modules              value/input/game/content/graphics/audio/media
  -> crate-private native bridge         typed safe calls over dynamic symbols
  -> cna_sys                             raw ABI 0.21 declarations
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

`cna-sys` contains the reviewed ABI-0.21 slice: fixed-width aliases, exact
semantic handle typedefs, `repr(C)` structures, callbacks, constants, and 1,326
function-pointer declarations. The
safe bridge is grouped by concern:

- `native/api.rs`: symbol inventory and the loading gate;
- `native/abi.rs`: the canonical ABI admission policy;
- `native/loader.rs`: dynamic-library ownership, Unix and Windows loading;
- `native/game.rs`, `graphics.rs`, `display.rs`, `window.rs`, `input.rs`,
  `device_manager.rs`, `storage.rs`, `audio.rs`, and `media.rs`: typed facade calls;
- `native/fault.rs`: feature-gated, test-only failure injection;
- `native/error.rs`: CNA error extraction, including the structured category.

The loader applies CNA's own versioning contract rather than a single constant
comparison: a different major is always rejected, an experimental `0.x` minor
must equal the reviewed one because CNA ships incompatible change as a minor
increment, a stable major admits a higher minor, and a higher patch is always
admitted. See [abi-migration-evidence.md](abi-migration-evidence.md).
There is no fake backend fallback. Unix is runtime-tested; Windows is
implemented in source over `LoadLibraryW`/`GetProcAddress`/`FreeLibrary` but no
Windows Rust target exists on this host, so it is neither compiled nor run
here. A target with neither loader returns a typed error rather than pretending
to load. The Windows path encoding goes through a helper compiled on every
host, so the mistake most likely to occur there -- an interior NUL truncating a
path, a missing terminator, or a lossy `str` conversion corrupting an unpaired
surrogate -- is unit-tested everywhere rather than only where the loader can be
built.

The ABI verifier derives full C prototypes from Clang's view of canonical CNA
headers and compares them with every reviewed `cna-sys` function type. It
measures return and parameter types, scalar width/signedness, pointer depth and
constness, callback/struct pointers, and boolean/enum representations. The
current pass checks 1,326 functions and 4,574 prototype type positions.
Independent C and Rust probes make 1,845 measurements across 98 structures,
19 callback signatures, scalar representations, and 665 constants, with zero
mismatches.

A second, independent gate covers the 1,119 symbol acquisitions that fill the
runtime function-pointer tables. Matching prototypes are not enough on their
own: two neighbouring routes can each be declared correctly and still be wired
to each other's table field, because the acquisition site casts an untyped
symbol. The gate therefore derives the expected alias from each acquired
symbol's own name and proves the field it fills carries *that* route's
prototype, so a swapped pair fails even though nothing about either declaration
is wrong.

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
host-owned portion needed during callbacks. CNA ABI 0.21's frame hooks now drive
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

CNA ABI 0.21 rejects `cna_game_destroy` while owned child handles remain, but
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
native dispatch because ABI 0.21's `RelativePath` copies valid UTF-8 but does
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
routes accept only layouts CNA ABI 0.21 can represent exactly; they validate
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

## Audio/XACT ownership and dispatch

Audio uses one per-Game generation registry for native cleanup and callback
invalidation; it does not introduce ambient Game state. SoundEffect owns one
native root, and every SoundEffectInstance owns one native instance while
strongly retaining its effect state. DynamicSoundEffectInstance composes that
same instance state instead of wrapping the native handle a second time.

AudioEngine owns the XACT root. AudioCategory is a parent-owned stable facade;
WaveBank and SoundBank are owned children retaining the engine, and Cue retains
the SoundBank plus engine/native dependency state. Explicit disposal and Drop
are idempotent. If CNA refuses wrong-thread destruction, the state keeps the
handle intact for a later owner-thread retry. Shutdown releases live Audio
children before destroying their Game parent.

Native BufferNeeded and microphone notification trampolines catch panic and
enqueue only weakly referenced work. Arbitrary handlers execute through the
existing owner-thread FrameworkDispatcher pump after a successful Game Update;
there is no Audio-specific worker or second event pump. Subscription snapshots
preserve order, duplicates, self-removal, and reentrant submission. Native
registrations are detached before invalidation, so queued work cannot
resurrect disposed objects or a previous Game generation.

SoundEffect's four process-static values still call real CNA routes. The
process-global Media callback registrations retain the exact native library
generation, and the values are now verified to persist across Game recreation
as XNA requires. Likewise, multiple-listener Apply3D returns
UnsupportedRuntime because CNA only accepts one listener; AudioEngine
renderer/look-ahead values are validated and forwarded even though CNA ignores
them. Physical microphone capture and authored XACT playback remain
hardware/asset qualification gaps, not structural gaps.

## Media/Video global ownership and dispatch

`MediaPlayer` is a constructorless process-global facade over one synchronized
`MediaRuntime`. The runtime accepts exactly one active Game, records its owner
thread and a monotonic generation, and retains no raw native pointer globally.
Every Media object records that generation. Game teardown invalidates the
registered graph, drains or defers owner-thread releases, discards pending
events, and detaches the native Game before destruction. A later Game obtains
a fresh generation; an old queue, Song, library child, Video, or player fails
deterministically rather than aliasing the new native handles. Only XNA's
process-global scalar settings and event subscriptions survive that boundary.

`MediaLibrary` and `MediaSource` use CNA's provider routes exclusively. Seven
read-only collection facades preserve native order, stable per-index identity,
checked bounds, snapshot iteration, and parent invalidation. Relationship
properties cache the same public `Arc` facade on repeated access. Empty music
catalogs are accepted as provider evidence; the host picture provider's real
entries are not replaced by arbitrary filesystem files or synthetic metadata.

MediaPlayer's queue is one cached facade per generation. Queue elements are
independently owned CNA Song copies whose stable Rust identities are cached;
the queue facade never owns the player. Play, pause, resume, stop, movement,
volume, mute, repeat, shuffle, control, position, and visualization all use the
canonical native routes. Visualization buffers are fixed 256-value read-only
views backed by the measured 2,056-byte ABI structure.

ActiveSongChanged and MediaStateChanged native trampolines catch panic and
only enqueue generation-tagged work. The existing FrameworkDispatcher owner
thread takes the registration cutoff recorded at callback time and invokes
handler snapshots in order. Self-removal, later handlers after panic,
reentrant Stop/Play through dispatch-scoped extension helpers, skipped dispatch
after a failed Update, teardown discard, and Game recreation are covered
without a second Media dispatcher. Those helpers refuse calls outside a Media
handler and never publish an ambient Game handle.

`Video` owns its CNA metadata object, while `VideoPlayer` owns one player and
retains the active Video. Player scalar properties are cached exactly where
XNA keeps them readable after disposal; NaN volume is preserved and finite
out-of-range values fail. CNA's frame texture is player-owned and valid only
until the next call on that player, so `GetTexture` reads
`cna_video_player_get_frame_ext` and wraps a decoded frame in a borrowed
`Texture2D`: the Rust view never destroys the handle and refuses every use once
a later player call has replaced it, one call before CNA would answer
`INVALID_HANDLE`. The monotonic frame generation and presentation time CNA
publishes alongside the texture have no XNA counterpart and are exposed through
`cna::extensions::media`.

## Verification and fault evidence

The API verifier hashes all seven XNA 4.0 Windows runtime assemblies, extracts
neutral CLR metadata with Mono, applies the normative mapping, and inspects
compiler-produced rustdoc JSON. Schema 2 measures every declared structural
category and emits the deterministic type scoreboard. `unmeasuredCategories`
and the allowlist are empty; the leak-only public-surface gate is zero.

The XNA-derived managed corpus has 215 named observations plus a final count
assertion (216 assertions total). Twenty Audio observations cover enums,
listener/emitter values, binary32 sample arithmetic, instance defaults and
cached disposal behavior, and microphone sample arithmetic. Ten Media
observations cover exact enum values and the two 256-value visualization
views. Backend/hardware
state and callback timing remain native qualification. The pre-Audio native
suite uses isolated child processes for 209 created game lifetimes and
1,012 owned child-handle constructions, including ten buffer binding cycles,
ten SpriteFont atlas/content cycles, ten Effect parent/child cycles, ten
complete compressed Model/XNB cycles, ten stock-effect/Texture3D/
OcclusionQuery cycles, ten combined Framework/Touch/Storage/GamerServices
cycles, repeated dispose/drop, live-child parent shutdown, stable identities,
callback self-removal/panic/recreation, transfer validation, and injected
failures. Dedicated Audio stress adds at least 75 effect, 75 instance, 75
dynamic, 50 callback, 60 microphone, 21 engine, and 60 malformed-bank cycles,
including wrong-thread refusal and owner-thread retry. Dedicated Media stress
adds at least 20 library, 20 Song, 20 queue-generation, 20 Video, 20
VideoPlayer, 20 frame-route, and 50 callback-delivery cycles, including
wrong-thread retry, stale generations, panic, self-removal, reentrancy, and
Game recreation. Absence of a crash is not a leak proof.
`tools/native-stress/run-sanitized.sh` is an optional path requiring a
separately instrumented exact ABI-0.21 CNA library. Sanitizer status is
`not-run`; no sanitizer pass is claimed for the current run.

## Current blockers and dependency order

The historical unmodified-build blocker is closed. Canonical CNA HEAD
`1bb2145d99ed572dd4eb15009c34e2e5f410fcf0` failed its C API build at
`CnaCApiCoreExt.cpp:250`, where the renderer identity assertion reduced to
`49 == 50`; ABI 0.20.0 removed eleven renderer identities and moved
`CNA_GRAPHICS_RENDERER_MAXIMUM` to 49, which is exactly that assertion. Runtime
evidence now uses an out-of-tree build of an unmodified canonical checkout,
labelled experimental ABI-0.21 HEADLESS.

The selected XNA 4.0 Windows runtime Rust projection is structurally complete:
all 259 expected Rust types are present and every strict diagnostic is zero.
This is not a claim about wider GamerServices/Avatar, Net, Content Pipeline,
Xbox, Windows Phone, or every host platform. LZX is complete for XNA 4.0
Windows framing. Repeated frame hosting still needs a CNA
core-callback-context rebinding route. PNG decoding remains a texture route,
not an alias for XNB content. Media catalog/picture providers, physical audio,
authored XACT assets, and real video decode remain explicitly qualified or
pending in the runtime-capability inventory.
