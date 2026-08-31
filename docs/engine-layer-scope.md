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
| GPU timers and particle systems | `gpu_timer`, `particle_system`, `particle` | 35 | `VERIFIED_GPU` |
| storage buffers and compute shaders | `storage_buffer`, `compute_shader`, `graphics_memory_barrier` | 23 | `VERIFIED_GPU` |
| decals, cube-map skies and the analytic sky | `decal_pass`, `skybox`, `atmospheric_sky` | 38 | `VERIFIED_STATE` |
| the seventeen screen-space passes, the fullscreen draw and the render-target scope | `bloom_pass`, `ssao_pass`, `ssr_pass`, `color_grade_pass`, `depth_of_field_pass`, `motion_blur_pass`, `height_fog_pass`, `volumetric_fog_pass`, `light_shaft_pass`, `lens_flare_pass`, `chromatic_aberration_pass`, `film_grain_pass`, `ascii_pass`, `aerial_perspective_pass`, `spatial_upscale_pass`, `contact_shadow_pass`, `fullscreen_pass`, `scoped_render_target` | 150 | `VERIFIED_STATE` |
| spot, cube and cascaded shadow maps, and the punctual light values | `spot_shadow_map`, `cube_shadow_map`, `cascaded_shadow_map`, `point_light_ext`, `spot_light_ext`, `punctual_light_ext`, `shadow_cascade_state_ext` | 61 | `VERIFIED_STATE` |
| the depth/normal prepass and both transparency paths | `depth_normal_prepass`, `weighted_blended_transparency`, `transparent_draw_list` | 49 | `VERIFIED_STATE` |
| HDR display output, auto exposure and `.cube` lookup tables | `hdr_display_output`, `auto_exposure_ext`, `cube_lut` | 39 | `VERIFIED_STATE` |
| debug drawing, frustum culling and levels of detail | `debug_draw`, `frustum_culler_ext`, `lod_group_ext` | 41 | `VERIFIED_STATE` |
| the clustered light set, the cluster grid and the light-to-cluster assignment | `clustered_light_ext`, `clustered_light_set`, `clustered_light_grid`, `clustered_light_assignment` | 43 | `VERIFIED_STATE` |
| the shadow budget, the light upload buffer and the compute sort | `clustered_shadow_policy`, `clustered_light_buffer`, `clustered_light_compute` | 30 | `VERIFIED_GPU` |
| the clustered forward effect | `clustered_forward_effect` (23 of 29; the other six wait on the PBR material extensions, the light probes and the area lights) | 23 | `VERIFIED_STATE` |
| light probes, image-based lights and the environment processor | `light_probe_ext`, `image_based_light_ext`, `environment_processor` | 34 | `VERIFIED_GPU` |
| probe volumes and the probe baker | `light_probe_volume_ext`, `light_probe_baker` | 27 | `VERIFIED_GPU` |
| the material extension texture slots, material identity, thin-film iridescence and the skinned PBR effect | `pbr_material_extensions_*_texture`, `pbr_material_ext`, `thin_film_iridescence`, `skinned_pbr_effect` | 31 | `VERIFIED_STATE` |
| the shader-effect factory, the glTF material bridge, the pipeline's skybox and the routes that were waiting on other families | `shader_effect_factory`, `gltf_material_*`, `render_pipeline_*_skybox`, the last `clustered_forward_effect` and `debug_draw` routes | 21 | `VERIFIED_GPU` |
| area lights, their BRDF table and the shading maths | `area_light_ext`, `area_light_brdf_table`, `area_light_shading`, `clustered_forward_effect_set_area_light` | 17 | `VERIFIED_GPU` |
| an effect's shadow, punctual-light and image-based-light slots | `effect_*_ext` | 16 | `VERIFIED_STATE` |

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
- **A GPU timer query resolves after the frame that closed it.** Polling
  straight after `end` collects nothing, and re-opening the range next frame
  re-issues the query -- so a loop that polls in the wrong order measures
  nothing for ever while reporting the timer as supported. Polling before the
  next range opens is what turned "supported, zero samples" into a measured
  0.007 ms of real GPU work.
