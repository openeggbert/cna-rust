# Normative XNA 4.0 to Rust mapping

Status: normative, version 2

Profile: XNA 4.0 Windows runtime

Authority: the seven hashed reference assemblies in
`tools/api-compat/profiles/xna40-windows-runtime.json`

This document defines the strict public projection below
`cna::Microsoft::Xna::Framework`. It does not claim that C# source is Rust
source compatible. A difference is acceptable only when this document defines
the language transformation and the verifier applies the same rule. Existing
CNA-Rust source is not an authority and receives no grandfathering.

## Identity and casing

XNA namespace segments, types, members, fields, enum variants, parameters, and
generic parameter names retain their original identifiers and casing wherever
Rust syntax permits it. Rust naming conventions are subordinate inside the
strict hierarchy. Compatibility modules use narrowly scoped
`non_snake_case` and `non_upper_case_globals` allowances.

Examples include `Microsoft::Xna::Framework`, `GameTime`, `CreateLookAt`,
`Vector2::Zero`, `Vector3::Up`, `Matrix::Identity`, `Color::CornflowerBlue`,
`TotalGameTime`, `ElapsedGameTime`, and `IsRunningSlowly`. Names invented by an
earlier scaffold are not aliases and do not become part of the contract.

Rust-only helpers belong at crate root or under `cna::extensions`; they do not
become unexpected strict XNA members. In particular, crate-root `run` and
`run_for_frames` are host functions and are not the projection of `Game.Run()`.

## Modules and types

CLR namespaces become nested Rust modules by replacing `.` with `::`. Public
XNA type names are preserved. A nested CLR type becomes a nested contract path;
when Rust cannot physically nest the declaration, the public re-export must
still expose the deterministic path recorded in the expected contract.

Non-generic types preserve their names. If a generic and non-generic CLR type
would collide after Rust removes CLR arity notation, the generic name gains
`OfT`, `OfT1T2`, and so on. The two current profile collisions are encoded in
`mapping-rules.json`, not an allowlist.

`Audio.RendererDetail` is an XNA value type whose public identity contains two
runtime-owned strings. It therefore maps to a non-`Copy` Rust value with copied
string properties; this ownership rule is explicit in `mapping-rules.json`.

CLR structs map to Rust structs. A nested CLR type whose enclosing type occupies
Rust's type namespace is flattened into the same XNA namespace using the
deterministic concatenation `OuterInner` (for example,
`TouchCollection.Enumerator` becomes `TouchCollectionEnumerator`); Rust cannot
declare a module and a struct with the same name. CLR enums map to `#[repr(...)]` Rust enums when
all bit patterns are invalid except declared values, or to a transparent
newtype with associated constants for flags/open native values. CLR interfaces
map to traits. CLR delegates map to callback traits or typed closure aliases as
specified by their use site. Classes normally map to structs plus contract
traits; the exceptional `Game` mapping is below.

## Fields and value semantics

An XNA public field is a public Rust field with the same name and compatible
projected type. XNA value types implement `Copy`, `Clone`, and `PartialEq`.
`Debug`, `Default`, `Eq`, `Hash`, and ordering traits are added only where their
semantics are valid. These Rust traits are language ergonomics, not unexpected
XNA members.

Pure math and value behavior is implemented in Rust and never crosses FFI.
Floating-point implementations must preserve XNA operation order where it
affects observable results. `Default` is not assumed to mean an XNA static
property unless the two values genuinely coincide.

## Static and instance properties

An immutable, const-representable static property `Foo` maps to the associated
constant `Type::Foo`. It preserves XNA casing even though ordinary Rust uses
upper snake case. A non-const static property maps to `Foo()`. There are no
method-shaped aliases for const properties.

A metadata read-only static property is not treated as a constant when its
value is runtime state. Such a property maps to a zero-argument function (plus
any normal explicit runtime context) even when its CLR value type could be a
Rust constant. `TouchPanel.IsGestureAvailable` is the current profile example:
it observes a queue and can fail when no active callback-scoped Game exists.
The exception is named in the machine mapping rather than inferred from its
`bool` return type.

