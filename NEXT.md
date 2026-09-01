# CNA-Rust next work

## 2026-09-01 — every route decided: UNREVIEWED reaches zero

The previous milestone closed the XNA 4.0 runtime profile and admitted ABI
0.21. This one asked the *other* question the census had never asked
separately -- not what a route is for, but **why it is not bound** -- and then
answered it for every route CNA exports. All 4,054 now carry a binding
decision, and the census gate passes for the first time.

```text
CNANEXT_HEAD=c195fe8ce616048930b7d565a2fc7d9d5de26d35
CNANEXT_AT_QUALIFICATION=35268971c826d48ec3d40939e9b34a2b0595f94b
                             # the artifacts below were built from this.
                             # cnanext has moved on since; re-measured against
                             # the canonical headers at HEAD, the exported
                             # surface is still 4054 routes and the ABI still
                             # 0.21.0, so the census here still describes it
SHARP_RUNTIMENEXT_HEAD=bd282d101640005454639b372f67e119ffa5642b

ARTIFACT_ENGINE=cnanext/cmake-build-opengles3   # CNA_CNAEXT=ON, OPENGLES3
ARTIFACT_HEADLESS=cnanext/cmake-build-headless  # CNA_CNAEXT=OFF, HEADLESS
LIBRARY_SHA256=78f8933f9a84aa23f9ea8f3834ca9940f2744fd7c269e0dcc1fc79be7462e39b
LIBRARY_EXPORTS=4054
HEADER_EXPORTS=4054

CANONICAL_ROUTES=4054
BOUND=3232                   # was 2909
DELIBERATE_NON_BINDING=801   # was 714
BLOCKED_UPSTREAM=15
DEFERRED_TRACKED=6           # RUST-EXT-016, RUST-EXT-017 (RUST-EXT-018 is a
                             # Rust-side defect, not a deferred binding)
UNREVIEWED=0                 # was 416
ACTIONABLE_LOCAL=0

RUST_SYS_DECLARATIONS=3247
SYMBOL_ACQUISITIONS=3243
LINKED_DECLARATIONS=3247
PROTOTYPE_MISMATCHES=0
SYMBOL_TYPE_MISMATCHES=0
LAYOUT_FIELD_SETS_CHECKED=187
C_RUST_MEASUREMENTS=3174
ABI_FINDINGS=0
UNAUDITED_DECLARATIONS=0

WORKSPACE_TEST_FILES=45
WORKSPACE_TEST_FUNCTIONS=203
BOUND_WITHOUT_SAFE_CALL_SITE=303   # reported, not gated; RUST-CENSUS-002
                                   # was 894, then 1077 once this milestone's
                                   # routes landed -- both wrong. The detector
                                   # looked for a table field named outside
                                   # native/, and most of the safe layer calls a
                                   # wrapper that names it instead. It is
                                   # two-hop now, and 303 is the first honest
                                   # reading of this number.
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

### Families decided end to end

| Family | Bound | Deliberate | What was actually there |
|---|---|---|---|
| `models.h` | 80 | 30 | the Rust `Model` already existed; what was missing was CNA's *loaded* model and its glTF facts |
| `sensors.h` | 50 | 12 | a whole motion sensor, events, and the deterministic backends |
| `cnb.h` | 178 | 0 | the format itself: writer, cursor, chunks, codecs, `.cnj` |
| `effects.h` | 66 | 12 | `ShaderEffect`, `ColorMatrixEffect`, and what an effect can say about itself |
| `content_readers.h` | 49 | 12 | the content `Tag`, the XNB reader, and the one of two extension points that is safe to project |
| `content.h` | 28 | 0 | what a manager has loaded, which reader served it, and the key CNA looks assets up by |
| `graphics_resource.h` | 12 | 0 | `Name` and `Tag` were Rust-side state CNA never heard about; four more properties have no XNA counterpart at all |
| `input_keyboard.h` | 10 | 2 | the physical layout: which key a scancode produces, and what the platform calls each |
| `input_mouse.h` | 9 | 1 | the desktop cursor, pointer lock, capture, and the clicked event with its test hooks |
| `texture.h` | 14 | 0 | CNA's format arithmetic, and textures held with no graphics device at all |
| `runtime.h` | 18 | 0 | the frame budget three ways, CNA's own launch parameters, and the title container |
| `video.h` + `media.h` | 21 | 0 | the `Video`, `Song` and `SongCollection` a game *builds* rather than is handed |
| `audio.h` + `xact.h` | 16 | 2 | the disposal facts, the renderer identity, and the float streaming path; two helpers deliberately left to Rust because CNA's arithmetic is not XNA's |

| the last of the ABI | 61 | 3 | `ContentLost`, the back buffer, the window controls, the joystick hotplug hooks, the storage root, the assembly title -- the tail that took the census to zero |

**Every header now stands at zero undecided routes.**

### What this milestone found

Three defects in CNA, each with a reproducer that runs without this repository:

- **`RUST-UPSTREAM-021`** — destroying a content-loaded `Model` with a mesh
  part faults. `~MeshResource` moves an empty `detachedValue` over a loaded
  part's `value` and `~PartResource` dereferences it. Leaking the handle does
  not avoid it; the registry runs the same destructor at exit.
- **`RUST-UPSTREAM-022`** — a content-loaded skin's skeleton is unreachable,
  with a refusal the header does not document.
- **`RUST-UPSTREAM-027`** — CNA's sample-duration and sample-size helpers are
  not XNA's. The duration truncates where `TimeSpan.FromMilliseconds` rounds,
  and answers *zero* for a buffer XNA says lasts a millisecond; the size drops
  XNA's frame alignment and does the rate division in the wrong precision. The
  Rust projection already implements both faithfully, so the C routes are a
  deliberate non-binding: binding them would have replaced a correct answer
  with a wrong one.

- **`RUST-UPSTREAM-026`** — `cna_game_launch_parameters_add` has three
  contracts and honours none of them. The header says "adds or replaces", the
  implementation's `emplace` keeps the value already there, and XNA's own
  dictionary throws. The second add reports success and does nothing, so a C
  caller can neither overwrite a parameter nor learn that the write was lost.

- **`RUST-UPSTREAM-024`** — the morph-target stride list restates a literal
  `{32, 52, 56}` where CNA's canonical table lists eleven. The three it takes
  are exactly the tangent-less layouts, so PBR morph targets (stride 48 and 68)
  are unreachable through the C API. The blender had the same literal and was
  fixed; the C API's validator was not.

- **`RUST-UPSTREAM-025`** — `cna_area_light_brdf_table_get_texture` publishes an
  owned handle where every other engine-layer getter publishes borrowed. The
  aliasing behaves as documented; what is wrong is the handle kind, which gates
  `cna_game_destroy`. The contract, not the implementation.

- **`RUST-UPSTREAM-023`** — `cna_graphics_device_create` races with itself.
  Six threads in it at once corrupt the heap on the GL renderers; the stack is
  CNA's own, down through SDL3 into EGL. `GraphicsDevice::new` now serialises
  construction, which the reproducer measures to be sufficient, so no route is
  blocked — but any other caller in the same process still is. This is what
  made `extensions_effects` fail intermittently under the parallel suite.

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

### The full handoff

`docs/handoff-2026-09-01.md` is the thirteen-section record for this milestone:
start state and where the reported numbers were wrong, dependency identity, the
scoreboard, every family, the reasoning behind the 801 deliberate
non-bindings, blockers, the eleven defects found, the mutation-style evidence,
the qualification matrix, documentation changes, git state, the metric that
changed meaning, and the next frontier.

### Do next

1. **`RUST-CENSUS-002`: the 303 bound routes with no safe call site.** Now a
   number worth working, because it is finally measured correctly. Each one
   needs either a safe caller or a stated reason; the justified list in
   `classification.json` is where the reasons go. Each needs the same treatment the families above got: read what
   is already in Rust before deciding anything is missing. Twice in this
   milestone that inverted the plan -- `runtime_components.h` and the touch
   value operations turned out to be Rust already -- and twice it found the
   opposite: `Name` and the launch parameters were Rust-only state that CNA
   could not see.
2. **The three deferred families.** `RUST-EXT-016` (adoption constructors for
   `SpriteFont` and `SoundEffect`), `RUST-EXT-017` (make the XACT disposing
   event fire on engine teardown, then bind CNA's), `RUST-EXT-018` (make the
   XNA-shaped `ContentLost` handlers fire by bridging them onto CNA's). All
   three are the same shape: a Rust event that does not fire where XNA's would,
   and a native event that does.
3. **The standing blockers.** A wasm toolchain for `RUST-PLATFORM-003`; a
   second machine for `RUST-BEHAVIOR-012`. The GPU-backed engine artifact is no
   longer one: `cnanext/cmake-build-opengles3` is built with `CNA_CNAEXT=ON`
   and the engine suite runs against it.