- **`cna_particle_system_get_settings` validates its own output structure.** It
  is the one engine getter that refuses a destination whose `struct_size` and
  `struct_version` the caller has not filled, where every other getter fills
  them itself.
- **`cna_skybox_get_environment` publishes a handle too,** and a Rust
  `has_environment` that only compared it against `CNA_INVALID_HANDLE` leaked
  one per call. The leak was invisible until game shutdown, which then refused
  with "All owned C child resources must be destroyed before the game" and
  named nothing. Every engine query that answers with a handle is now a
  lifetime-bound view that releases on drop.
- **A handle is not a type.** `cna_ascii_pass_get_effect` publishes a handle to
  CNAEXT's `AsciiPostProcessEffect`, not to a generic `Effect`, and releasing it
  through `cna_effect_destroy` does not fail: the call succeeds, every test
  passes, and the process then calls `std::terminate` at exit with no
  diagnostic. It took a bisection over one test's own body to find, and the fix
  is a separate Rust view type so the wrong release cannot be written.
- **Two upstream argument orders disagree with their own doc comments.**
  `cna_weighted_blended_transparency_weight` takes `(view_depth, alpha, ...)`
  where the summary reads "for one fragment's depth and coverage" but the
  `@param` list is the authority, and `cna_height_fog_pass_optical_depth` puts
  the ray geometry before the fog parameters. Both compile silently either way
  because every argument is a `float`; the curves are what caught them -- fog
  integrated to zero over any distance, and the transparency weight *rose* with
  depth, which would have made the accumulation order-dependent.
- **A bare `BUFFER_TOO_SMALL` carries no message.** `CopyValueRange` returns the
  code without going through `Fail`, so the thread-local last-error still holds
  whatever the previous failing call said. Sizing the SSAO kernel from
  `sample_count` -- which is not its size -- therefore reported "The destination
  cannot hold the complete text" about a kernel of vectors.
- **`atmospheric_sky`'s `sun_direction` is the direction the light travels,**
  the same convention `DirectionalLightEXT` uses, and the opposite of what "the
  direction the sun is in" in the header suggests. Measured by sweeping five
  view directions at one elevation: the sky is dimmest at right angles to the
  vector and brightest at the far end of the sweep.
- **`cull_transforms` is not "one local box, many instances".** The bounds array
  is parallel to the transforms and holds *world* bounds, and the tail rule is
  the surprising half: a transform with no bound of its own is **kept**, not
  dropped. A Rust wrapper that passed a single shared box therefore returned
  three of four transforms where the per-box test agreed on one, and the three
  extra ones were the unpaired tail sailing through. The header states the rule;
  it was the measurement that made anyone read it.
- **A LOD boundary is exclusive, and past the last one there is no level.**
  `selectIndex` is an `upper_bound`, so a distance sitting exactly on a level's
  `max_distance` belongs to the *next* level, and a distance beyond the coarsest
  level answers `-1` -- "draw nothing" -- rather than clamping to the coarsest.
  Both halves are the sort of off-by-one that a `>= 0` assertion would never see.
- **`select_index` is not a query.** It stores the level it chose, and with a
  hysteresis margin set, the next call near that level's boundary answers the
  remembered level instead of the one the distance falls in. Two identical calls
  with the same argument legitimately return different answers; the Rust doc
  says so and the test pins both sides of it around a three-unit margin.
- **Three clustered-lighting constructors take a graphics device, not a game.**
  `cna_clustered_light_set_create`, `cna_clustered_light_grid_create` and
  `cna_clustered_light_assignment_create` all name the parameter `game` and
  document it as "the owning game", and all three resolve it with
  `GetBorrowedGraphicsDevice`, taking the game from the device afterwards.
  Passing the game handle answers "the graphics-device handle is invalid for
  this call" -- a `Handle` error naming a parameter the header does not mention.
  The Rust constructors take `&GraphicsDevice` and say why.