When a CLR static graphics property requires the process-global current device
but the safe Rust projection intentionally has no global native runtime, the
mapping injects `graphicsDevice: &GraphicsDevice` as an explicit context
parameter. This applies to `GraphicsAdapter` static adapter/preference
properties and is encoded in `mapping-rules.json`. It does not authorize an
untracked handle or implicit global device.

Audio uses the same explicit-context rule. `SoundEffect` and
`DynamicSoundEffectInstance` constructors, `SoundEffect.FromStream`, the four
process-global `SoundEffect` settings, microphone enumeration/default lookup,
`AudioEngine` constructors, and `FrameworkDispatcher.Update` receive
`game: &GameContext`. CNA ABI 0.20 requires a game handle for ownership or thread
affinity, while CNA-Rust intentionally has no ambient current-game singleton.
Calls that can reach native Audio/XACT state return `Result`; the pure
binary32 sample arithmetic and cached disposed/value properties remain
infallible.

Media uses the explicit context for `Song.FromUri`, media-source enumeration,
`MediaLibrary` construction, all `MediaPlayer` operations, and
`VideoPlayer` construction. `MediaPlayer` nevertheless remains an XNA
process-global static facade: the context proves the active owner thread and
selects the current CNA Game generation; it does not create a per-Game player
object. Native handles and queue views are generation-bound, while XNA-defined
process-global scalar properties persist across Game recreation.

A read-only instance property `Foo` maps to `Foo(&self)`. A read/write property
maps to `Foo(&self)` and `SetFoo(&mut self, value)`. A natural borrowed result is
returned as `&T` or `&mut T`; a resource is not cloned to imitate a property.
Where a separate mutable borrow is required by Rust, `FooMut` is the sole
deterministic adaptation. Protected properties are exposed only through the
base-contract trait/context that represents the protected relationship.

## Constructors

The primary constructor maps to `new`. It is selected deterministically by
ascending parameter count, then parameter/type contract order. Each additional
constructor maps to `from_<snake_case_parameter_names>`. If two constructor
signatures produce the same name, both append
`_as_<snake_case_projected_parameter_types>`. Constructor projection names are
Rust language adaptations because CLR constructors have no ordinary member
identifier.

The verifier owns this selection. A type may not choose a more convenient
constructor name locally. `From<T>` and tuple conversions may additionally be
implemented as Rust ergonomics, but do not replace the named strict projection.

## Method overloads and operators

Within each CLR method overload group, the first metadata declaration keeps the
exact XNA member name. Every later overload maps to
`NameWith<PascalCaseParameterNameAnd...>`. Existing camel-case inside a
parameter is retained (`scaleFactor` becomes `ScaleFactor`, never
`Scalefactor`). If two alternatives still collide, both append
`As<ProjectedParameterTypeAnd...>`. This rule is mechanical and is implemented
by the verifier.

CLR `op_*` methods do not create extra inherent methods. They map to the
corresponding Rust `Add`, `Sub`, `Mul`, `Div`, `Neg`, assignment, equality, and
comparison traits. The ordinary named XNA methods such as `Add`, `Multiply`,
and `Negate` remain independently required. Operator trait implementations are
known language ergonomics, not unexpected strict members.

## Interfaces and class inheritance

An XNA interface is a Rust trait with the projected public contract. Class
inheritance is represented by composition plus traits for the public base
contracts. It never uses pointer casts. For example, `GraphicsResource` and
`Texture` are traits, and `Texture2D: Texture` expresses the important
`Texture2D : Texture : GraphicsResource` relationship.

The verifier checks each reference base/interface edge against
the corresponding trait implementation or declared composition relation.
Borrowed reflected children retain their parent with a borrow or shared owner;
they never destroy a parent-owned handle.

