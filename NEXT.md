# CNA-Rust next work

## 2026-09-01 — the binding-status axis, and five families decided end to end

The previous milestone closed the XNA 4.0 runtime profile and admitted ABI
0.21. This one is about the *other* question the census had never asked
separately: not what a route is for, but **why it is not bound**.

```text
CNANEXT_HEAD=e20749761c0d57cfcae5cc7ff57b76e94a78b319
CNANEXT_AT_QUALIFICATION=35268971c826d48ec3d40939e9b34a2b0595f94b
                             # the artifacts below were built from this;
                             # e2074976 is one net fix and does not touch
                             # modules/c-api/include, so the canonical surface
                             # measured here is unchanged
SHARP_RUNTIMENEXT_HEAD=bd282d101640005454639b372f67e119ffa5642b

ARTIFACT_ENGINE=cnanext/cmake-build-opengles3   # CNA_CNAEXT=ON, OPENGLES3
ARTIFACT_HEADLESS=cnanext/cmake-build-headless  # CNA_CNAEXT=OFF, HEADLESS
LIBRARY_SHA256=78f8933f9a84aa23f9ea8f3834ca9940f2744fd7c269e0dcc1fc79be7462e39b
LIBRARY_EXPORTS=4054
HEADER_EXPORTS=4054

CANONICAL_ROUTES=4054
BOUND=2909                   # was 2523
DELIBERATE_NON_BINDING=714   # was 457
BLOCKED_UPSTREAM=15
DEFERRED_TRACKED=0
UNREVIEWED=416               # was 1074
ACTIONABLE_LOCAL=0

RUST_SYS_DECLARATIONS=2924
SYMBOL_ACQUISITIONS=2920
LINKED_DECLARATIONS=2924
PROTOTYPE_MISMATCHES=0
SYMBOL_TYPE_MISMATCHES=0
LAYOUT_FIELD_SETS_CHECKED=179
C_RUST_MEASUREMENTS=3040
ABI_FINDINGS=0
UNAUDITED_DECLARATIONS=0

WORKSPACE_TEST_FILES=30
WORKSPACE_TEST_FUNCTIONS=159
BOUND_WITHOUT_SAFE_CALL_SITE=894   # reported, not gated; RUST-CENSUS-002
```

### The census now asks two questions instead of one

```text
purpose   what is this route for?      CNA_EXTENSION_BACKING, STRICT_XNA_BACKING, …
binding   why does Rust not bind it?   BOUND, DELIBERATE_NON_BINDING, BLOCKED_*, …
```

Conflating them had two consequences, both now fixed: 1,170 routes had *no*
purpose because `RUST_SYS_BOUND` short-circuited classification, and two header
rules had been deleted as "unused" for the same reason. A declared status needs
a reason; a block or a deferral needs a task as well; and a rule may carry
`rustEvidence`, Rust symbols the census greps for, so "Rust already does this"
cannot rot silently when the Rust it names is renamed.

The gate fails while anything is `UNREVIEWED` or `ACTIONABLE_LOCAL`. It
currently fails on 416 undecided routes, which is the honest state.

### Five families decided end to end

| Family | Bound | Deliberate | What was actually there |
|---|---|---|---|
| `models.h` | 80 | 30 | the Rust `Model` already existed; what was missing was CNA's *loaded* model and its glTF facts |
| `sensors.h` | 50 | 12 | a whole motion sensor, events, and the deterministic backends |
| `cnb.h` | 178 | 0 | the format itself: writer, cursor, chunks, codecs, `.cnj` |
| `effects.h` | 66 | 12 | `ShaderEffect`, `ColorMatrixEffect`, and what an effect can say about itself |
| `content_readers.h` | 14 | 12 | the content `Tag`; the reflective builder is unsafe to project |

### What this milestone found

Three defects in CNA, each with a reproducer that runs without this repository:

- **`RUST-UPSTREAM-021`** — destroying a content-loaded `Model` with a mesh
  part faults. `~MeshResource` moves an empty `detachedValue` over a loaded
  part's `value` and `~PartResource` dereferences it. Leaking the handle does
  not avoid it; the registry runs the same destructor at exit.
- **`RUST-UPSTREAM-022`** — a content-loaded skin's skeleton is unreachable,
  with a refusal the header does not document.
- **`RUST-UPSTREAM-020`** — the camera test backend leaves CNA's global
  platform override dangling. Re-measured, still reproduces.

And two in this crate, both found by asserting a value rather than a code:

- **The PBR texture slots were transposed.** `TextureSlot::index()` mapped
  Occlusion to 3 and Emissive to 4; the ABI is the other way round, so
  `PbrMaterialFull`'s per-slot coordinate set and transform silently read and
  wrote each other's slot.
- **`Accelerometer::inject` documented the wrong unit.** It takes metres per
  second squared and the reading comes back in `g`; injecting 9.80665 reads
  back 1.0. The gyroscope converts nothing, and now says so.

Two more measured facts that are not defects but were not written down:

- `cna_effect_get_graphics_device` resolves through the effect's *parent game*,
  so an effect on an independently constructed `GraphicsDevice` is refused.
- `cna_cnb_document_require_asset`'s second argument is the **highest** schema
  version a decoder understands, not the version it expects. Reading it the
  other way makes every decoder refuse every file older than itself.

### Do next

1. **The remaining 416 undecided routes.** `graphics_device.h` (40),
   `input_gamepad.h` (37), `runtime_components.h` (34), `devices.h` (34),
   `content.h` (30), `input_touch.h` (25) and a long tail. Each needs the same
   treatment the five families above got: read what is already in Rust before
   deciding anything is missing.
2. **`RUST-CENSUS-002`: 894 bound routes with no safe call site.** Reported
   rather than gated, because declaring a whole family so a missing symbol fails
   at load is deliberate and read-only projections legitimately leave the C
   mutators uncalled. Working the list down is what turns that from a claim into
   a measurement.
3. **The standing blockers.** A GPU-backed qualified artifact for the engine
   layer; a wasm toolchain for `RUST-PLATFORM-003`; a second machine for
   `RUST-BEHAVIOR-012`.
