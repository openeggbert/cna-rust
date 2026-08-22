# Normative XNA 4.0 to Rust mapping

Status: normative, version 1

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

CLR structs map to Rust structs. CLR enums map to `#[repr(...)]` Rust enums when
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

The verifier must eventually check each reference base/interface edge against
the corresponding trait implementation or declared composition relation.
Borrowed reflected children retain their parent with a borrow or shared owner;
they never destroy a parent-owned handle.

## Game

XNA `Game` is projected as a user-implemented `Game` lifecycle trait composed
with a callback-scoped `GameContext`. The context provides access to the
host-owned state whose lifetime CNA controls. This avoids pretending that a
stateless callback trait is the CLR base class and prevents a borrowed device
from escaping its native callback.

Lifecycle virtuals retain XNA names (`Initialize`, `LoadContent`, `Update`,
`Draw`, `UnloadContent`, and `OnExiting`). Properties, events, components,
services, window/device state, and run/exit behavior remain required work in the
strict contract. Crate-root `run` is the Rust host entry point; it is explicitly
outside the strict hierarchy. `GameContext` is a mapped support type with
machine-declared `GraphicsDevice` and `Exit` members.

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

## Strings, arrays, and collections

Borrow-only string inputs map to `&str`; retained or returned strings map to
`String`. FFI converts UTF-8 with explicit length and reports encoding errors.
No native string pointer escapes.

Borrowed contiguous inputs map to slices. Owned returned arrays map to `Vec<T>`
only when XNA exposes array semantics. Named XNA collections receive wrapper
types preserving mutation and read-only rules, with `IntoIterator`, `Index`,
and `AsRef<[T]>` added only when behavior remains equivalent. Native array
storage is never exposed directly.

## Generics and content

Rust generics remain typed Rust generics. XNA content loading maps asset types
through a `ContentLoadable` contract/reader registry; the registry supplies the
runtime type information that CLR reflection supplied. `ContentManager.Load<T>`
retains `Load` on its primary projected overload. Raw encoded-image loading via
`Texture2D::FromStream` is distinct from XNB content loading and is never
reported as `ContentManager` support.

## Events and delegates

An event `Foo` maps to `AddFooHandler` and `RemoveFooHandler`. The returned
registration token owns removal when token semantics are necessary for safe
teardown. Handlers are typed callbacks; panic is caught at the FFI boundary and
converted to a callback error. A closure must be `Send`/`Sync` only when CNA can
invoke it from the corresponding threads. Delegate signatures follow the same
parameter, null, and ref/out mappings as methods.

## Disposal, ownership, and unsafe code

`IDisposable.Dispose()` maps to explicit `Dispose(&mut self) -> cna::Result<()>`
and idempotent `Drop`. Successful disposal first destroys the exactly-once
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
The strict verifier compares every current public strict type/member name with
the authoritative transformed contract. It exits nonzero for missing,
unexpected, or mismatched items. Categories not yet implemented are reported as
unmeasured rather than printed as zero. The allowlist starts empty and must stay
empty unless a difference is individually justified and cannot be expressed as
a general language rule.
