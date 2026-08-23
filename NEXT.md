# Session evidence and next work

## 2026-08-23 — Design complete

### Exact strict handoff

```text
                                      BEFORE -> AFTER
REFERENCE_TYPES                           257 -> 257
REFERENCE_MEMBERS                        2964 -> 2964
EXPECTED_RUST_TYPES                       259 -> 259
ACTUAL_RUST_TYPES                         203 -> 216
TOTAL_DIAGNOSTICS                          56 -> 43
MISSING_TYPES                              56 -> 43
MISSING_MEMBERS                             0 -> 0

CONSTRUCTOR_MAPPING_MISMATCH                0
OVERLOAD_MAPPING_MISMATCH                   0
PROPERTY_MAPPING_MISMATCH                   0
EVENT_MAPPING_MISMATCH                      0
BASE_PROJECTION_MISMATCH                    0
TRAIT_MISMATCH                              0
INTERFACE_MISMATCH                          0
PARAMETER_MISMATCH                          0
RETURN_TYPE_MISMATCH                        0
GENERIC_MISMATCH                            0
GENERIC_BOUND_MISMATCH                      0
REF_OUT_MISMATCH                            0
ENUM_VALUE_MISMATCH                         0
FLAGS_MISMATCH                              0
DELEGATE_MISMATCH                           0
DISPOSAL_MISMATCH                           0
UNEXPECTED_TYPES                            0
UNEXPECTED_MEMBERS                          0
TYPE_KIND_MISMATCH                          0
INTERNAL_TYPE_LEAK                          0
RAW_HANDLE_LEAK                             0
PUBLIC_UNSAFE_API                           0
ALLOWLIST                                   0
UNMEASURED_CATEGORIES                       0
```

Normal strict mode exits 1 only for the 43 genuine whole missing types.
Report-only generation records the same result; leak-only exits 0 with zero
findings. Verifier self-tests pass 24/24.

```text
Graphics             0 -> 0
Framework/core       0 -> 0
Input                0 -> 0
Storage              0 -> 0
GamerServices        0 -> 0
Design              13 -> 0
Audio               19 -> 19
Media               24 -> 24
```

### Formal Design projection

The strict namespace contains exactly the thirteen XNA types and all have zero
local diagnostics:

```text
Microsoft.Xna.Framework.Design.MathTypeConverter             0
Microsoft.Xna.Framework.Design.BoundingBoxConverter          0
Microsoft.Xna.Framework.Design.BoundingSphereConverter       0
Microsoft.Xna.Framework.Design.ColorConverter                0
Microsoft.Xna.Framework.Design.MatrixConverter               0
Microsoft.Xna.Framework.Design.PlaneConverter                0
Microsoft.Xna.Framework.Design.PointConverter                0
Microsoft.Xna.Framework.Design.QuaternionConverter           0
Microsoft.Xna.Framework.Design.RayConverter                  0
Microsoft.Xna.Framework.Design.RectangleConverter            0
Microsoft.Xna.Framework.Design.Vector2Converter              0
Microsoft.Xna.Framework.Design.Vector3Converter              0
Microsoft.Xna.Framework.Design.Vector4Converter              0
```

No fake `System.ComponentModel` hierarchy exists. The formal crate-root Rust
support vocabulary is:

```text
TypeConverter/ExpandableObjectConverter -> MathTypeConverterBase
System.Type                              -> DesignType
CultureInfo                              -> &DesignCulture
object/null                              -> DesignValue / Option
PropertyDescriptor                      -> DesignPropertyDescriptor
PropertyDescriptorCollection            -> &[DesignPropertyDescriptor]
IDictionary                             -> &[DesignPropertyValue]
InstanceDescriptor                      -> DesignInstanceDescriptor
reflected constructor                   -> DesignConstructor
ITypeDescriptorContext                  -> omitted
Attribute[]                             -> omitted
```

`DesignValue` is a closed union, never arbitrary `dyn Any`. Properties are
immutable and ordered. Creation performs exact name lookup, ignores unrelated
extras, and rejects null, missing, duplicate, or wrong-typed required values.
Nested XNA values are copied. Reconstruction descriptors are complete,
executable, and limited to the twelve constructors selected by XNA; there is
no reflection or symbol lookup.

### Converter behavior