- **A cluster grid's depth slices are logarithmic, and there is one more
  boundary than slice.** With a near plane of 1 and a far plane of 100 over
  eight slices the boundaries are `1, 1.778, 3.162, 5.623, 10, 17.78, 31.62,
  56.23, 100` -- exactly `near * (far/near)^(i/N)`. A linear split passes every
  monotonicity check and fails this one. Placing a distance is *clamped* at both
  ends rather than refused, so a light straddling the frustum edge still lands
  in a slice.
- **A cleared assignment keeps its shape invariant.** `clear` leaves zero
  clusters and zero references but still one offset -- the single zero that
  empties them -- because the offset array is always one longer than the cluster
  count. A test asserting "everything is zero after clear" reads that as a bug.
- **A shadow-budget score is zero beyond the light's own range,** because the
  policy ranks with the same windowed inverse square the clustered shader
  shades with, and that window is zero at `distance >= range`. Five casters
  spread from 5 to 45 units with a range of 8 therefore produced *one*
  candidate, not five, and a budget of two admitted one light while refusing
  four. The fix in the test was to put each caster inside its own range; the
  finding is that "in the frustum" is not the same question as "reaches the
  camera", and only the second one scores.
- **The compute sort really runs on this host, and agrees with the CPU.**
  `cna_clustered_light_compute_is_supported` answers true on the OPENGLES3
  artifact and `used_compute` confirms the GPU path ran; its cluster offsets
  are identical to the CPU sort's and its light references are the same
  multiset. The documented CPU fallback could **not** be measured here: every
  artifact on this host either has the engine layer *and* compute, or neither,
  so `NOT_MEASURED_HERE` is the honest label for the fallback branch rather
  than `VERIFIED`.
- **`EngineHandle::release` was clearing the slot before the destroy, and CNA
  refuses some destroys.** Upstream declines to destroy an object while a
  counted borrow taken from it is outstanding -- the clustered forward effect's
  shader effect is one -- and this module's release had already replaced the
  handle with `INVALID` by the time it saw that refusal. The only handle anyone
  held to a live native object was gone: every later call answered "has been
  released", the object stayed owned by the game, and the process aborted at
  exit with `terminate called without an active exception`. Release now clears
  the slot **only after** the destroy has succeeded, so a refused destroy leaves
  the value callable. This was reachable by every family in this module, and it
  took a family whose refusal is *documented* to surface it.
- **The clustered forward effect's clamps are three different rules, all
  measured.** Base colour and metallic clamp to zero-to-one; roughness clamps
  to **0.04**-to-one, because a perfectly smooth surface collapses the specular
  lobe to a point the shader cannot integrate; and the ambient term is only
  *floored* at zero -- a channel above one survives, because a brighter-than-
  white ambient is a choice while a negative one would subtract light that was
  never added. A single "clamped to 0..1" assumption would have been wrong for
  two of the four.
- **The environment processor's three cube generators run on this host.** The
  equirectangular conversion, the cosine-convolved irradiance cube and the
  GGX-prefiltered specular chain all produce real cube maps at the sizes they
  were asked for on the OPENGLES3 artifact -- the header warns that a renderer
  without cube storage answers `NOT_SUPPORTED`, and this one does not. The
  test records the outcome per generator rather than assuming either way, so
  the same test is honest on a renderer that refuses.
- **A light probe has no parent at all.** `cna_light_probe_ext_create` takes no
  device and no game and calls no `AddOwnedGraphicsResourceFor`, so unlike
  every other owned engine object in this module a probe registers with no
  device: its lifetime is entirely the Rust value's. Registering it anyway
  would have been harmless but wrong, and the difference is only visible in
  upstream's source.
- **`Hammersley` samples stratum centres, not stratum edges.** The first
  coordinate is `(i + 0.5) / n`, so point zero is `0.125` for four samples
  rather than `0`. A test asserting `i / n` fails on every point; one asserting
  only "inside the unit interval" would never notice the sequence was
  half-a-stratum out.