`DynamicSoundEffectInstance : SoundEffectInstance` is projected through the
crate-root `SoundEffectInstanceBase` contract and one composed instance state;
there is never a second native owner. `AudioCategory` is a CLR value type, but
its ABI-0.20 representation is an owned category handle tied to an engine. Its
Rust value is consequently non-`Copy`; `Clone`/equality preserve category value
semantics by sharing the one safe facade rather than duplicating native
ownership.

## Game

XNA `Game` is projected as a user-implemented `Game` lifecycle trait composed
through `GameStateAccess` with one `Arc<GameState>` and a callback-scoped
`GameContext`. The state provides durable per-instance components, services,
launch parameters, window, timing flags, and events; the context exposes the
host-owned portion while CNA invokes callbacks. This avoids pretending that a
stateless callback trait is the CLR base class. `GraphicsDevice` is a durable,
safe shared identity rather than a callback lifetime: only its private native
borrow is callback-scoped, and safe operations fail once the host invalidates
the identity.

Lifecycle virtuals retain XNA names (`Initialize`, `LoadContent`, `BeginRun`,
`Update`, `BeginDraw`, `Draw`, `EndDraw`, `OnExiting`, `EndRun`,
`UnloadContent`, and `Dispose`). `BeginDraw` returns `bool`; CNA skips the draw
callback when it is false. Public properties/events and run controls are trait
methods delegating to the composed state. Crate-root `run` is an additional
host entry point and remains explicitly outside the strict hierarchy.
`GameContext` is a mapped support type with machine-declared
`GraphicsDevice` and `Exit` members.

CLR component interface references map to shared
`Arc<dyn IGameComponent + Send + Sync>` identities. The component collection
uses stable registration order as the tie-breaker for equal update/draw order
and takes a shared snapshot for each traversal, so collection mutation cannot
invalidate a live Rust borrow. A component added after game initialization is
initialized immediately. `GameServiceContainer` maps CLR runtime type tokens
to Rust `TypeId` and providers to shared typed `Arc` identities; services are
owned by one game and are never global.

`Game.RunOneFrame` is a strict trait member, while the current native host can
safely execute it only as one complete owned session. CNA ABI 0.20 retains its
creation-time callback context and exposes no rebinding route, so arbitrary
repeated ticks cannot safely retain a borrowed Rust game. The implementation
must fail explicitly rather than manufacture a `'static` reference. This is a
runtime capability limitation, not a removal or renaming of the mapped member.

User `UnloadContent` and host resource teardown are distinct. The host may
dispose registered native children before asking CNA to destroy the game, but
that internal action must not synthesize `UnloadContent` or public resource
events. CNA supplies the one user lifecycle notification during native
shutdown. Ordinary user cleanup is not required to be idempotent merely to
accommodate host teardown.

## Parameters, ref/out, and null

An input `ref T` maps to `&T`; a mutating `ref T` maps to `&mut T`. A single
`out T` maps to `&mut T` when initialized storage is semantically harmless.
Otherwise it maps to the return value. Multiple `out` values map to a named
result struct when field identity matters, or a tuple when the values are
anonymous and ordered. The choice is derived from metadata and a small typed
rule table, never selected ad hoc per implementation.

Nullable references map to `Option<T>`, `Option<&T>`, or `Option<&mut T>` based
on ownership. Nullable values map to `Option<T>`. Optional arguments remain
ordinary arguments or projected overloads; they are not conflated with null.
Safe APIs never represent a null native handle with integer zero.
Opaque `System.IntPtr` window identities map to the safe, non-dereferenceable
`cna::extensions::window::WindowHandle` value. Its integer representation is
private and no raw-pointer constructor or accessor is part of the safe API.

## Strings, arrays, and collections

Borrow-only string inputs map to `&str`; retained or returned strings map to
`String`. FFI converts UTF-8 with explicit length and reports encoding errors.
No native string pointer escapes.