```text
Type                 String input  String output  Ordered properties
MathTypeConverter    capability*   base fallback  none
PointConverter       component     component      X,Y
RectangleConverter   rejected      fallback       X,Y,Width,Height
Vector2Converter     component     component      X,Y
Vector3Converter     component     component      X,Y,Z
Vector4Converter     component     component      X,Y,Z,W
QuaternionConverter  component     component      X,Y,Z,W
ColorConverter       component     component      R,G,B,A (Byte)
MatrixConverter      rejected      fallback       Translation,M11..M44
BoundingBoxConverter rejected      fallback       Min,Max (Vector3)
BoundingSphereConv.  rejected      fallback       Center,Radius
PlaneConverter       rejected      fallback       Normal,D
RayConverter         rejected      fallback       Position,Direction
```

`MathTypeConverter` reports String capability as XNA does but has no concrete
target conversion. Every concrete converter supports properties,
CreateInstance, String destination, and executable InstanceDescriptor
reconstruction. Matrix exposes Translation first but creation/descriptors use
only M11..M44. Color is byte RGBA with no named-color grammar. XNA's observed
Color descriptor has an Int32x4 constructor member and ordered boxed Byte
arguments; the Rust descriptor retains that combination.

Invariant/en-US and de-DE component formatting/parsing are covered. Floating
output uses XNA Windows seven-significant-digit legacy half-up Single
formatting, culture decimal/list/infinity symbols, uppercase exponent text,
and formats signed zero
as `0`. Parsing targets f32 directly, preserves signed zero/binary32, accepts
whitespace/exponents/specials, and rejects malformed values or finite overflow.
Unsupported-input converters preserve value `ToString` output; the passed
converter culture does not create a component grammar.

Full evidence: `docs/design-evidence.md`.

### Behavior and native invariants

```text
XNA-derived observations                  145 -> 185
assertions including final count           146 -> 186
Design observations                                 40
failures                                             0

reviewed ABI functions                    431 -> 431
prototype type positions                 1509 -> 1509
independent C/Rust measurements            936 -> 936
layouts / callbacks / constants       56/5/243 -> 56/5/243
ABI version                              0x0700 / 1792
ABI mismatches                                       0

native Game lifetimes                      209 -> 209
owned child-handle constructions          1012 -> 1012
native crashes                                       0
observed double-free/UAF                              0
sanitizer status                                not-run
```

Design is managed-only. No CNA C function, handle, layout, callback, constant,
device/resource ownership, or template source was added. The complete native
ABI verifier and crash-isolated native stress remain the regression evidence;
sanitizers require a separately instrumented exact ABI-0.7 library and are not
claimed by this run.

### CNA qualification

Canonical read-only CNA HEAD remains:

```text
1bb2145d99ed572dd4eb15009c34e2e5f410fcf0
```

The prior unmodified C API build remains blocked at `CnaCApiCoreExt.cpp:250`,
renderer identity `49 == 50`. Runtime evidence uses the labelled qualified
exact ABI-0.7 HEADLESS artifact:

```text
/tmp/cna-rust-native-070/modules/c-api/libcna_c_api.so
size:   16889928 bytes
sha256: 6dcefcadb7aa0233da98682bdbc343581a9f1e754a09c641078d1bef97afd7ca
```

Arbitrary repeated borrowed-game RunOneFrame/Tick still requires a safe CNA
core-callback-context rebinding route. No unsafe lifetime workaround exists.

### Template and consumer

```text
template source changed                          NO
template git HEAD          86612449a2414663f0e17dac98c1bd5239712559
template tests                                  PASS
template native smoke                     60 / PASS
template native stability                600 / PASS
fresh vendored consumer workspace tests        PASS
fresh consumer native smoke               60 / PASS
developer/sibling absolute-path findings         0
symlinks in generated consumer                    0
```

Cargo output is directed to `/tmp`; the sibling template source and status
remain unchanged. Generated consumers use only their vendored `cna` and
`cna-sys` crates; the relative path between them is intentional.

### Final gates

Passed:

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo clippy --workspace --all-targets --all-features` (audited
  compatibility/style warnings, exit 0)
- `cargo test --workspace --all-features`
- `cargo doc --workspace --no-deps`
- verifier self-tests, report-only, expected-failing normal strict, leak-only
- 185-observation XNA behavior corpus
- complete native ABI verifier
- full crash-isolated native stress
- `git diff --check`
- unchanged-template tests, 60/600 native runs, fresh vendored tests/smoke,
  developer/sibling path audit, and symlink audit

### Remaining work

Only these selected families remain, each as a separate milestone:

```text
Audio    19
Media    24
```

Start Audio by regenerating the authoritative scoreboard and reviewing the
native ownership/callback dependency graph. Do not reopen Graphics,
Framework/core, Input, Storage, GamerServices, Design, Content/XNB/LZX, or
other completed families unless a concrete regression appears. Do not start
Media as part of the Audio milestone.