- **A probe's visibility slots are `+X, -X, +Y, -Y, +Z, -Z` in that order and
  the weight blends across them by squared component,** so storing statistics in
  slot zero and then querying straight up reads *nothing* -- the answer is a
  confident `1.0`, meaning "nothing is known to be in the way", and looks
  exactly like a working query. Beyond the mean the weight is Chebyshev's
  `variance / (variance + gap^2)`, which the test pins to the value rather than
  to the direction of change.
- **`LightProbeVolume::set_probe` relocates the probe it stores.** The grid
  decides where a probe is, so the stored copy is moved to the cell's own
  position and the caller's probe is left alone. A probe written in at
  `(9, 9, 9)` comes back out at `(-1, -2, -3)` -- and because
  `cna_light_probe_ext_equals` compares the position, a round-trip
  `set_probe`/`get_probe` pair is **not** equal to what went in until the
  original is moved there too. Upstream's own comment says why: a probe placed
  somewhere else makes the interpolation weights describe one arrangement and
  the light another, and the result looks like the lighting is lagging behind
  the geometry.
- **`cna_light_probe_ext_equals` does not compare visibility, although its
  header says it does.** The canonical `operator==` tests the position and the
  nine coefficients and stops; two probes differing only in their occluder
  statistics answer `true`. The Rust doc says what the call actually does and
  the test pins it, so nobody builds a "did the visibility bake change
  anything" check on it.
- **The probe baker works on this host, and a failing callback cannot stop
  it.** `is_supported` -- which CNA measures by rendering one capture at
  construction rather than asking the renderer -- is true on the OPENGLES3
  artifact, and a probe bake calls the scene callback exactly six times, a
  two-probe volume twelve. The C callback returns `void`, so a Rust closure
  that fails does not abort the capture: every remaining face still runs and
  the safe wrapper reports the Rust cause afterwards. The test asserts both the
  message and the six calls.
- **The baker's capture planes are validated as a pair and refused as a pair.**
  Three refused calls in a row -- a zero near plane, a negative one, and a far
  plane below the near one -- left both distances at the values the last
  accepted call set. An implementation that wrote the near plane before
  validating the far one would fail exactly here and nowhere else.
- **The nine material-extension texture slots are bindable after all.** An
  earlier note in this crate said they were "deliberately not exposed" because
  a safe Rust value holding one would be a raw-handle leak. That was true of a
  design that stored the handle; it is not true of the one this module has
  since settled on. `cna_pbr_material_extensions_get_*_texture` publishes a
  fresh handle through the same `CreateBorrowedRenderTarget2D` path the
  clustered forward effect's opaque frame uses, so the getter is a
  lifetime-bound `BorrowedRenderTarget`, and the setter *takes* the texture the
  way every other `RETAINED_DEPENDENCY` in this module does. The doc comment
  has been replaced rather than left standing.
- **`SkinnedPbrEffect` had no `Drop` and aborted the game at shutdown.** The
  type was written without one, and CNA counts an effect against the parent
  game's owned children: the first test to construct one failed with "All owned
  C child resources must be destroyed before the game" -- at teardown, naming
  nothing. Found the same way the earlier leaks were, and fixed the same way.
- **A thin film of zero thickness is the base reflectance *exactly*, which the
  glTF reference implementation is not.** Upstream departs from it on purpose:
  the reference clamps `R12 * R23` to a floor of 1e-5 rather than to zero, so at
  zero thickness the first interference order survives as a coloured residue of
  about 0.007 -- and a material carrying the extension with the film switched
  off would not be the material without it. A film whose index *matches* the
  medium around it is not short-circuited, though: the Airy summation still
  runs and shifts the colour by about 0.001, so "same index" is not "no film".
  Total internal reflection returns white.
- **CNA spells "no volume absorption" as zero, where glTF spells it
  `+Infinity`.** `cna_gltf_material_extension_source_ext_init` leaves
  `attenuation_distance_ext` at zero, a fresh `PbrMaterialExtensions` reads zero,
  the setter floors negatives to zero, and the shader gates absorption on
  `uAttenuationDistance > 0`. The convention is consistent -- but an importer
  that translates glTF's infinity literally lands on a very large finite
  distance and almost no absorption, which *looks* right, while one that
  translates "absent" to zero gets what CNA means. The bridge also writes a
  source's zero over a distance that had already been set, so zero is a value
  rather than "leave it alone".
