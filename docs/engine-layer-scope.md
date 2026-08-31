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
