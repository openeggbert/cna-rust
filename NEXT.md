# Session evidence and next work

## 2026-08-23 — Content, device completion, Effect, and SpriteFont run

### Strict result

```text
reference XNA types                257 -> 257
reference members                2964 -> 2964
expected mapped Rust types         259 -> 259
actual strict Rust types           117 -> 165

total diagnostics                  178 -> 94
missing types                      142 -> 94
missing members                     36 -> 0

constructor mapping mismatch         1 -> 0
overload mapping mismatch            17 -> 0
property mapping mismatch             4 -> 0
event mapping mismatch                0 -> 0
```

All remaining 94 diagnostics are missing types. Every parameter/return,
base/trait/interface, generic/bound, ref/out, enum/value, flags, delegate,
disposal, unexpected-surface, type-kind, internal/raw-handle/public-unsafe,
allowlist, and unmeasured-category count remains zero. Normal strict mode exits
1 only for those genuine missing types; leak-only exits 0.

```text
Game                 2 -> 0
GraphicsDevice      26 -> 0
SpriteBatch          8 -> 0
```

### Content and Game

`ContentManager` owns a typed, case-insensitive asset cache and an ordered
disposable registry. `Load<T>` uses the formal `ContentLoadable` and
`ContentTypeReaderRegistry` contracts; it does not expose an unrelated dynamic
`Any` loader. `Unload` disposes unique resources in reverse registration order,
continues after one disposal failure, clears the cache, and preserves failed
resources for a later `Dispose`. Wrong requested type, missing asset, reader
version, existing instance, shared resources, external references, reader
failure, duplicate disposable identity, and partial cleanup are tested.

The managed XNB reader validates `XNBw`, version 5, flags, declared size,
uncompressed framing, reader table/indexes/versions, shared fixups, and trailing
data. Built-ins cover strings, Boolean/integer/floating families, Vector2/3/4,
Quaternion, Matrix, Color, Rectangle, lists needed by SpriteFont, Texture2D,
SpriteFont, and Effect. One user-defined type loads through the ordinary reader
table and typed custom reader. LZX is not attempted.

`Game.Content` returns one stable manager and the mapped setter is complete.
The manager never owns the native Game. Content resources are released before
parent native destruction, while user `UnloadContent` still occurs exactly
once. PNG decoding remains the separate `Texture2D::FromStream` route.

### Repeated hosting

The CNA headers and loaded ABI were audited for callback-context lifetime
support. ABI 0.7 retains creation-time user data and provides no safe rebinding
operation. Arbitrary repeated `RunOneFrame`/`Tick` on a borrowed Rust game is
therefore still `backend blocked`; no transmute, fake static lifetime, leak, or
dangling game pointer was introduced. The existing owned host session remains
safe and lifecycle order remains measured.

### Graphics device and buffers

`VertexElement`, `VertexDeclaration`, `IVertexType`, `VertexBufferBinding`, the
buffer enums, and five built-in vertex structs are strict-complete. Built-in
`repr(C)` size, alignment, offsets, and XNA strides are asserted. Custom typed
vertices use the public safe `VertexData` transfer contract; raw bytes remain
private.

Vertex/index and dynamic buffers are real owned `GraphicsResource` objects
associated with, but never owning, the durable device. Transfers validate
offsets, start/count, stride/element size, formats, and dynamic options before
CNA dispatch. `Discard` and `NoOverwrite` are passed exactly. A tracked binding
guard refuses disposal while CNA could retain a raw bound buffer; shutdown
unbinds before reverse-order child cleanup.

`Indices`, `GetVertexBuffers`, `SetVertexBuffer(s)`, all selected typed
`DrawUser*`, bound draws, instanced draws, render-target operations,
`Reset` overloads, `Present`, and back-buffer overloads are complete and use
reviewed CNA calls. HEADLESS accepts transfer/binding/reset/present routes,
returns the exact missing-applied-effect error for draw submission, and reports
explicit unsupported errors for rasterized back-buffer readback and unsupported
render-target/cube operations. No route is a no-op.

### Effect and SpriteBatch

The base Effect graph is strict-complete: Effect, Material, annotations,
parameters, passes, techniques, collections, and parameter class/type enums.
The owned Effect retains its device; child views are parent-owned and retain
the parent state but never destroy it. Collections cache child identity by CNA
handle. Name/index/semantic lookup, current-technique assignment, clone,
material, child-after-parent failure, and double disposal are verified.