Borrowed contiguous inputs map to slices. Owned returned arrays map to `Vec<T>`
only when XNA exposes array semantics. Named XNA collections receive wrapper
types preserving mutation and read-only rules, with `IntoIterator`, `Index`,
and `AsRef<[T]>` added only when behavior remains equivalent. Native array
storage is never exposed directly.

The two selected read-only Audio collections have no separately selected CLR
collection type. `AudioEngine.RendererDetails` therefore returns an owned
`Vec<RendererDetail>`, and `Microphone.All` returns
`Vec<Arc<Self>>` (that is, `Vec<Arc<Microphone>>`). `Microphone.Default`
returns `Option<Arc<Self>>`; an empty native enumeration is not an error and no
synthetic device is inserted. Repeated enumeration reuses each game
generation's stable microphone facade identity.

The seven named Media collections are read-only wrappers, not mutable `Vec`
aliases. They retain their native parent, preserve native order, cache one
stable `Arc` facade per index, expose checked XNA `Item`/`Count`, and provide a
fallible snapshot enumerator plus ordinary Rust `IntoIterator` ergonomics. A
legitimately empty native collection remains empty; the binding never inserts
catalog entries.

Borrowed `System.Uri` inputs map to UTF-8 `&str`. `Picture.Date` maps to
`std::time::SystemTime`. Media methods accepting `System.IO.Stream` map to a
borrowed `&mut R where R: Read`; returned album-art, picture, and thumbnail
streams map to owned `Box<dyn Read + Send>` values. No stream route is replaced
by direct host-filesystem access. `VisualizationData`'s two CLR
`ReadOnlyCollection<float>` properties map to immutable `&[f32]` views whose
length is always exactly 256.

## Design-time math converters

The thirteen `Microsoft.Xna.Framework.Design` types are an observable value
conversion API, but their CLR base classes and host services are not a useful
Rust runtime model. CNA-Rust therefore does not publish a fake
`System::ComponentModel` namespace. The strict converter types remain under
the XNA Design namespace; their small shared support vocabulary is published
at crate root so it cannot be confused with additional XNA types.

`MathTypeConverter` is the concrete, directly constructible XNA foundation.
Its common observable behavior is also represented by the crate-root
`MathTypeConverterBase` trait, which is the declared base projection for the
twelve concrete converters. The protected CLR implementation fields
`propertyDescriptions` and `supportStringConvert` are omitted: they are not
public state, and their effects are represented by the trait operations and
immutable converter metadata.

The CLR concepts used by the selected contract map as follows:

| CLR concept | Rust projection |
|---|---|
| `TypeConverter` / `ExpandableObjectConverter` | `MathTypeConverterBase`; no general CLR converter hierarchy |
| `System.Type` | closed, stable `DesignType` enum; never public `std::any::TypeId` |
| `CultureInfo` | explicit `&DesignCulture` containing only decimal/list/special-number symbols |
| `PropertyDescriptor` | immutable `DesignPropertyDescriptor` with stable name and `DesignType` |
| `PropertyDescriptorCollection` | immutable ordered `&[DesignPropertyDescriptor]` |
| `InstanceDescriptor` | executable `DesignInstanceDescriptor` with stable `DesignConstructor`, ordered arguments, and completeness |
| `IDictionary` | optional ordered `&[DesignPropertyValue]`; reconstruction looks up names explicitly |
| `object` converter values | closed `DesignValue` union plus `Option` for CLR null |
| `ITypeDescriptorContext` | omitted; XNA converter IL does not observe it outside CLR delegation/converter lookup |
| `Attribute[]` property filters | omitted; all XNA math converters return their fixed descriptor collection without inspecting it |

`DesignValue` is deliberately not `dyn Any`. It admits only the strings,
component scalars, and existing XNA value types needed by this family. This
makes null and wrong-type failures deterministic without leaking arbitrary
reflection objects. A property input is name-addressed: source order is not
semantically significant, unrelated names are ignored as CLR `IDictionary`
entries are, and every required name must occur exactly once with a non-null
value of the exact mapped type. Property metadata and extracted values retain
the XNA descriptor order. Nested XNA value types are copied, preserving CLR
value-type snapshot semantics.

