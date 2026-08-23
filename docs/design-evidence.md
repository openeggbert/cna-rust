# Design converter evidence

**Updated:** 2026-08-23

**Authority:** Microsoft XNA Framework 4.0 Windows runtime metadata and IL,
qualified direct execution of the reference converter assembly, and
reference-backed Windows CLR Single observations

**Runtime:** managed Rust only; no CNA ABI calls

## Scope and result

The regenerated authoritative queue contained exactly these thirteen public,
concrete, constructible classes:

- `MathTypeConverter`
- `BoundingBoxConverter`
- `BoundingSphereConverter`
- `ColorConverter`
- `MatrixConverter`
- `PlaneConverter`
- `PointConverter`
- `QuaternionConverter`
- `RayConverter`
- `RectangleConverter`
- `Vector2Converter`
- `Vector3Converter`
- `Vector4Converter`

All thirteen now have zero local strict diagnostics. The strict inventory moved
from 203 to 216 actual Rust types and from 56 to 43 diagnostics. Every
remaining diagnostic is a whole missing Audio or Media type; Design is zero.

## Evidence method

The verifier re-extracted metadata from the seven hashed XNA 4.0 Windows
runtime assemblies before implementation. The Design declarations were also
checked against disassembled/decompiled IL, including the two protected
`MathTypeConverter` fields and the internal field/property descriptors used by
each concrete constructor.

A focused C# probe loaded a patched copy of the original reference assembly
under Mono so that managed converter IL could execute without the mixed-mode
native initializer. It recorded capability flags, descriptor order and types,
string results for invariant, en-US, and de-DE, malformed input failures,
property reconstruction, nested value snapshots, selected constructor members,
ordered descriptor arguments, and completeness. This qualifies the XNA
converter code and its reflection choices. Mono's modern `SingleConverter`
prints nine round-trip digits, so it is not used as evidence for XNA's Windows
.NET Framework legacy Single text. The latter is pinned from Windows reference
observations and the exact values used by the independent CNA-Java evidence,
including `1E-30`, `3.402823E+38`, special values, and signed zero formatting.

No other binding defines the contract. CNA-Java was used to identify useful
cases and cross-check evidence coverage only.

## Formal Rust projection

The observable converter behavior does not require a CLR designer runtime.
CNA-Rust publishes no `System::ComponentModel` tree and no general reflection
API. The strict XNA namespace contains only the thirteen converter types.
Their shared support types live at crate root and therefore do not increase the
strict expected-type inventory.

| CLR concept | Exact Rust mapping or omission |
|---|---|
| `TypeConverter` / `ExpandableObjectConverter` | `MathTypeConverterBase`, the common selected converter contract |
| `MathTypeConverter` | concrete strict XNA type plus implementation of that contract |
| `System.Type` | stable closed `DesignType`; public `std::any::TypeId` is not used |
| `CultureInfo` | explicit `&DesignCulture` with decimal/list/NaN/infinity symbols |
| `object` | closed `DesignValue`; `Option<&DesignValue>` represents CLR null |
| `PropertyDescriptor` | immutable `DesignPropertyDescriptor` name/type pair |
| `PropertyDescriptorCollection` | immutable ordered `&[DesignPropertyDescriptor]` |
| `IDictionary` | `Option<&[DesignPropertyValue]>` with explicit name lookup |
| `InstanceDescriptor` | executable `DesignInstanceDescriptor` |
| reflected constructor | closed stable `DesignConstructor` identity |
| `ITypeDescriptorContext` | omitted; XNA does not inspect it except when delegating to CLR converter services |
| `Attribute[]` | omitted; XNA returns the same fixed descriptors regardless of the filter |
| protected `propertyDescriptions` / `supportStringConvert` | omitted implementation state; observable effects remain |

The machine rule is `designTypeConverterProjection` in
`tools/api-compat/mapping-rules.json`. The verifier derives the selected Rust
signatures from that rule; it does not allowlist the CLR-shaped members.
Operations corresponding to conversion exceptions return `Result`.

## Ordered properties and creation