Typed parameter APIs cover bool, int, float, Vector2/3/4, Quaternion, Matrix,
arrays, transpose variants, strings, and Texture types. `EffectPass.Apply`
uses CNA. CNA's manual reflection graph is also exercised through a deliberate
extension outside the strict XNA namespace; clone/material rebuild only when
the HEADLESS CNA clone route returns its known empty manual graph.

Both Effect-bearing `SpriteBatch.Begin` overloads now pass the actual Effect
and optional transform to CNA. `None`, wrong-device, disposed Effect, failed
Begin recovery, and normal Begin recovery are tested. Compiled XNA/FNA effect
bytecode remains an explicit HEADLESS capability blocker (`CNA error 6`),
including through a legal uncompressed Effect XNB fixture; no shader-execution
or visual correctness claim is made.

### SpriteFont and DrawString

`SpriteFont` is strict-complete with real native atlas/glyph/cropping/kerning/
character data, spacing/default-character state, and measurement. A legal
uncompressed XNB fixture builds its atlas through the ordinary Texture2D reader
and proves cache identity and ContentManager unload ownership. The atlas is
owned once by the font graph and is never owned by SpriteBatch.

All six mapped `DrawString` overloads submit real CNA glyph commands. Empty,
multiline, missing/default glyph, spacing, scalar/vector scale, rotation,
origin, flip effects, depth, disposed font, disposed atlas, and batch-state
paths are covered. HEADLESS proves the execution route and lifetime contract,
not visible glyph correctness.

### Behavior, ABI, and safety evidence

```text
XNA-derived observations          123 -> 140
assertions including count        124 -> 141
failures                                    0

reviewed ABI functions            104 -> 235
prototype type positions          388 -> 879
independent C/Rust measurements   419 -> 805
layouts / callbacks / constants    19 / 3 / 129 -> 48 / 3 / 206
mismatches                                  0

native game lifetimes             146 -> 177
owned native child resources      103 -> 283
buffer binding stress                       10 cycles
SpriteFont atlas/content stress             10 cycles
Effect parent/child stress                   10 cycles
compiled Effect XNB blocker                   1 cycle
native crashes                                0
observed double-free/UAF                      0
sanitizer status                        not-run
```

The corpus's new platform-neutral groups cover Content metadata/cache identity
and vertex declarations/layout values. Native-dependent reader, draw,
reflection, and font-measurement cases remain in isolated native stress. The
ABI verifier compares all reviewed full prototypes and independently compiled
C/Rust layouts, callbacks, enum/flag scalars, and constants against canonical
headers and the exact loaded library.

Canonical CNA HEAD is still
`1bb2145d99ed572dd4eb15009c34e2e5f410fcf0`. Its unmodified C API build remains
blocked at `CnaCApiCoreExt.cpp:250` by renderer identity `49 == 50`; CNA was not
modified. Tests use the labelled experimental exact ABI-0.7 HEADLESS artifact.
No exact ABI-0.7 sanitizer artifact was available, so crash absence is not leak
proof.

### Template and quality gates

The unchanged truthful template passes tests plus fresh 60- and 600-frame
HEADLESS runs with real PNG Texture2D, SpriteBatch, input, per-game service
identity, and clean shutdown. It still does not claim XNB content or windowed
rendering. A freshly generated consumer vendors both binding crates, contains
no developer/sibling paths, passes workspace tests, and completes 60 frames.

Format, workspace check, Clippy, all-feature tests, docs, report-only strict
generation, expected-failing strict check, leak-only gate, behavior corpus,
native ABI, native stress, template, and fresh-consumer gates are the required
final verification set.

### Next coherent slice

1. Build the Model graph now that buffers, Effect, and ContentReader exist.
2. Add Texture3D and stock effects through real CNA routes.
3. Extend graphic ContentReader built-ins and optionally implement XNA LZX.
4. Seek an upstream callback-context rebinding route for repeated ticks.
5. Continue the remaining generated scoreboard without prioritizing broad
   Audio/Media above the ready graphics/content dependencies.
6. Run ASan/UBSan only with an exact unmodified ABI-0.7 CNA build.