`DesignInstanceDescriptor` is not a reflection framework. It can identify and
invoke only the twelve constructors selected by XNA's concrete converters.
Arguments remain ordered and typed `DesignValue`s, and every descriptor is
complete because XNA constructs each one with the three-argument
`InstanceDescriptor` overload's default completeness. Matrix exposes
`Translation` as its first property but reconstructs from `M11` through `M44`;
that asymmetry is retained.

All conversion, extraction, creation, and descriptor invocation operations
that can observe CLR conversion exceptions return `Result`. `CanConvertFrom`
and `CanConvertTo` remain infallible capability checks. A null destination
type, null value, incompatible value, malformed component, missing/duplicate
property, or wrong property type is a deterministic error. Extra property
names are ignored.

String-support flags follow XNA IL, not converter naming. Point, Vector2,
Vector3, Vector4, Quaternion, and Color accept component strings. Rectangle,
Matrix, BoundingBox, BoundingSphere, Plane, and Ray reject string input.
Every concrete converter can convert its selected value to a string and an
instance descriptor. Supported component strings use the culture list
separator followed by one space on output. Integer/byte conversion is decimal;
Single conversion reproduces the XNA Windows CLR seven-significant-digit
legacy half-up rounding, exponent threshold/casing, culture decimal symbol,
special values, and
the observable normalization of negative zero to `0`. Parsing is directly to
binary32, accepts surrounding component whitespace and exponent notation,
preserves signed zero, accepts the culture's NaN/infinity tokens, and rejects
finite overflow. The unsupported-input converters use the inherited value
`ToString` fallback rather than inventing a component grammar.

## Begin/End asynchronous operations and storage streams

The XNA Storage `Begin*`/`End*` pattern maps to a concrete crate-root
`StorageAsyncResult`, not CLR `IAsyncResult`, a thread pool, or a fabricated
pending task. A Begin method returns `Result<StorageAsyncResult>`, receives an
optional one-shot `StorageAsyncCallback`, and retains an optional
`StorageAsyncState` (`Arc<dyn Any + Send + Sync>`). CNA 0.20 completes these
operations synchronously, so the callback runs exactly once before Begin
returns. The result still records completion and enforces the observable CLR
End rules: End is one-shot, a result is valid only for its originating
operation, and container-open results additionally belong to their originating
device. A callback panic is caught at the Rust boundary and returned as a
callback error; it never crosses C.

Returned `System.IO.Stream` values from `StorageContainer` map to the concrete
crate-root `StorageStream`. It owns one CNA storage-stream handle, retains its
container, implements `Read`, `Write`, and `Seek`, and closes idempotently.
Storage file modes/access/share values map to safe crate-root enums with the
fixed XNA identities. Storage I/O, selector, filesystem, and disposal members
return `Result`; no Storage operation bypasses CNA through `std::fs`.

## Generics and content

Rust generics remain typed Rust generics. XNA content loading maps asset types
through a `ContentLoadable` contract/reader registry; the registry supplies the
runtime type information that CLR reflection supplied. `ContentManager.Load<T>`
retains `Load` on its primary projected overload. Raw encoded-image loading via
`Texture2D::FromStream` is distinct from XNB content loading and is never
reported as `ContentManager` support.

`ContentManager.Load<T>` returns `Arc<T>` because XNA caches and returns one
observable reference identity for an asset/type pair. The manager's erased
storage is private; callers and custom readers remain typed. A cache hit with a
different requested `T` is an error, not a second interpretation of the same
payload. `ContentLoadable` supplies the typed target and optional disposable
identity; `ContentDisposable` is the Rust ownership hook used by `Unload` and
does not introduce a dynamic public XNA loading surface.

