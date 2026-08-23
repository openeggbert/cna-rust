# CNA-Rust measured implementation plan

Status date: 2026-08-23

Strict target: XNA 4.0 Windows runtime Rust API projection

MSRV: Rust 1.74

## Governing constraints

- Preserve authoritative XNA identifiers and casing below
  `cna::Microsoft::Xna::Framework`.
- Keep private implementation modules idiomatic, the compatibility allowlist
  empty, and every structural/safety zero as a permanent gate.
- Add complete ownership/dependency families with real managed or reviewed CNA
  behavior; never add signature-only placeholders.
- Treat the generated `typeScoreboard` as the only authoritative work queue.

## Current strict measurement

| Measurement | Graphics-run baseline | Current |
|---|---:|---:|
| XNA reference types | 257 | 257 |
| XNA reference members | 2,964 | 2,964 |
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

The 67 remaining diagnostics are only whole missing types:

```text
Graphics           0
Media             24
Audio             19
Design            13
Framework/core     4
Input               3
Storage             3
GamerServices       1
```

## Completed dependency chain

- [x] Completed the Model graph: Model, bones, meshes, mesh parts, collections,
  enumerators, effects, transforms, tags, stable identity, validation, and
  parent invalidation.
- [x] Implemented Model.Draw through ordinary VertexBuffer, IndexBuffer,
  EffectPass.Apply, and GraphicsDevice indexed-draw routes. No second native
  Model renderer exists.
- [x] Added the ordinary managed Model XNB reader with shared-resource fixups,
  rollback, cache/unload/reload, and legal synthetic graph evidence.
- [x] Completed BasicEffect, AlphaTestEffect, DualTextureEffect,
  EnvironmentMapEffect, and SkinnedEffect through their distinct CNA stock
  effect APIs, including clone, pass application, texture retention, defaults,
  validation, and XNB readers.
- [x] Completed IEffectFog, IEffectLights, IEffectMatrices, and stable
  parent-owned DirectionalLight views without duplicate public state.
- [x] Completed Texture3D with typed Color transfers, mips/boxes/windows,
  validation, disposal, and a built-in XNB reader. The reviewed native route is
  bound; qualified HEADLESS construction remains explicitly backend-blocked by
  CNA error 6.
- [x] Completed OcclusionQuery with real create/begin/end/completion/result/
  destroy routes and state-machine validation.
- [x] Completed IGraphicsDeviceService and the three mapped graphics exception
  support types under the normative Rust error policy.
- [x] Audited all new Graphics content readers. TextureCube now also has its
  missing built-in reader; legal Texture3D/TextureCube fixtures exercise normal
  construction or explicit backend failure and rollback.
- [x] Kept XNA LZX explicitly unimplemented rather than landing a partial or
  MonoGame-specific decoder.
- [x] Rechecked repeated RunOneFrame/Tick and canonical CNA HEAD. Neither
  unrelated blocker was allowed to weaken Graphics completion.

## Ownership model

Model owns the complete graph. Child facades use stable `Arc` identity; bone
parents and mesh-part back-links are `Weak`, and the shared lifetime contains
no facade back-link, so there is no strong cycle. Buffers and Effects retain
their existing single native owners. Shared mesh parts refer to those owners
without constructing aliases that can destroy the same handle.

DirectionalLight objects are parent-owned stock-effect views. They retain the
parent ResourceState, never own the Effect, and are invalidated after parent
disposal. ContentManager records the Model last so reverse unload invalidates
the graph before releasing effects and buffers. Parent shutdown, retained
children, repeated disposal, partial XNB failure, and shared resources are all
covered by crash-isolated stress.

See `docs/graphics-evidence.md` for the capability table and exact graph/XNB
evidence boundaries.

## Measured evidence

| Measurement | Previous | Current |
|---|---:|---:|
| named XNA-derived observations | 140 | 140 |
| assertions including final count | 141 | 141 |
| behavior failures | 0 | 0 |
| reviewed ABI functions | 235 | 347 |
| prototype type positions | 879 | 1,220 |
| independent C/Rust ABI measurements | 805 | 840 |
| layouts / callbacks / constants | 48 / 3 / 206 | 51 / 3 / 206 |
| ABI mismatches | 0 | 0 |
| native game lifetimes with a created game | 177 | 197 |
| owned native child-handle constructions | 283 | 893 |
| native crashes / observed double-free or UAF | 0 / 0 | 0 / 0 |

The behavior count deliberately did not grow: Model/stock-effect/query
construction and application require a native device and remain in native
stress instead of being described as platform-neutral XNA observations.
HEADLESS verifies native command paths, not visible 3D or shader output.

Canonical read-only CNA HEAD remains
`1bb2145d99ed572dd4eb15009c34e2e5f410fcf0`; its unmodified C API build blocker
at `CnaCApiCoreExt.cpp:250` remains renderer identity `49 == 50`. Runtime
evidence therefore continues to use the labelled experimental exact ABI-0.7
HEADLESS artifact. No exact ABI-0.7 sanitizer artifact was available;
sanitizer status is `not-run`, and crash absence is not leak proof.

## Next dependency-ordered work

1. Preserve Graphics zero and all structural/safety zeros on every subsequent
   change.
2. Consider XNA LZX only as a complete framed decoder with malformed-input,
   cleanup, cache, and unload evidence; do not land half a decoder.
3. Regenerate the scoreboard and prefer one complete small family among
   Framework/core, Input, Storage, or GamerServices. Do not start broad Audio
   or Media merely to reduce the type count.
4. Resolve repeated frame hosting only through a durable CNA callback context
   or a reviewed ABI user-data rebinding route.
5. Run ASan/UBSan only when an exact unmodified ABI-0.7 CNA artifact is
   available.

No post-Graphics small family was started in this run.

## Definition of complete compatibility

The selected profile is complete only when all mapped types and members and
every structural category reach zero; public-surface safety gates remain zero;
behavior is XNA-derived; native prototypes/layouts are compiler-verified;
ownership and sanitizer evidence pass; canonical CNA builds unmodified; and
every claimed platform has fresh runtime evidence.
