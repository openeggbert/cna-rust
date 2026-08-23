# Session evidence and next work

## 2026-08-23 — Remaining Graphics completion

### Exact strict handoff

```text
REFERENCE_TYPES                       257
REFERENCE_MEMBERS                    2964
EXPECTED_RUST_TYPES                   259
ACTUAL_RUST_TYPES                     192
TOTAL_DIAGNOSTICS                      67
MISSING_TYPES                          67
MISSING_MEMBERS                         0

CONSTRUCTOR_MAPPING_MISMATCH            0
OVERLOAD_MAPPING_MISMATCH               0
PROPERTY_MAPPING_MISMATCH               0
EVENT_MAPPING_MISMATCH                  0
BASE_PROJECTION_MISMATCH                0
TRAIT_MISMATCH                          0
INTERFACE_MISMATCH                      0
PARAMETER_MISMATCH                      0
RETURN_TYPE_MISMATCH                    0
GENERIC_MISMATCH                        0
GENERIC_BOUND_MISMATCH                  0
REF_OUT_MISMATCH                        0
ENUM_VALUE_MISMATCH                     0
FLAGS_MISMATCH                          0
DELEGATE_MISMATCH                       0
DISPOSAL_MISMATCH                       0
UNEXPECTED_TYPES                        0
UNEXPECTED_MEMBERS                      0
TYPE_KIND_MISMATCH                      0
INTERNAL_TYPE_LEAK                      0
RAW_HANDLE_LEAK                         0
PUBLIC_UNSAFE_API                       0
ALLOWLIST                               0
UNMEASURED_CATEGORIES                   0
```

Normal strict mode exits 1 only for the 67 genuine whole missing types.
Report-only generation records the same result and leak-only exits 0.

```text
Graphics           27 -> 0
Media              24
Audio              19
Design             13
Framework/core      4
Input                3
Storage              3
GamerServices        1
```

### Completed Graphics work

The complete 27-type queue was implemented rather than hidden:

- Model, ModelBone, ModelMesh, ModelMeshPart, all four collection types, and
  their four flattened enumerators;
- BasicEffect, AlphaTestEffect, DualTextureEffect, EnvironmentMapEffect,
  SkinnedEffect, DirectionalLight, IEffectFog, IEffectLights, and
  IEffectMatrices;
- Texture3D and OcclusionQuery; and
- IGraphicsDeviceService, DeviceLostException, DeviceNotResetException, and
  NoSuitableGraphicsDeviceException.

Model has stable public facade identity and a cycle-free ownership graph.
Strong ownership runs Model to collections/children/resources; bone parent and
mesh-part back-links are Weak. Vertex/index buffers and effects retain their
existing single owners. Model.Draw walks the graph through ordinary binding,
EffectPass.Apply, and indexed GraphicsDevice submission. Qualified HEADLESS
therefore proves the native command path, not visual Model rendering.

All five stock effects use their real CNA stock APIs. Construction, state,
clone, stable DirectionalLight children, pass Apply, texture retention,
disposal, and child-after-parent invalidation are exercised. They are
`STRICT_COMPLETE`, `NATIVE_CONSTRUCTION_VERIFIED`, and
`NATIVE_APPLY_VERIFIED`; shader pixels are `VISUAL_EXECUTION_NOT_QUALIFIED`.

Texture3D is strict-complete with exact Color encoding and full validation.
The ABI path is bound, but the qualified HEADLESS renderer rejects volume
storage at construction with CNA error 6, so native transfer is not claimed.
OcclusionQuery construction and its state machine are native-verified; the
reported PixelCount is CNA's real HEADLESS result, never a Rust fabrication.

### Content

The managed built-in table now includes Model, VertexDeclaration,
VertexBuffer, IndexBuffer, all five stock effects, Texture3D, and TextureCube.
The legal Model fixture has two bones, a hierarchy, a mesh, two parts, shared
real buffers, a shared BasicEffect, sphere, tags, and root identity. It covers
cache identity, unload/reload, transforms, shared fixups, wrong requested type,
malformed root, missing part effect, rollback, retained child invalidation, and
draw submission.

Legal standalone Texture3D and TextureCube XNB fixtures cover ordinary reader
activation, success/cache where the backend permits it, explicit backend
errors, and rollback. Only SurfaceFormat.Color is admitted because no other
format has an exact reviewed typed transfer route.

LZX status is `UNIMPLEMENTED`. Uncompressed Graphics content is complete; no
partial decoder or test-only asset switch was added.

### Behavior, ABI, ownership, and CNA

```text
XNA-derived observations                  140 -> 140
assertions including count                141 -> 141
failures                                         0

reviewed ABI functions                    235 -> 347
prototype type positions                  879 -> 1220
independent C/Rust measurements           805 -> 840
layouts / callbacks / constants       48 / 3 / 206 -> 51 / 3 / 206
mismatches                                       0

native game lifetimes                     177 -> 197
owned native child-handle constructions   283 -> 893
Model XNB/ownership cycles                       10
stock effects/Texture3D/query cycles              10
native crashes                                    0
observed double-free/UAF                          0
sanitizer status                            not-run
```

The managed behavior count remains unchanged because the new evidence needs a
native device and belongs in crash-isolated native stress. The full ABI
verifier checks all 347 exact prototypes, the loaded symbol set, layouts,
callbacks, enum/flag representations, and constants against canonical CNA.

Canonical CNA HEAD remains
`1bb2145d99ed572dd4eb15009c34e2e5f410fcf0`. Its unmodified build blocker at
`CnaCApiCoreExt.cpp:250` remains renderer identity `49 == 50`. CNA was not
modified, and the labelled exact ABI-0.7 HEADLESS artifact remains the
qualified runtime. Sanitizers were not run, so no leak-freedom claim is made.

Arbitrary repeated borrowed-game RunOneFrame/Tick remains upstream/backend
blocked: ABI 0.7 retains creation-time callback user data and has no reviewed
rebinding route. The minimum CNA addition is a durable caller-owned context or
an atomic callback user-data rebinding operation with explicit lifetime
semantics.

### Template and gates

The template source remains intentionally unchanged and continues to be the
small PNG/Texture2D/SpriteBatch/input/service/lifecycle canary. It is not a
Model showcase and makes no visible 3D claim. Fresh template tests, the
60-frame smoke, and the 600-frame stability run all pass with the qualified
HEADLESS artifact. A newly generated consumer vendors both crates, passes its
complete workspace tests, completes 60 native frames, contains no developer or
sibling-repository path, and contains no symlink back to the development tree.

### Next coherent slice

1. Keep Graphics at zero and preserve every structural/safety zero.
2. Implement LZX only if the complete XNA framing, decoder, malformed-input,
   rollback, cache, and unload slice can land together.
3. Prefer one complete small remaining family (Framework/core, Input, Storage,
   or GamerServices) after regenerating the scoreboard.
4. Do not start Audio or Media merely for breadth.
5. Seek an upstream callback-context lifetime/rebinding route for repeated
   ticks and an unmodified sanitizer-capable exact ABI-0.7 CNA artifact.

No optional small family was started during this Graphics run.