The CLR `ContentTypeReader<T>` collision maps to `ContentTypeReaderOfT<T>` under
the generic/non-generic collision rule. Registered readers are activated from
the XNB reader table, use their declared reader version, receive an optional
existing instance, and return the exact typed `Arc<T>`. Shared resources use
typed fixups after the root object is read. External references recursively use
the same manager/cache pipeline. Reader-created disposable resources are
recorded as they are created so a later reader failure can release a partial
object graph.

XNB framing and the reader system are managed Rust. Native CNA is entered only
when a built-in reader must construct a graphics resource. An uncompressed
`Texture2D` XNB therefore follows `ContentManager -> ContentReader ->
Texture2DReader -> Texture2D` while an encoded PNG follows `FromStream`; these
routes are intentionally never conflated.

## Vertex and index data

XNA `IVertexType` maps exactly to a safe trait whose
`VertexDeclaration(&self)` returns a retained declaration reference. Built-in
vertex structs also expose their XNA static `VertexDeclaration()` property and
return one stable declaration identity. The structs use explicit C-compatible
layout where native transfer observes layout; size, alignment, offsets, and
stride are verified independently.

Generic buffer and `DrawUser*` element types map to typed Rust slices. The
crate-root `VertexData` and `IndexData` bounds are Rust safety adaptations
outside the strict namespace. `VertexData` explicitly encodes/decodes fields
and provides a declaration, so arbitrary Rust padding is never reinterpreted as
initialized bytes. `IndexData` admits only the exact 16- and 32-bit integer
families. The strict API is not weakened to `&[u8]`; validated byte buffers
exist only inside the bridge.

`VertexBufferBinding` retains shared buffer identity. Device binding state
therefore remains observable and protects the underlying CNA raw binding: a
bound buffer cannot be destroyed until it is unbound. `Discard` and
`NoOverwrite` retain their distinct enum identities and are forwarded exactly;
an unsupported native option must fail rather than degrade to `None`.

## Effects, fonts, and collection indexers

`Effect` is an owned `GraphicsResource`. Its annotation, parameter, pass,
technique, and collection objects are parent-owned views: they retain the
parent's durable state, never destroy the Effect, and fail safely after parent
disposal. Repeated lookup of the same native child handle returns one cached
logical identity. Assigning `CurrentTechnique` validates parent and device
identity.

Effect parameter overloads remain typed families for booleans, integers,
floats, vectors, quaternion, matrices, arrays, strings, and textures. They do
not collapse into `Any`, JSON, or an untyped public byte value. Texture
parameters retain a safe tracked wrapper so CNA's raw handle does not create a
second owner. `EffectPass.Apply` and Effect-bearing `SpriteBatch.Begin` methods
must execute real CNA routes; a renderer capability failure remains an error.

CLR collection indexers overloaded by integer and string cannot share one Rust
method name. The strict collection retains the metadata-selected string
`Item`, while the deterministic integer operation is exposed as `item_at`
through a collection-specific trait in `cna::extensions::graphics`. The
extension changes only Rust call syntax; it does not invent another strict XNA
member or alter the collection's parent-owned identity.

`SpriteFont` has no public XNA constructor and is produced by its normal XNB
reader. The font retains its atlas as part of one content-owned object graph;
SpriteBatch borrows it and never owns it. CLR `StringBuilder` draw/measure
inputs map to `&str` in this profile because Rust has no separate mutable
builder reference type at the call boundary and the operation only observes
text. Scalar and vector scale overloads remain mechanically distinct.

## Models and stock effects

The Model graph is reference-valued. `Model` owns strong identities for every
bone, mesh, mesh part, collection, buffer, and effect required by the graph.
Repeated collection/index/name access returns the same `Arc` facade. Back-links
that would otherwise close a strong cycle use `Weak`: in particular bone
parents and mesh-part sibling/owner links. The shared lifetime contains no
strong facade back-link. Retaining a child can therefore preserve its safe
facade identity without keeping an uncollectable Model cycle alive; after
Model invalidation, child operations return an error.

