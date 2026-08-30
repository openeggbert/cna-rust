# Remaining Graphics completion evidence

Status date: 2026-08-23

This document records the dependency-coherent completion of the final 27
Graphics types in the selected XNA 4.0 Windows runtime projection. The strict
report is `target/xna-api-report-graphics-final.json`; generated `target`
artifacts are evidence outputs, not checked-in API definitions.

## Strict result

| Measurement | Before | After |
|---|---:|---:|
| reference XNA types | 257 | 257 |
| reference XNA members | 2,964 | 2,964 |
| expected mapped Rust types | 259 | 259 |
| actual strict Rust types | 165 | 192 |
| total diagnostics | 94 | 67 |
| missing types | 94 | 67 |
| missing members | 0 | 0 |
| Graphics missing types | 27 | 0 |

Constructor, overload, property, event, base projection, trait, interface,
parameter, return, generic, generic-bound, ref/out, enum value, flags,
delegate, and disposal mismatch counts are all zero. Unexpected types and
members, type-kind mismatches, internal-type leaks, raw-handle leaks, public
unsafe APIs, allowlist entries, and unmeasured categories are also all zero.

The remaining 67 diagnostics are whole missing types only:

```text
Media             24
Audio             19
Design            13
Framework/core     4
Input               3
Storage             3
GamerServices       1
Graphics            0
```

## Capability classification

| Family | Classification | Evidence boundary |
|---|---|---|
| Model graph | `STRICT_COMPLETE`, `MANAGED_VERIFIED` | Stable graph identity, transforms, tags, collections, validation, and parent invalidation |
| Model.Draw | `NATIVE_APPLY_VERIFIED` | Real buffer binding, pass application, and indexed command submission; no visual rendering claim |
| Model XNB | `STRICT_COMPLETE`, `NATIVE_CONSTRUCTION_VERIFIED` | Legal uncompressed graph, shared fixups, cache, unload/reload, malformed graphs, and rollback |
| BasicEffect | `STRICT_COMPLETE`, `NATIVE_CONSTRUCTION_VERIFIED`, `NATIVE_APPLY_VERIFIED` | Real CNA stock-effect handle and pass route |
| AlphaTestEffect | `STRICT_COMPLETE`, `NATIVE_CONSTRUCTION_VERIFIED`, `NATIVE_APPLY_VERIFIED` | Real CNA stock-effect handle and pass route |
| DualTextureEffect | `STRICT_COMPLETE`, `NATIVE_CONSTRUCTION_VERIFIED`, `NATIVE_APPLY_VERIFIED` | Real CNA stock-effect handle and pass route |
| EnvironmentMapEffect | `STRICT_COMPLETE`, `NATIVE_CONSTRUCTION_VERIFIED`, `NATIVE_APPLY_VERIFIED` | Real CNA stock-effect and light handles |
| SkinnedEffect | `STRICT_COMPLETE`, `NATIVE_CONSTRUCTION_VERIFIED`, `NATIVE_APPLY_VERIFIED` | Real CNA stock-effect and bone-transform routes |
| DirectionalLight and effect traits | `STRICT_COMPLETE`, `MANAGED_VERIFIED`, `NATIVE_APPLY_VERIFIED` | Stable parent-owned lights expose the concrete effect state |
| Texture3D | `STRICT_COMPLETE`, `MANAGED_VERIFIED`, `BACKEND_BLOCKED` | ABI route and validation are complete; qualified HEADLESS rejects construction with CNA error 6 |
| OcclusionQuery | `STRICT_COMPLETE`, `NATIVE_CONSTRUCTION_VERIFIED`, `NATIVE_APPLY_VERIFIED` | Begin/end/completion/result and disposal state machine verified; HEADLESS reports its real conservative result |
| Graphics services/exceptions | `STRICT_COMPLETE`, `MANAGED_VERIFIED` | Rust `Result<CnaError>` policy retained; no C# exception emulation |

All stock-effect entries are also `VISUAL_EXECUTION_NOT_QUALIFIED`: HEADLESS
proves native construction, parameter mutation, technique/pass application,
and command acceptance, not shader pixels. Generic compiled Effect bytecode
remains separately blocked by CNA error 6; no conclusion about stock effects
was inferred from that unrelated route.

## Model graph and ownership

`Model` strongly retains the complete graph and its shared lifetime. Bone,
mesh, mesh-part, and collection facades are stable `Arc` identities. Bone
parents and mesh-part sibling back-links are `Weak`; the shared graph lifetime
contains no back-link to a facade. Consequently the graph contains no strong
`Arc` cycle.