Each descriptor slice is static, immutable, and in XNA order. Extraction
returns the same order and copies nested XNA value types. Creation performs
name lookup, not positional or hash iteration. A null property collection,
missing required name, duplicate required name, null value, or incompatible
value type fails deterministically. Unrelated extra names are ignored, matching
XNA's direct `IDictionary["Name"]` accesses.

| Converter | Ordered properties and mapped value types |
|---|---|
| MathTypeConverter | none (`null` CLR collection becomes an empty Rust slice) |
| Point | X:Int32, Y:Int32 |
| Rectangle | X:Int32, Y:Int32, Width:Int32, Height:Int32 |
| Vector2 | X:Single, Y:Single |
| Vector3 | X:Single, Y:Single, Z:Single |
| Vector4 | X:Single, Y:Single, Z:Single, W:Single |
| Quaternion | X:Single, Y:Single, Z:Single, W:Single |
| Color | R:Byte, G:Byte, B:Byte, A:Byte |
| Matrix | Translation:Vector3, then M11..M44:Single in row-major field order |
| BoundingBox | Min:Vector3, Max:Vector3 |
| BoundingSphere | Center:Vector3, Radius:Single |
| Plane | Normal:Vector3, D:Single |
| Ray | Position:Vector3, Direction:Vector3 |

Matrix's `Translation` descriptor mutates/reads M41, M42, and M43 in XNA, but
`CreateInstance` ignores a Translation dictionary entry and consumes only the
sixteen named scalar fields. The Rust mapping retains this exact asymmetry and
does not reinterpret matrix layout.

BoundingBox, BoundingSphere, Plane, and Ray descriptors contain nested copied
Vector3 values, not flattened scalars or borrowed aliases. BoundingSphere
creation preserves XNA constructor validation: a negative radius fails, while
zero, non-finite component coordinates, and NaN radius retain ordinary XNA
floating behavior.

## String and culture behavior

The XNA `supportStringConvert` field is true by default and set false only by
the Rectangle, Matrix, BoundingBox, BoundingSphere, Plane, and Ray converter
constructors. Consequently exactly six concrete converters accept strings:
Point, Vector2, Vector3, Vector4, Quaternion, and Color.

Supported component output joins values with the culture list separator plus
one ASCII space. `Invariant` and `EnUs` use decimal dot/comma-list; `DeDe` uses
decimal comma/semicolon-list and its observed infinity symbols. Point and Color
use strict decimal Int32/Byte conversion. Floating paths parse directly to
`f32`, preserve binary32 values and negative zero, accept surrounding component
whitespace and exponent notation, accept the culture's NaN and infinity forms,
and reject malformed syntax and finite overflow. Output implements Windows CLR
legacy Single general formatting: seven significant digits with half-up
rounding, uppercase `E`, the observed fixed/scientific threshold, signed
two-digit exponent, and both
zeros formatted as `0`.

All concrete converters report String and InstanceDescriptor as supported
destinations. For the six converters that reject string input, conversion to
String falls through the CLR base converter to the value's existing
`ToString`; it is not a parseable component grammar. The passed converter
culture does not change that fallback path, as the direct reference probe
shows. Wrong selected values sent to String also use the base fallback; other
wrong values, null destination type, and unsupported destination types fail.

Representative pinned outputs include:

```text
Point invariant        1, -2
Point de-DE            1; -2
Vector3 invariant      1.25, -2.5, 3.75
Vector3 de-DE          1,25; -2,5; 3,75
Vector4 invariant      NaN, Infinity, -Infinity, 0
Vector4 de-DE          NaN; +unendlich; -unendlich; 0
Vector2 extremes       1E-30, 3.402823E+38
Color                  0, 255, 10, 40
Rectangle fallback     {X:1 Y:2 Width:3 Height:4}
```

No named-color grammar is implemented because XNA ColorConverter accepts only
four byte components. Empty text, named colors, wrong component counts, empty
components, incompatible decimal separators, non-integral integer components,
Int32/Byte overflow, and Single overflow fail.

## Reconstruction descriptors

`DesignInstanceDescriptor` stores one stable constructor identity, ordered
closed-union arguments, and `IsComplete == true`. `Invoke` validates argument
count and type before reconstructing the value. It performs no dynamic symbol
lookup and exposes no arbitrary reflection member.