Model `Tag` values map to `Option<Arc<dyn Any + Send + Sync>>`, matching the
same safe retained object policy used by other reference-valued tags.
`ModelMeshPart.Effect` maps to `Option<Arc<dyn EffectBase>>` because XNA permits
different Effect subclasses and multiple parts can observe one shared effect
identity. The trait object retains the one concrete native owner; it never
creates an Effect wrapper from a raw handle. Model collection CLR enumerators
are non-Copy Rust structs because they retain reference identities while
advancing. Their flattened names follow the general nested-type rule.

BasicEffect, AlphaTestEffect, DualTextureEffect, EnvironmentMapEffect, and
SkinnedEffect each compose the public `EffectBase` contract while retaining a
distinct concrete CNA stock-effect owner. Shared implementation is private and
does not merge their public types. IEffectFog, IEffectLights, and
IEffectMatrices map to Rust traits implemented directly by the corresponding
concrete effects; they expose the same underlying state rather than a copied
interface object.

DirectionalLight properties return stable borrowed children of a stock
effect. A light shares the parent ResourceState, never destroys the Effect, and
fails after parent disposal. The parent destroys each native light view once.
Stock-effect Texture2D/TextureCube properties use `Option<Arc<T>>` so an
assigned native raw texture handle is always backed by a safe tracked identity.

Generic Texture3D transfer methods map through the sealed crate-root
`Texture3DData` safety contract. It admits only element layouts with an exact
reviewed CNA encoding; this profile currently exposes `Color`. The strict
surface remains typed slices rather than public bytes, and an unsupported
SurfaceFormat/type combination is an error.

## Events and delegates

An event `Foo` maps to `AddFooHandler` and `RemoveFooHandler`. The returned
registration token identifies removal when token semantics are necessary for
safe teardown. Instance subscription methods take `&self`: the event registry
uses interior mutability so events remain usable through stable shared class
identities such as `Game.Components`. A generic CLR event payload maps to
`EventHandler<TEventArgs>` instead of erasing the argument type. Handlers are
typed callbacks; panic is caught before it can unwind into native code and is
converted to a callback error. Handler order and self-removal follow the
registration snapshot taken for that event emission. A closure must be
`Send`/`Sync` only when CNA can invoke it from the corresponding threads.
Delegate signatures follow the same parameter, null, and ref/out mappings as
methods.

Static MediaPlayer events retain XNA's null sender as the stateless Rust unit
sender. Because ordinary MediaPlayer operations require an explicit
callback-scoped `GameContext`, the CNA-only `extensions::media::PlayFromEvent`
and `StopFromEvent` helpers provide the otherwise unreachable XNA reentrant
transport case. They are enabled only while the single owner-thread Media
event pump is invoking a handler, use the current checked generation, and fail
outside that scope. They do not create ambient general-purpose Game state or
additional strict XNA members.

## Disposal, ownership, and unsafe code

An owning `IDisposable.Dispose()` maps to explicit
`Dispose(&mut self) -> cna::Result<()>` and idempotent `Drop`. A CLR value type
whose `Dispose` is a no-op (notably a collection enumerator) retains the
explicit no-op method but does not gain Rust `Drop`; this preserves its `Copy`
value semantics. Successful resource disposal first destroys the exactly-once
owned native handle and then marks the wrapper disposed. Repeated `Dispose` and
`Dispose` followed by `Drop` do not call CNA twice. A failed destroy preserves
the handle so the caller or `Drop` can retry according to the native contract.

Internal handle states are:

- `Owned`: this wrapper destroys the handle;
- `Borrowed`: validity is bounded by a Rust lifetime and this wrapper never
  destroys it;
- `ParentOwned`: the parent owner keeps it alive and destroys it;
- `Adopted`: ownership transfers exactly once and the source is invalidated.