Vertex and index buffers retain their one existing native owner. Mesh parts
hold shared safe buffer identities rather than creating duplicate owners.
Effects use `Arc<dyn EffectBase>` so multiple parts can share one effect
identity. Directional lights are stable parent-owned views; disposing the
effect destroys their native view handles once and invalidates retained child
state.

Content unload records the Model graph last, so reverse disposal invalidates
the public graph before destroying stock effects and buffers. Model cleanup
clears only its own live device bindings before those buffers are destroyed.
Retained child facades then fail deterministically instead of observing a
dangling native resource.

`Model.Draw` walks meshes and parts, updates world, view, and projection through
IEffectMatrices-capable stock effects, rejects an incompatible generic Effect,
binds the ordinary vertex and index buffers, applies every current technique
pass, and calls the normal indexed device draw route. It uses the mesh-part
vertex offset as the indexed draw base vertex and does not introduce a native
Model renderer.

## Content/XNB

The built-in reader table now covers Model, VertexDeclaration, VertexBuffer,
IndexBuffer, BasicEffect, AlphaTestEffect, DualTextureEffect,
EnvironmentMapEffect, SkinnedEffect, Texture3D, and TextureCube in addition to
the previously completed Graphics readers.

The legal Model fixture contains two bones with a parent/child hierarchy, one
mesh, two parts, one shared vertex buffer, one shared index buffer, one shared
BasicEffect, tags, a bounding sphere, and a root identity. A private reader
finalization hook resolves ordinary shared-resource fixups before publishing
the graph. Tests cover case-insensitive typed cache identity, unload/reload,
wrong requested type, malformed root indices, absent part effects, failure
rollback, shared effect identity, transforms, retained child invalidation, and
the indexed draw path.

Legal standalone Texture3D and TextureCube fixtures exercise their ordinary
reader-table routes. Texture3D preserves the qualified HEADLESS construction
error. TextureCube either completes construction/cache identity or preserves
the backend's explicit unsupported transfer error; partial resources roll back
through the normal ContentManager registry.

Only `SurfaceFormat.Color` is accepted by these readers because that is the
only element encoding this safe binding can transfer exactly through the
reviewed routes. Unsupported formats are rejected, never reinterpreted.

XNA LZX framing remains explicitly unimplemented. No partial decoder or
MonoGame-specific compression assumption was added.

## ABI and runtime evidence

| Measurement | Before | After |
|---|---:|---:|
| reviewed ABI functions | 235 | 347 |
| prototype type positions | 879 | 1,220 |
| independent C/Rust measurements | 805 | 840 |
| layouts | 48 | 51 |
| callbacks | 3 | 3 |
| constants | 206 | 206 |
| mismatches | 0 | 0 |

The compiler-backed verifier checks all 347 functions against canonical CNA C
headers and the loaded library. The runtime is the labelled experimental exact
ABI-0.7 HEADLESS artifact at
`/tmp/cna-rust-native-070/modules/c-api/libcna_c_api.so`.

Crash-isolated stress now performs 197 created game lifetimes. The new slice
adds ten full Model/XNB/unload/reload cycles and ten stock-effect/Texture3D/
OcclusionQuery cycles to the previous buffer, SpriteFont, and Effect cycles.
Across the deterministic cases, 893 owned native child-handle constructions
are exercised. Native crashes, observed double-free, and observed UAF remain
zero. Sanitizers were not run, so this is not a leak-freedom claim.

The handle total is derived from the explicit deterministic constructors, not
from a leak heuristic: each Model cycle exercises five six-handle graph
attempts (wrong type, malformed root, missing part effect, valid load, and
reload), ten standalone stock-effect/light handles, and one TextureCube handle
(41 total); each remaining-Graphics cycle exercises 18 stock-effect/light and
two query handles (20 total). Thus `283 + 10*41 + 10*20 = 893`.

The platform-neutral XNA behavior corpus remains at 140 observations, 141
assertions including the final count, and zero failures. The new Model, stock
effect, and query evidence is native-dependent and therefore remains in native
stress instead of being mislabeled as a platform-neutral observation.

The unmodified canonical CNA checkout now builds its C API; the
`CnaCApiCoreExt.cpp:250` renderer-identity blocker was repaired by ABI 0.20.0
itself. CNA was not modified.

