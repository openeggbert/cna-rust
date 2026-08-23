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

## Final strict measurement for this run

| Measurement | Run baseline | Final |
|---|---:|---:|
| XNA reference types | 257 | 257 |
| XNA reference members | 2,964 | 2,964 |
| expected mapped Rust types | 259 | 259 |
| actual strict Rust types | 117 | 165 |
| total diagnostics | 178 | 94 |
| missing types | 142 | 94 |
| missing members | 36 | 0 |
| constructor mapping mismatches | 1 | 0 |
| overload mapping mismatches | 17 | 0 |
| property mapping mismatches | 4 | 0 |
| event mapping mismatches | 0 | 0 |

Base projection, trait, interface, parameter, return, generic,
generic-bound, ref/out, enum/value, flags, delegate, disposal, unexpected
type/member, type-kind, internal-type leak, raw-handle leak, public unsafe API,
allowlist, and unmeasured-category counts are all zero.

The 94 remaining diagnostics are only missing types:

```text
Graphics          27
Media             24
Audio             19
Design            13
Framework/core     4
Input               3
Storage             3
GamerServices       1
```

## Primary scoreboard outcome

| Strict type | Run baseline | Final |
|---|---:|---:|
| `Game` | 2 | 0 |
| `GraphicsDevice` | 26 | 0 |
| `SpriteBatch` | 8 | 0 |

Every one of the 48 added strict types has zero local diagnostics: the complete
selected Content family; typed vertex/index declarations and buffers;
`TextureCube`; render targets; `SpriteFont`; and the base Effect reflection
graph.

## Completed implementation

- [x] Added a real typed `ContentManager`/`ContentReader` XNB pipeline with
  validated uncompressed framing, reader tables and versions, activation,
  existing instances, shared resources, external references, typed caching,
  disposal tracking, rollback, custom readers, and primitive/value readers.
- [x] Wired one stable per-game `ContentManager`, cleared both final `Game`
  members, and preserved child-content disposal before native parent teardown.
  Raw PNG `Texture2D::FromStream` remains separate from XNB.
- [x] Added real Texture2D and SpriteFont XNB readers. The legal synthetic
  SpriteFont fixture constructs its atlas through the normal Texture2D reader,
  preserves cache identity, measures text, submits glyph draws, and unloads
  without double ownership.
- [x] Audited repeated `RunOneFrame`/`Tick`. CNA ABI 0.7 exposes no callback
  user-data rebinding route, so arbitrary repeated borrowed-game ticks remain
  an explicit safe backend blocker; the owned one-session host remains intact.
- [x] Added exact typed vertex declarations, five built-in vertex layouts,
  vertex/index and dynamic buffers, transfer validation, binding identity,
  wrong-device/disposed checks, and refusal to destroy a bound buffer.
- [x] Completed all `GraphicsDevice` buffer properties, typed `DrawUser*`,
  bound draw/instanced routes, render-target binding, reset/present, and
  back-buffer overloads through real CNA calls. HEADLESS-specific unsupported
  paths return exact errors instead of success.
- [x] Added the complete base `Effect` ownership/reflection family, typed
  scalar/vector/quaternion/matrix/string/texture parameter access, stable
  child/collection identity, clone/material support, real `EffectPass.Apply`,
  and reviewed tooling extensions for constructing a native reflection graph.
- [x] Cleared both Effect-bearing `SpriteBatch.Begin` overloads without
  ignoring the Effect, including `None`, matrix, device, disposed, and recovery
  validation.
- [x] Added real `SpriteFont` state, atlas ownership, measurement, and all six
  `DrawString` projections backed by CNA glyph submission.
- [x] Added a legal uncompressed Effect XNB probe. Its managed reader pipeline
  is verified; compiled Effect construction is truthfully backend-blocked by
  the current HEADLESS renderer (`CNA error 6`).

## Evidence

| Measurement | Run baseline | Final |
|---|---:|---:|
| named XNA-derived observations | 123 | 140 |
| assertions including final count | 124 | 141 |
| reviewed ABI functions | 104 | 235 |
| prototype type positions | 388 | 879 |
| independent C/Rust ABI measurements | 419 | 805 |
| layouts / callbacks / constants | 19 / 3 / 129 | 48 / 3 / 206 |
| ABI mismatches | 0 | 0 |
| native game lifetimes with a created game | 146 | 177 |
| owned native child-resource constructions | 103 | 283 |
| native crashes / observed double-free or UAF | 0 / 0 | 0 / 0 |

The native suite includes ten buffer-binding cycles, ten SpriteFont/atlas/XNB
cycles, ten Effect parent/child/clone/material cycles, and one compiled-Effect
XNB backend-failure cycle. The managed corpus adds Content metadata/cache and
vertex declaration/value observations. Native-dependent draw, reflection, and
font measurement evidence remains in crash-isolated native stress rather than
being mislabeled as a platform-neutral text observation.

Linux x86-64 experimental ABI-0.7 HEADLESS template tests and fresh 60/600
frame runs pass. A fresh generated consumer vendors both crates, contains no
developer or sibling path, passes its workspace tests, and completes 60 native
frames. The canary continues to exercise real PNG, Texture2D, SpriteBatch,
input, service identity, and clean shutdown; it makes no windowed-rendering or
XNB-template claim.

Canonical read-only CNA HEAD remains
`1bb2145d99ed572dd4eb15009c34e2e5f410fcf0`; its unmodified C API build blocker
at `CnaCApiCoreExt.cpp:250` is still the renderer identity assertion
`49 == 50`. Runtime evidence therefore continues to use the labelled
experimental exact ABI-0.7 HEADLESS artifact. No exact ABI-0.7 ASan/UBSan CNA
artifact was available, so sanitizer status is `not-run`; crash absence is not
claimed as allocator-level leak freedom.

## Next dependency-ordered work

1. Add Model only now that buffers, Effect, and ContentReader are real; retain
   parent-owned bone/mesh/part/effect identity.
2. Add Texture3D and the stock effects through reviewed CNA routes, then extend
   graphic XNB readers without side channels.
3. Add uncompressed remaining content readers before optional XNA LZX. Do not
   substitute a MonoGame-specific compression format.
4. Resolve repeated frame hosting only through a durable CNA callback context
   or a reviewed ABI rebinding route.
5. Continue the generated missing-type scoreboard; Audio/Media remain behind
   the current graphics/content foundations rather than displacing them.
6. Run ASan/UBSan when canonical CNA can produce an unmodified, instrumented,
   exact ABI-0.7 artifact.

## Definition of complete compatibility

The selected profile is complete only when all mapped types and members and
every structural category reach zero; public-surface safety gates remain zero;
behavior is XNA-derived; native prototypes/layouts are compiler-verified;
ownership and sanitizer evidence pass; canonical CNA builds unmodified; and
every claimed platform has fresh runtime evidence.