Only `cna-sys` and the crate-private bridge contain unsafe operations.
`unsafe_op_in_unsafe_fn` is denied. No raw pointer, integer handle, `cna_sys`
type, or public unsafe function may leak into the strict safe API.

Game-owned `GraphicsDevice` uses shared identity, not shared native ownership.
Resources retain that identity so `GraphicsResource.GraphicsDevice` aliases the
same logical device and same-device checks remain meaningful. The game remains
the sole native device owner. When shutdown begins, registered owned children
are released in reverse registration order, the parent destroy is attempted,
and the shared device identity is invalidated even when CNA reports a destroy
error. Access after invalidation fails safely; disposing an already released
child remains idempotent.

Managed graphics-state resources may be constructed without a device, so the
base `GraphicsResource.GraphicsDevice` result is `Option<&GraphicsDevice>`.
Their first real application binds them to one durable device identity.
Const-representable XNA stock-state properties are associated constants; their
descriptors are immutable and use the same first-bind rule.

Persistent reference-valued device properties use stable shared identity.
Repeated access to `PresentationParameters`, adapter, texture/sampler
collections, and graphics-state objects returns the same logical object.
Native refresh updates that retained object in place; explicit `Clone` of
`PresentationParameters` creates an independent managed value. A device
collection may return a tracked safe resource wrapper, but it must never create
an unrelated owner merely because CNA reports the same raw handle.

Mutable CLR reference graphs delivered through events follow the same shared
identity rule. `GraphicsDeviceInformation` retains its `GraphicsAdapter` and
`PresentationParameters` as `Arc<T>` values, and
`PreparingDeviceSettingsEventArgs` retains the information object as an
`Arc<T>`. The information object's property setters use a shared receiver and
interior mutability. Consequently one event handler's edits are visible to
later handlers and to native device selection after the event returns; an
owned Rust clone would incorrectly sever CLR reference identity.

Protected methods that mutate a caller-owned CLR `List<T>` use
`&mut Vec<T>`. In particular, `GraphicsDeviceManager.RankDevices` receives a
mutable vector; consuming the vector would lose the observable in-place
ranking contract.

Render-target bindings use the same retained-resource rule as buffer bindings.
The device stores stable logical bindings only after native `SetRenderTargets`
succeeds, rejects duplicate/wrong-device/disposed/incompatible targets first,
and clears bindings before child destruction. HEADLESS or another backend may
reject the operation; the safe projection preserves that error and never
fabricates a render target or pixel result.

## TimeSpan and errors

`System.TimeSpan` maps to a signed `TimeSpan` value containing `i64` 100 ns
ticks. It supports negative intervals and checked arithmetic; it is not
`std::time::Duration`. `GameTime` exposes `TotalGameTime`, `ElapsedGameTime`,
and `IsRunningSlowly` under the property rule.

Fallible native calls return `cna::Result<T, CnaError>`. Native result codes and
diagnostic text are retained. Invalid Rust-side use returns a deterministic
typed error. Reference operations that throw for invalid values map to a
documented panic only for infallible value APIs; resource, I/O, platform, and
runtime failures use `Result`. Unsupported historical services return an
explicit unsupported error and never fake success.

## Extensions and verification

CNA-only functionality lives under `cna::extensions` and is excluded from the
strict expected XNA surface. Raw handles and internal types are forbidden in
both surfaces.

`tools/api-compat/mapping-rules.json` is the executable subset of this document.
The strict verifier compares every current public strict type/member contract
with the authoritative transformed contract. It measures bases/traits,
interfaces, parameters/returns, generics/bounds, ref/out, enum/flags,
delegates/events, disposal, constructors, overloads, and properties, and emits
a per-type scoreboard. A future category must enter `unmeasuredCategories`
until it has an executable comparison; it may not be printed as a false zero.
The allowlist starts empty and must stay empty unless a difference is
individually justified and cannot be expressed as a general language rule.