| Converter | Reconstruction identity and argument order |
|---|---|
| Point | Point(Int32, Int32): X, Y |
| Rectangle | Rectangle(Int32 x4): X, Y, Width, Height |
| Vector2 | Vector2(Single x2): X, Y |
| Vector3 | Vector3(Single x3): X, Y, Z |
| Vector4 | Vector4(Single x4): X, Y, Z, W |
| Quaternion | Quaternion(Single x4): X, Y, Z, W |
| Color | reflected Color(Int32 x4) identity with ordered Byte R, G, B, A arguments, as observed from XNA |
| Matrix | Matrix(Single x16): M11..M44; no Translation argument |
| BoundingBox | BoundingBox(Vector3, Vector3): Min, Max |
| BoundingSphere | BoundingSphere(Vector3, Single): Center, Radius |
| Plane | Plane(Vector3, Single): Normal, D |
| Ray | Ray(Vector3, Vector3): Position, Direction |

Every descriptor is executed in the focused Rust suite and compared to the
source value. Color preserves the reference assembly's unusual observable
combination: the reflected constructor member reports four Int32 parameters,
while the descriptor arguments are boxed bytes. Invocation widens those four
bytes to the existing Int32 XNA constructor.

## Per-type completion matrix

`Properties`, `Create`, and `Descriptor` are all complete for every concrete
converter. “Fallback” means supported value `ToString` output but rejected
string input.

| Type | Local diagnostics | String parse | String format | Properties | Create | Descriptor | Culture behavior |
|---|---:|---|---|---|---|---|---|
| MathTypeConverter | 0 | base capability only; conversion unsupported | base value fallback | empty | unsupported | unsupported | no component culture |
| PointConverter | 0 | yes | component | X,Y | yes | yes | list separator; invariant integers |
| RectangleConverter | 0 | no | fallback | X,Y,Width,Height | yes | yes | passed culture ignored by fallback |
| Vector2Converter | 0 | yes | component | X,Y | yes | yes | decimal/list/special Single symbols |
| Vector3Converter | 0 | yes | component | X,Y,Z | yes | yes | decimal/list/special Single symbols |
| Vector4Converter | 0 | yes | component | X,Y,Z,W | yes | yes | decimal/list/special Single symbols |
| QuaternionConverter | 0 | yes | component | X,Y,Z,W | yes | yes | decimal/list/special Single symbols |
| ColorConverter | 0 | yes | component | R,G,B,A | yes | yes | list separator; byte components |
| MatrixConverter | 0 | no | fallback | Translation,M11..M44 | yes | yes | passed culture ignored by fallback |
| BoundingBoxConverter | 0 | no | fallback | Min,Max | yes | yes | passed culture ignored by fallback |
| BoundingSphereConverter | 0 | no | fallback | Center,Radius | yes | yes | passed culture ignored by fallback |
| PlaneConverter | 0 | no | fallback | Normal,D | yes | yes | passed culture ignored by fallback |
| RayConverter | 0 | no | fallback | Position,Direction | yes | yes | passed culture ignored by fallback |

## Behavior corpus and safety

The focused suite adds nine tests and the shared XNA-derived corpus adds 40
named Design observations. Total corpus measurements are now 185 observations,
186 assertions including the final count, and zero failures. Coverage includes
capability flags, all property orders/types, extraction, all CreateInstance
shapes, extra/missing/null/wrong properties, duplicate keys, supported and
unsupported string paths, invariant/en-US/de-DE, integer boundaries, byte
boundaries, exponent/whitespace/special/binary32 cases, malformed strings,
Matrix asymmetry, nested snapshots, all constructor identities, completeness,
and executable descriptor round trips.

Design adds no CNA function, native handle, callback, layout, constant, unsafe
public method, raw pointer, arbitrary `dyn Any`, or implementation type under
the strict XNA namespace. The native ABI inventory therefore remains exactly
431 reviewed functions, 1,509 prototype type positions, 936 independent C/Rust
measurements, 56 layouts, five callbacks, 243 constants, ABI 0.7, and zero
mismatches.
