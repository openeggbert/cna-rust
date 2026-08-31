# The engine-layer scope decision (RUST-EXT-010)

Decision date: 2026-08-31. Measured against cnanext `599d14e5`, ABI 0.21,
engine layer revision 2, on the qualified HEADLESS artifact.

## The measurement

`engine_layer.h` declares **857** canonical routes across **224** families.
Forty-nine are bound. The other 808 are not.

They are not blocked. This was measured rather than assumed, because "headless
must refuse it" is the comfortable answer and it is wrong:

```text
independent GraphicsDevice          SUCCESS
cna_render_pipeline_create          SUCCESS
cna_shadow_map_create               SUCCESS
cna_particle_system_create          SUCCESS
cna_pbr_material_extensions_create  SUCCESS
engine layer revision               2
```

Every engine object this probe tried **constructs** on the qualified artifact.
So the remaining 848 routes are genuinely reachable, and the question of
whether to bind them is a product question rather than a platform one.

## Why the answer is not "bind them"

**Scale.** 808 routes is more than half the entire surface bound today
(1,591 total). Binding the layer wholesale would roughly double the crate for
one subsystem.

**What could actually be asserted.** The objects construct, but they do not
render: the same probe reads a GPU memory estimate of **0**, because a headless
device allocates no GPU memory. A shadow map that draws nothing cannot be
tested for drawing the right thing. Tests for those 808 routes would assert
that a call returned success and a handle was non-zero -- exactly the assertion
quality this project rejects everywhere else, and exactly what "do not bind for
percentages" means.

**Two slices already exist, chosen on a criterion.** `PbrEffect` and its
material (`RUST-EXT-005`, `RUST-EXT-005b`) and the render-pipeline settings
(`RUST-EXT-010a`) are bound because their semantics *can* be asserted exactly
on this artifact:

- every PBR scalar round-trips through distinguishable values, all three alpha
  modes are walked, and an out-of-range texture-coordinate set is refused;
- `normalize` corrects exposure, gamma, bloom intensity and SSAO radius while
  leaving the integer counts alone, and the test asserts the exact pass-through;
- a quality preset moves the fields upstream has settled and no others;
- serialized settings report how many fields were recognised.

None of that is "it returned success".

## The decision

**The engine layer is bound one vertical slice at a time, and a slice
qualifies when its semantics can be asserted exactly on a qualified artifact.
Coverage is not the criterion and is not a goal.**

The 808 unbound routes are therefore `PRODUCT_DECISION_REQUIRED` for *this*
artifact: reachable, but not bindable to this project's evidence standard on a
device that renders nothing.

## What changes the answer

A GPU-backed qualified artifact. On one, most of the 224 families become
semantically testable -- a shadow map can be asserted to shadow, a bloom pass to
bloom, an SSAO pass to darken a crease -- and the slices below become actionable
in the ordinary way. That is the trigger, and it is a concrete one:

```text
current qualified renderer   HEADLESS
gpu memory estimate          0
```

The slices worth taking first when it arrives, in rough order of how much a
game needs them and how cleanly they can be asserted:

| Slice | Families | Why it is first |
|---|---|---|
| render pipeline begin/end + scene target | `render_pipeline` | everything else hangs off it, and pass counts and timings are exact values |
| shadow maps | `shadow_map`, `cascaded_shadow_map`, `spot_shadow_map`, `cube_shadow_map` | `did_shadow_pass_run` is a real assertion |
| post-process chain | `post_process_chain`, `post_process_pass`, `tonemap_pass`, `ssao_pass`, `ssr_pass` | back-buffer readback makes each pass's effect measurable |
| particles and decals | `particle_system`, `decal_pass` | counts and lifetimes are exact |
| GPU timers | `render_pipeline` timing routes | `is_gpu_timing_enabled` and per-pass names are values |

`cna_pbr_material_extensions` was the one family this decision identified as
testable **now** rather than on a GPU artifact, because it is pure value state.
Leaving it unbound while saying so would have made the criterion above a
rationalisation rather than a rule, so it was bound: `PbrMaterialExtensions`
covers clearcoat, sheen, transmission, volume attenuation, iridescence and
subsurface scattering, and the test walks every accessor with distinguishable
values, asserts the neutral-until-set property a renderer branches on, and
checks that a copy equals and hashes equal to its source. Its texture slots
stay out for the same reason `PbrMaterialFull`'s do: they are non-owning
handles, and a safe value holding one would be a raw-handle leak.

That leaves the criterion clean. Everything still unbound is unbound because
its semantics cannot be asserted here, not because it was inconvenient.

## 2026-08-31, later the same day: the trigger arrived

A GPU-backed artifact now exists and is qualified. `docs/gpu-evidence.md`
records it: an `OPENGLES3` build with `CNA_CNAEXT=ON` runs real frames on this
host's AMD Radeon 780M through the Rust binding, and `OPENGL33` and `VULKAN`
artifacts run the template's 60- and 600-frame canaries on the same hardware.

The discriminator this document named has moved:

```text
current qualified renderer   OPENGLES3 (AMD Radeon 780M, GL ES 3.2)
gpu memory estimate          230,400 bytes   (was 0)
```

That number is the pipeline's own scene target, 320 x 180 x 4 bytes, and the
test asserts it as exactly that rather than as "greater than zero". The frame
that filled it is read back off the GPU and every pixel equals the colour the
frame cleared to. That is the assertion class this document said a headless
device could not support.

`RUST-EXT-010b` is therefore no longer one row. It is decomposed family by
family, and each family is taken when its semantics can be asserted exactly --
the criterion above is unchanged, only its precondition has been met.

### Families bound since

| Slice | Families | Routes | Qualification |
|---|---|---:|---|
| render pipeline lifecycle, scene target, statistics, pass timings | `render_pipeline` | 27 | `VERIFIED_PIXEL` |
| directional-light shadow maps | `shadow_map`, plus the pipeline's shadow-scene pair | 24 | `VERIFIED_GPU` |
| post-process chain, passes, target pool, tonemap and FXAA | `post_process_chain`, `post_process_pass`, `blit_pass`, `render_target_pool`, `tonemap_pass`, `fxaa_pass` | 49 | `VERIFIED_PIXEL` |

### What the GPU artifact changed about the evidence

Three things could not have been measured on the headless artifact, and all
three found something:

- **The scene target is handed out only inside an open frame.** Upstream's
  `getSceneTarget` returns null unless `frameOpen_`, so a caller that asks
  between frames sees "none" on a pipeline that has one and is reporting its
  bytes. Nothing in the header says so; it was read from the implementation
  after the first test measured `None` next to a non-zero memory estimate.
- **A borrowed view is a handle, not a pointer.** `get_scene_target`,
  `get_shadow_map`, `get_shadow_texture` and `get_caster_effect` each publish a
  *new* handle that holds its owner alive and has to be released. Wrapping one
  in a non-destroying Rust view leaked it, kept the pipeline alive past its own
  device, and aborted the process at exit. Every borrow in this module now
  carries its owner's Rust lifetime and releases on drop.
- **The transparent-scene callback runs only when the transparency mode is not
  `None`,** which is the default. A registration that never fires looks exactly
  like a broken one.