- **glTF's own metallic and roughness defaults are one, not zero,** and the
  default index of refraction is 1.5 -- all three of which
  `cna_gltf_material_source_ext_init` reproduces. A staging structure zeroed by
  the caller would give a perfectly smooth dielectric where the format's default
  is a fully rough metal.
- **The shader-effect factory's compile count follows names, not requests.**
  Two acquires of the same name compile once; a second name compiles again. The
  factory refuses `clear` while any effect view it published is still alive and
  allows it once they are gone, which is the counted-borrow rule again.
- **A cluster-slice gizmo drawn from a grid with no projection adds no lines and
  succeeds.** There is nothing to place the slices with, and upstream chose an
  overlay that stays empty until the grid is ready over one that refuses. The
  test asserts the line count is *unchanged* rather than that the call returned
  `Ok`, which is the difference between measuring the rule and measuring nothing.
- **One texture accessor in the engine layer publishes an *owned* handle where
  every other publishes a borrowed one.**
  `cna_area_light_brdf_table_get_texture` goes through `CreateOwnedTexture2D`,
  not `CreateBorrowedRenderTarget2D`: the handle counts against the game's own
  children and is released with the texture destroy. Wrapping it in this
  module's `BorrowedRenderTarget`, which releases through
  `cna_render_target_destroy`, failed silently and stranded the child -- the
  game then refused to shut down with "All owned C child resources must be
  destroyed before the game". It is the same class of defect as the ASCII
  effect's handle earlier in this document, found the same way, and the fix is
  the same: the accessor answers a real `Texture2D` adopted with
  `from_owned_handle`. Two accessors that look identical in the header are not
  interchangeable, and only the implementation says which is which.
- **A tube area light shades as the rectangle its axes span; a disc shades as
  that rectangle scaled by `sqrt(pi)/2`.** All three shapes reduce to a
  quadrilateral, and the disc's is the *equal-area* rectangle, so the two share
  an area rather than a bounding box. The tube's quad is byte-identical to the
  rectangle's -- the difference between them is energy, not outline -- which a
  test asserting "the three shapes differ" fails on and a test asserting the
  numbers catches correctly.
- **The area-light lobe scale is `max(roughness^2, 0.02)`.** The floor is what
  stops a mirror-smooth surface collapsing the lobe to a point, the same reason
  the clustered forward effect floors roughness at 0.04. Pinned to the value
  rather than to the direction of change.
- **The BRDF table costs real time and says so.** A default 32x32 table with 64
  samples per entry took 7.8 ms to integrate on this host, and its own
  `generation_milliseconds` reports it -- which is why it is an object to build
  once and share rather than a call to make per frame.
- **`cna_effect_get_punctual_light_ext` never publishes the shadow handles.**
  It fills every other field and leaves `shadow_cube` and `shadow_map` at
  `CNA_INVALID_HANDLE` whatever was bound, deliberately: "this ABI does not
  invent a name for a texture it does not own". So `PunctualLight::has_shadow_*`
  read back from an effect is a *constant false*, and a test that treated it as
  a round-trip would be asserting nothing. The Rust binding knows the answer
  because it is what holds those textures, and publishes it as
  `EffectLighting::has_punctual_shadow_cube` / `has_punctual_shadow_map`; the
  doc on the getter says why it cannot come from CNA.
- **The lighting slots are a binding with a lifetime, not a set of free
  functions.** Every texture in them is a raw pointer CNA never releases, so
  `EffectLighting` holds the Rust resources and clears every slot it filled when
  it drops. Without that, an effect drawn after its shadow map was dropped would
  read freed memory -- and nothing in CNA would notice. The test asserts the
  clearing directly: after the binding goes out of scope, a fresh one reads back
  no shadow map and no image-based light.
- **Setting a punctual light's shadow of one kind clears the other.** The two
  slots are one structure, so binding a cube leaves the map empty and vice
  versa; there is no partial update.