Arbitrary repeated ticking of one live borrowed game was re-measured on ABI
0.20 and the shape of the gap is unchanged: `CNA_GameCallbacks` is copied
during `cna_game_create` and no route rebinds its `context` afterwards.
`cna_game_set_frame_hooks_ext` does rebind its own table's context, but that
table carries Initialize/BeginRun/EndRun/BeginDraw/EndDraw, not Update and
Draw.

Worth stating precisely, because the previous milestone left it implicit: XNA's
own `Game.RunOneFrame` delegates to a host that does not exist until `Run`
creates one, so it does nothing when called outside a running host. The Rust
projection's self-contained single-frame session is therefore a deliberate
mapping rather than a workaround, and what remains genuinely missing is
repeated ticking of one live game from Rust.

The sibling template source was not changed. Its tests plus fresh 60- and
600-frame HEADLESS runs pass. A fresh generated consumer vendors both binding
crates, passes all workspace tests and a 60-frame native smoke, and contains no
developer path, sibling dependency, or symlink back to the source workspace.

## ABI 0.20 capability gaps the version migration left refused (2026-08-31)

The 0.7 -> 0.20 migration moved the version gate and proved the reviewed slice
had not decayed. It did not re-measure the *refusals* the safe layer carried
from 0.7, and six of them still named that version in a message a consumer
could see. Re-measuring each against the live canonical headers found four
that ABI 0.20 can now satisfy.

| Refusal carried from ABI 0.7 | Live canonical route | Outcome |
|---|---|---|
| `Keyboard.GetState(PlayerIndex)` had no route | `cna_keyboard_get_state_for_player` | bound; every slot answers |
| `Clear` could not select depth or stencil | `cna_graphics_device_clear_options` | bound; all three option bits reach CNA |
| a vertex byte offset could not carry `Discard`/`NoOverwrite` | `cna_vertex_buffer_set_data_raw_at_with_options` | bound; the offset write is read back |
| `Discard`/`NoOverwrite` needed a built-in XNA layout | `cna_vertex_buffer_set_data_raw_with_options` | bound; a user-declared vertex type streams |
| `GraphicsDeviceManager.RankDevices` | none | re-measured; still absent |
| an index byte offset could not carry a streaming hint | none | re-measured; `index_resources.h` has no option-carrying route |

`cna_graphics_device_dispose` also exists now, but it answers
`CNA_RESULT_NOT_SUPPORTED` for the running game's device **by design**: the
game owns that device and `cna_game_destroy` performs the canonical disposal.
The projection therefore still refuses, and the message now states CNA's
reason rather than claiming a missing route.

### What the clear rework actually changed

XNA's `Clear(ClearOptions, Vector4, float, int)` is not a floating-point
clear. Its first statement is `new Color(color)`, so XNA packs to eight bits
per channel before the device sees the value, and the `Vector4` and `Color`
overloads are the same call. The projection now performs that pack and sends
both overloads to `cna_graphics_device_clear_options`, which is why they agree
exactly.

`Clear(Color)` is `Clear(DefaultClearOptions, color, 1f, 0)`, and
`DefaultClearOptions` is state, not a constant: `Target`, plus `DepthBuffer`
when the active depth-stencil format is not `None`, plus `Stencil` only for
`Depth24Stencil8`. The active format is render target zero's when any render
target is bound and the presentation parameters' otherwise. The projection
computes it the same way; before this change the single-argument overload
cleared color alone.

`cna_graphics_device_clear_rgba` keeps the four `f32` channels and so has no
XNA spelling at all. Rather than dropping it from the audited table it is
published as `cna::extensions::graphics::FloatClearExt::clear_color_channels`,
a CNA-only clear for a renderer with a wide color target.

### Measurements

| Measurement | Before | After |
|---|---:|---:|
| reviewed ABI functions | 886 | 890 |
| prototype type positions | 3,019 | 3,044 |
| independent C/Rust measurements | 1,236 | 1,241 |
| layouts | 71 | 71 |
| callbacks | 8 | 8 |
| constants | 397 | 400 |
| mismatches | 0 | 0 |

Runtime evidence is the HEADLESS ABI 0.20 artifact with SHA-256
`092b2d80a775f39a6ad872d084bc09492576c82ac33641faeb4a3036c7fc347b`. The
streaming upload is proved by reading the buffer back and checking that the
vertex landed in slot one with slots zero and two unchanged; the per-player
keyboard snapshot is proved by checking all four slots against `GetState`; the
clear masks are proved by executing each of them and by CNA still rejecting a
non-finite depth through the safe wrapper.
