# CNA upstream findings measured from CNA-Rust

Defects in CNA itself, found while binding or qualifying it from Rust. Each one
is written so a later CNA session can reproduce it without this repository's
history, this chat, or any handoff note: exact symbols, exact commit, the
smallest sequence that shows it, what should happen, and what does.

None of these are worked around in Rust. A Rust wrapper that turned a crashing
native lifecycle into a tidy `Result` would hide the defect from the only people
who can fix it, and would leave every other CNA binding to rediscover it.

Re-measure a finding by running its reproducer against the current dependency
and updating the "last measured" line. If one stops reproducing, verify the fix
is semantic rather than incidental, then retire the finding and reclassify the
routes it blocks -- `tools/c-api-inventory/classification.json` names the
finding id, so the census will not let the change pass unnoticed.

---

## RUST-UPSTREAM-020 — a destroyed camera leaves CNA's platform override dangling

| | |
|---|---|
| Symbols | `cna_camera_create_with_test_backend_ext`, `cna_camera_destroy`, and every route that reads the platform camera list |
| Dependency | cnanext `35268971c826d48ec3d40939e9b34a2b0595f94b`, ABI 0.21.0 |
| Artifact | `cmake-build-opengles3`, `CNA_CNAEXT=ON -DCNA_DEVICES=ON` |
| Severity | Process fault. `SIGSEGV`, wait status 139 |
| Blocks | 15 routes, the whole `cna_camera_*` family |
| Last measured | 2026-09-01, reproduces |
| Related | CNA-Java reports the same family as `JAVA-UPSTREAM-019`. This finding was located and measured independently from Rust; the mechanism below comes from reading the current source, not from the Java report. |

### Mechanism

`cna_camera_create_with_test_backend_ext` (`modules/c-api/src/CnaCApiDevices.cpp`,
around line 2099) points CNA's **global** platform override at memory the camera
handle owns:

```cpp
auto resource = std::make_shared<CameraResource>();
resource->testState   = std::make_shared<CameraTestState>();
resource->testService = std::make_unique<TestCameraProvider>(resource->testState);
CNA::C::Detail::GetPlatformOverride().SetCamera(resource->testService.get());
```

`SetCamera` stores a bare `IPlatformCameraProvider*`
(`CnaCApiPlatformOverride.hpp:226`). `cna_camera_destroy` releases the handle,
which destroys `CameraResource` and with it the `unique_ptr` that owned that
provider -- and never clears the override:

```cpp
CNA_Result cna_camera_destroy(const CNA_CameraHandle camera)
{
    ...
    const CNA_Result result = CNA::C::Detail::GetRuntimeHandles().Release(camera);
    ...   // no SetCamera(nullptr) anywhere
}
```

Every later route that consults the platform camera list -- `cna_camera_get_count_ext`
reaches it through `Camera::getAvailableCamerasProperty()` -- dereferences the
freed provider.

The comment on `CameraResource::testService` says "the provider lives as long as
the camera handle does", which is exactly right and exactly the problem: the
override outlives the handle.

### Reproducer

`crates/cna/tests/upstream_camera_destroy.rs`. It runs the sequence in a child
process, because the failure is a fault rather than a result code and an
in-process test would take the suite down without proving anything repeatable.

```
CNA_NATIVE_LIBRARY=<cmake-build-opengles3>/modules/c-api/libcna_c_api.so \
  cargo test -p cna-rust --test upstream_camera_destroy -- --nocapture
```

Two stages, differing by one call:

| Stage | Sequence | Expected | Measured |
|---|---|---|---|
| `baseline` | create test camera, set state, read state, destroy | exits 0 | exits 0 |
| `after-destroy` | the same, then one `cna_camera_get_count_ext` | exits 0 | **`SIGSEGV`, wait status 139** |

The only difference is the call after destroy, so the fault is the override and
not the teardown.

### Why this is CNA rather than Rust

The Rust side holds no pointer into the camera resource and does nothing after
`cna_camera_destroy` but call another public route with a valid game handle.
The freed memory belongs to a CNA global that CNA set and CNA never cleared. No
ordering a caller could choose avoids it: the override is process-wide, so any
camera destroy poisons every later camera query in the process.

### What a fix would look like

`cna_camera_destroy` clearing the override it set -- guarding against clearing a
provider some *other* live camera installed -- or the override holding a
`weak_ptr`/`shared_ptr` so the pointer cannot outlive the resource.

### Status in this binding

The 15 `cna_camera_*` routes are `BLOCKED_UPSTREAM` in
`tools/c-api-inventory/classification.json`, owned by this finding.
`cna::extensions::devices::Camera` exists only so the reproducer can drive the
sequence and keep measuring it; it is deliberately not a projection to build on,
and its doc comment says so.

---

## RUST-UPSTREAM-021 — destroying a content-loaded Model dereferences a null part

| | |
|---|---|
| Symbols | `cna_model_destroy` on a handle from `cna_content_manager_load_model` |
| Dependency | cnanext `35268971c826d48ec3d40939e9b34a2b0595f94b`, ABI 0.21.0 |
| Artifact | `cmake-build-headless`, `CNA_GRAPHICS_RENDERER=HEADLESS` |
| Severity | Process fault. `SIGSEGV`, wait status 139, faulting address `0x490` |
| Blocks | The teardown of every loaded model that has at least one mesh part |
| Last measured | 2026-09-01, reproduces |

### Mechanism

`MeshResource::~MeshResource` (`modules/c-api/src/CnaCApiModels.cpp`, around
line 320) hands each part back its standalone copy as the mesh goes away:

```cpp
for (const std::shared_ptr<PartResource>& part : parts) {
    if (part->parentMesh != this) { continue; }
    part->parentMesh = nullptr;
    part->value = std::move(part->detachedValue);
}
```

That is right for a **hand-built** part, which has a `detachedValue` holding a
`ModelMeshPart` of its own, so the part keeps working after its mesh is gone.

A **content-loaded** part has no `detachedValue`. `MirrorLoadedModel` fills only
`value`, with an aliasing pointer into the loaded model:

```cpp
part->value = BorrowFromModel(model->value, nativePart);   // detachedValue stays empty
```

So for a loaded part the move above assigns an **empty** `shared_ptr` over a
perfectly good one. `~PartResource` then runs, two lines later in the same
teardown, and dereferences it:

```cpp
~PartResource()
{
    value->setTagProperty(nullptr);                      // no null check
    if (detachedValue != nullptr) {                      // ... but this one has it
        detachedValue->setTagProperty(nullptr);
    }
}
```

`ModelMeshPart::setTagProperty` is `tag_ = value`, and `tag_` sits past two
`std::array<SamplerState, N>` members, which is why the fault address is
`0x490` rather than `0x0`: it is a null `this` plus the offset of `tag_`.

### Reproducer

`tools/reproducers/ext015g_load_model_destroy.c` loads a model and destroys it;
`tools/reproducers/ext015g_handbuilt_mesh.c` is the control that builds the same
shape by hand. `tools/reproducers/README.md` has the build line; both take the
headless artifact:

```sh
gcc -O0 -g -rdynamic -D_GNU_SOURCE tools/reproducers/ext015g_load_model_destroy.c \
  -I<cnanext>/modules/c-api/include \
  -L<cnanext>/cmake-build-headless/modules/c-api -lcna_c_api \
  -o build-probe/ext015g_load_model_destroy
LD_LIBRARY_PATH=<cnanext>/cmake-build-headless/modules/c-api \
  ./build-probe/ext015g_load_model_destroy <content-root> <asset-name>
```

From Rust, `crates/cna/tests/upstream_model_destroy.rs` runs the load in a child
process, because the failure is a fault rather than a result code.

| Case | Result |
|---|---|
| load a glTF with one mesh part, destroy | loads, then `SIGSEGV`, 139 |
| load the same asset and never destroy it | reaches the end of its work, then `SIGSEGV`, 139 at exit |
| build a model with one mesh part by hand, destroy | exits 0 |

The first and third isolate it: *content-loaded* rather than *hand-built* is
what makes the difference, which is exactly what a missing `detachedValue`
predicts. The second is why nothing in the binding guards the teardown.

A loaded model with **no** mesh part would be the other useful control, and it
does not exist: CNA's importer refuses such a source outright -- "contains no
mesh instances to import" -- so every model the glTF path can produce has a
part.

### Why this is CNA rather than the binding

The middle case is pure C with no Rust in the process, and it uses only the
teardown the header documents: "the handles this route creates for them are
released when the model is destroyed -- do not release them by hand". A caller
following that sentence exactly is the caller that faults.

### What a fix looks like

Either half closes it, and both are one line:

* guard `value` in `~PartResource` the way the next line already guards
  `detachedValue`; or
* in `~MeshResource`, only take `detachedValue` when there is one --
  `if (part->detachedValue != nullptr) { part->value = std::move(part->detachedValue); }`.

The second is the closer fix: a loaded part's `value` is an aliasing pointer that
keeps the model alive on its own, so it does not need replacing at all.

### Leaking the handle does not avoid it

Measured, because it was the obvious first response and it is wrong. With
`cna_model_destroy` never called at all -- the model handle simply abandoned,
the content manager and the device both destroyed cleanly --
`tools/reproducers/ext015g_manager_teardown.c` still exits 139, with the same stack
and the same `0x490`. The C API's handle registry owns the `ModelResource`, and
its teardown at process exit runs the same `~MeshResource` / `~PartResource`
pair.

So a guard on the Rust side would not remove the fault; it would move it from a
place the caller can see to one they cannot, and would leave a test suite that
reports success and then dies. Nothing in the binding guards it.

### Status in the binding

`cna::extensions::native_model::NativeModel` is bound and every route answers
correctly -- loading, navigation, the import report, cameras, skins and material
variants. Only teardown is affected, and it is unavoidable, so the type's own
documentation says up front that loading a model with a mesh part will fault the
process before it ends.

The crate's tests for the type therefore run in a **child process** and read the
results back through its output, exactly as the camera reproducer does. When
this finding stops reproducing, those children will start exiting 0 and say so.

---

## RUST-UPSTREAM-022 — a content-loaded skin's skeleton is unreachable

| | |
|---|---|
| Symbols | `cna_model_create_skin_skeleton_handle_ext` |
| Dependency | cnanext `35268971c826d48ec3d40939e9b34a2b0595f94b`, ABI 0.21.0 |
| Artifact | `cmake-build-headless`, `CNA_GRAPHICS_RENDERER=HEADLESS` |
| Severity | Capability gap. A documented route refuses a documented input; no fault, no corruption |
| Blocks | Reading the skeleton of any skin CNA's own content pipeline imported |
| Last measured | 2026-09-01, reproduces |

### What happens

Load a glTF that declares a skin, ask `cna_model_get_skin_ext` about it -- it
answers `out_has_data = true`, so the skin names a skeleton -- then ask for that
skeleton:

```text
cna_model_create_skin_skeleton_handle_ext(model, 0, &data)
  -> CNA_RESULT_INVALID_STATE
     "The Model skin's skeleton was not created through the C API."
```

The header documents one refusal for this route, and it is a different one:
"`CNA_RESULT_INVALID_STATE` when the skin names no skeleton". This skin does
name one.

### Why it matters

`cna_model_add_skin_ext` and `cna_model_create_skin_skeleton_handle_ext` are a
matched pair, and they work for a skin a C caller added. The skins that carry
real data are the ones the importer produced, and those are the ones the route
will not answer for -- so a binding can see that a glTF scene has skins, how
many meshes each poses and what each is called, but never the joints.

### Reproducer

`crates/cna/tests/extensions_native_model.rs`,
`the_imported_skin_names_the_meshes_it_poses`. It asserts the refusal as
measured and fails if the skeleton ever becomes reachable, which is what will
say the finding can be retired.

### What a fix looks like

Either publish a `SkinningData` handle over the loaded skeleton the way
`cna_model_get_content_tag_dictionary_ext` publishes an aliasing handle over the
model's tag, or -- if that is deliberate -- say so in the header, because the
documented refusal does not currently cover this case.

### Status in the binding

`NativeModel::skin_skeleton` passes the refusal through rather than folding it
into `None`: "there is no skeleton" and "the skeleton exists and cannot be
reached" are different facts, and `ModelSkin::has_skeleton` already reports the
first.

---

## RUST-UPSTREAM-023 — concurrent `cna_graphics_device_create` corrupts the heap

| | |
|---|---|
| Symbols | `cna_graphics_device_create` |
| Dependency | cnanext `35268971c826d48ec3d40939e9b34a2b0595f94b`, ABI 0.21.0 |
| Artifact | `cmake-build-opengles3`, `CNA_GRAPHICS_RENDERER=OPENGLES3`; not reproducible on `cmake-build-headless` |
| Severity | Memory corruption. `SIGSEGV` or a glibc `double free or corruption` `SIGABRT`, in a call documented to fail cleanly |
| Blocks | Nothing permanently — the binding serialises construction — but it makes any multi-threaded C caller of this route unsafe |
| Last measured | 2026-09-01, reproduces |

### What happens

Six threads call `cna_graphics_device_create` at the same time, each with its
own `CNA_PresentationParameters` and its own output handle. Nothing is shared
between them at the ABI surface. About one run in five dies:

```text
double free or corruption (fasttop)
tcache_thread_shutdown(): unaligned tcache chunk detected
```

or a plain `SIGSEGV`. The rest of the runs report `CNA_RESULT_SUCCESS` on
every thread and exit cleanly, so the corruption is a race and not a refusal.

### Mechanism

From the `SIGSEGV` core, the whole path is upstream C++ and SDL:

```text
cna_graphics_device_create
  Microsoft::Xna::Framework::Graphics::GraphicsDevice::GraphicsDevice(...)
  GraphicsDevice::resolveRenderer()
  GraphicsDevice::createRenderer()
  CNA::Internal::Renderers::EasyGL::CreateGraphicsRendererForProfile(...)
  EasyGL::EasyGLRenderer::EasyGLRenderer(...)
  EasyGL::EasyGLPlatformContext::EasyGLPlatformContext(...)
  CNA::Platform::Sdl3::Sdl3GlContext::CreateContext(uint32_t, const GlContextDescription&)
  SDL_GL_CreateContext_REAL -> Wayland_GLES_CreateContext
  SDL_EGL_CreateContext -> SDL_EGL_MakeCurrent -> driBindContext -> dri_create_image
```

`Sdl3GlContext::CreateContext` does take a `std::mutex` — the `SIGABRT` core
catches a second thread blocked in `std::lock_guard` inside it — but the lock
does not cover the whole construction, so two threads still reach SDL's video
and EGL layer concurrently. SDL's video subsystem is not safe to call from
several threads at once, and Mesa's context binding underneath it is what
scribbles on the heap.

### Reproducer

`tools/reproducers/ext015h_concurrent_device_create.c`. Build it against an
artifact and run it repeatedly; `REPRO_THREADS`, `REPRO_NO_DESTROY`,
`REPRO_SERIALIZE_CREATE` and `REPRO_SERIALIZE_DESTROY` select the variants.

| Variant | Artifact | Aborts |
|---|---|---|
| 6 threads, create + destroy | OPENGLES3 | 13 / 70 |
| 6 threads, create only (handles leaked) | OPENGLES3 | 8 / 30 |
| 1 thread, create + destroy | OPENGLES3 | 0 / 30 |
| 6 threads, create + destroy | HEADLESS | 0 / 30 |
| 6 threads, create only | HEADLESS | 0 / 30 |
| 6 threads, **create serialised**, destroy free | OPENGLES3 | 0 / 40 |
| 6 threads, create and destroy both serialised | OPENGLES3 | 0 / 40 |

Two things follow from that table. Destroying is not implicated: leaking every
handle crashes just as often, and serialising destroy on top of create buys
nothing. And the renderer is: HEADLESS builds no GL context and never faults.

### Why this is CNA rather than the binding

It reproduces in twenty lines of C with no Rust in the process, and the entire
faulting stack is inside `libcna_c_api.so` and its own dependencies.

It is also a contract question, not only an implementation one. This ABI
already has a way to say "you called me from a thread you may not":
`CNA_RESULT_THREAD` / `CNA_ERROR_CATEGORY_THREAD`, defined in `abi.h` and
`core.h` as "invoked from a disallowed thread". `cna_graphics_device_create`
neither documents a thread affinity in its header block — which is otherwise
careful, covering several devices being live at once and one being destroyed
while another lives — nor returns that result. It corrupts the heap instead.

### What a fix looks like

Either of two, and the choice is upstream's:

- Serialise renderer construction internally, so the guarantee the header
  already implies ("several may exist at once") holds however they were made.
- Or declare the affinity: document that this route is main-thread-only, and
  answer `CNA_RESULT_THREAD` when it is not, which is what the result code is
  for.

Serialising is the smaller change and matches what the binding measured to be
sufficient.

### Status in the binding

`GraphicsDevice::new` holds a process-wide `CREATING_A_DEVICE` mutex across the
`cna_graphics_device_create` call and releases it immediately after, so devices
are still used and dropped concurrently. Safe Rust may not hand out a data race
that corrupts the heap, and the reproducer shows serialising construction alone
removes it.

The lock reaches only calls made through this crate. A process that also calls
`cna_graphics_device_create` directly — another binding in the same address
space, say — is not protected, which is why this stays an upstream finding
rather than a closed one.

This is what made `crates/cna/tests/extensions_effects.rs` fail intermittently
under a parallel run: five of its six tests build an independent device, and
the test harness runs them on separate threads. Before the fix that binary
aborted on 12 of 40 runs; after it, 0 of 40. The regression test is
`crates/cna/tests/upstream_concurrent_device_create.rs`.

---

## RUST-UPSTREAM-024 — the morph-target stride list is stale, and excludes every tangent-carrying layout

| | |
|---|---|
| Symbols | `cna_morph_target_data_ext_create`, `cna_morph_target_data_ext_blend`, `cna_model_mesh_part_set_morph_weights_ext` |
| Dependency | cnanext `35268971c826d48ec3d40939e9b34a2b0595f94b`, ABI 0.21.0 |
| Artifact | Renderer-independent; measured on `cmake-build-headless` |
| Severity | Capability gap with a documented cause. A clean `CNA_RESULT_INVALID_ARGUMENT`, no fault |
| Blocks | Morph targets on any physically based glTF mesh, through the C API |
| Last measured | 2026-09-01, reproduces |

### What happens

`ValidateMorphShape` in `CnaCApiModels.cpp:1168` opens with

```cpp
if (data.Stride != 32 && data.Stride != 52 && data.Stride != 56) {
    return InvalidArgument("Morph target stride must be 32, 52, or 56 bytes.");
}
```

CNA's renderer has one canonical table of what a stride means,
`InferredLayoutForStride` in `VertexDeclarationFidelity.hpp`, and it lists
eleven strides: 16, 20, 24, 32, 48, 52, 56, 60, 68, 76, 80. The C API takes
three of them. Measured from Rust:

```text
accepted [32, 52, 56]
refused 16, 20, 24, 48, 60, 68, 76, 80
  -- all: "Morph target stride must be 32, 52, or 56 bytes."
```

### Why those three

They are exactly the canonical layouts with **no tangent**. Every stride the
route accepts carries Position at 0 and Normal at 12 and stops there; every
stride it refuses that has a normal — 48 and 68 — carries a `Vector4` Tangent
at offset 24 as well.

| Stride | Position | Normal | Tangent | C API |
|---|---|---|---|---|
| 32 | 0 | 12 | — | accepted |
| 52 | 0 | 12 | — | accepted |
| 56 | 0 | 12 | — | accepted |
| 48 | 0 | 12 | 24 (`Vector4`) | refused |
| 68 | 0 | 12 | 24 (`Vector4`) | refused |

48 is the unskinned tangent layout and 68 the skinned one. Since GLTF-215
changed which effect a metallic-roughness material selects, those two are what
an ordinary PBR glTF mesh gets — so PBR morph targets cannot be handed to this
route at all.

### This list was already retired once, in the other half of the codebase

`BlendMorphTargetsEXT` in `modules/graphics/src/Xna/MorphTargetEXT.cpp` used to
hold the same literal and no longer does. Its own comment says why:

> This used to be the literal list {32, 52, 56}, written when those were the
> only strides a mesh with normals could have. GLTF-215 changed which effect a
> metallic-roughness material selects, and with it the strides an ordinary glTF
> mesh gets (48 unskinned, 68 skinned) — both of which carry Normal at offset
> 12 and neither of which was in the list, so every PBR morph target silently
> kept its base normals while its positions moved. Restating an ABI is what let
> that happen, so the predicate is now a query against the canonical stride
> table itself and cannot go stale again.

The fix (GLTF-278) landed in the blender and not in the C API's validator, so
the same restated ABI survives one layer up — and it now fails closed rather
than silently, which is better but still wrong.

### Why this is CNA rather than the binding

The literal is in CNA's own C source, it contradicts CNA's own canonical table,
and CNA's own comment argues against restating it. Nothing about the Rust
projection is involved: `MorphTargetData::new` passes the caller's stride
through unchanged and reports the refusal.

The header is silent on this too. `models.h` documents `stride` only as "byte
stride of one base-pose vertex" and names no permitted set, so a caller has no
way to learn the restriction except by being refused.

### What a fix looks like

Replace the literal in `ValidateMorphShape` with the query the blender already
uses — `InferredLayoutForStride(stride, UnlistedStrideLayout::RendererRefusesIt)`
and a check that the layout is `known` — so the validator and the blender agree
by construction. If a narrower set really is intended, `models.h` should say
which and why.

### Status in the binding

Reported as measured. `MorphTargetData::new` documents the restriction and the
reason in full, and `crates/cna/tests/upstream_morph_stride.rs` pins the
accepted set to `[32, 52, 56]` and fails when it changes — which is what will
say this can be retired. No Rust-side workaround: re-packing a caller's stride-48
vertices into stride 32 would drop their tangents, which is the very data the
refusal is about.

---

## RUST-UPSTREAM-025 — the only engine-layer getter that publishes an owned handle

| | |
|---|---|
| Symbols | `cna_area_light_brdf_table_get_texture` |
| Dependency | cnanext `35268971c826d48ec3d40939e9b34a2b0595f94b`, ABI 0.21.0 |
| Artifact | `cmake-build-opengles3`, `CNA_CNAEXT=ON` (the engine layer is required) |
| Severity | Contract inconsistency with a real consequence: the handle gates `cna_game_destroy` |
| Blocks | Nothing. The route works and the binding models it as it is |
| Last measured | 2026-09-01, reproduces |

### The anomaly

The header says the handle borrows:

> The handle **borrows**: it keeps the table alive while it exists, and
> releasing it releases only the handle, never the texture.

The implementation publishes it through `CreateOwnedTexture2D`
(`CnaCApiEngineLayer.cpp:20394`), over an aliasing `shared_ptr` that shares the
*table's* refcount while pointing at the texture.

Counted across the engine layer, that makes it the odd one out:

| Publisher | Routes |
|---|---|
| `CreateBorrowedRenderTarget2D` | 10, every one of them a getter: `cna_color_grade_pass_get_lut`, `cna_effect_get_shadow_map_ext`, `cna_effect_get_image_based_light_ext`, `cna_render_pipeline_get_scene_target`, `cna_clustered_forward_effect_get_opaque_frame`, `cna_weighted_blended_transparency_get_{accumulation,revealage}_texture_ext`, `cna_render_target_pool_acquire`, `cna_pbr_material_apply_state`, `cna_clustered_shadow_policy_select` |
| `CreateOwnedTexture2D` | 4: `cna_color_grade_pass_create_identity_lut`, `cna_cube_lut_create_strip_texture`, `cna_environment_processor_generate_brdf_lut` — and `cna_area_light_brdf_table_get_texture` |

Three of the four owned publishers *make* a texture the caller asked for. The
fourth is a plain `_get_`. It is the only getter in the module that hands back
an owned handle.

### Implementation, contract, or both — it is the contract

The implementation does what the prose promises. Measured from Rust, in
`crates/cna/tests/extensions_engine.rs`:

```text
NOTE: RUST-UPSTREAM-025 brdf texture: both answered true, sizes agree true,
      table readable after one handle dropped true,
      width after the table was released Some(32)
```

Two successive calls both answer and describe the same texture; dropping one
handle leaves the table fully readable, so releasing the handle really does
release nothing but the handle; and a handle held past
`cna_area_light_brdf_table_destroy` still reads its width, so the alias really
does keep the table alive. Every clause of the header's sentence holds.

What differs is the *kind* of handle, and that is not a naming detail:

```cpp
// CreateOwnedTexture2DWithKind, CnaCApiGraphics.cpp:503
if (parentGame != CNA_INVALID_HANDLE) {
    AddOwnedGraphicsResourceFor(parentGame);
}
```

`CreateBorrowedRenderTarget2D` does not make that call, and takes an explicit
`adapterLifetime` instead. And the counter it bumps is the one that gates
shutdown — `AddOwnedGraphicsResourceFor`'s own comment says "Only a game's
resources gate `cna_game_destroy`".

So a caller who reads the table's texture inside a game and does not destroy
the handle has, without being told, made the game undestroyable. Doing exactly
the same thing with `cna_effect_get_shadow_map_ext` — the same shape of call,
one page away in the same header — costs nothing. The handle also answers
`cna_texture2d_destroy` rather than `cna_render_target_destroy`, unlike all ten
of its analogues, so a caller who disposes of it the way the neighbours are
disposed of strands it.

### Why this is CNA rather than the binding

The choice of publisher is in CNA's C source, the counter it bumps is CNA's,
and the header sentence that does not mention any of it is CNA's. A binding can
only model the behaviour or misreport it.

### What a fix looks like

Either of two, and the choice is upstream's — they are not equivalent:

- **Publish it borrowed**, like the other ten getters, so a `_get_` route costs
  nothing to call and the handle is disposed of the way its neighbours are.
  This changes the handle kind, so it is an ABI-visible change.
- **Or keep it owned and say so**: state in the header that this handle is an
  owned `Texture2D`, that it counts against the game's graphics resources, and
  that it must be released with `cna_texture2d_destroy`. The word "borrows" is
  true of the *storage* and misleading about the *handle*, which is precisely
  the confusion worth removing.

### Status in the binding

`AreaLightBrdfTable::texture` wraps it with `Texture2D::from_owned_handle`, so
Rust's drop destroys the handle and the game's resource count returns to zero
on its own. The doc comment states the anomaly rather than smoothing it over,
because a caller who reaches past the binding needs to know. The measurement
above is asserted, so a change of behaviour upstream fails the test rather than
passing silently.

---

## RUST-UPSTREAM-026 — `cna_game_launch_parameters_add` neither adds-or-replaces nor refuses

| | |
|---|---|
| Symbols | `cna_game_launch_parameters_add` |
| Dependency | cnanext `35268971c826d48ec3d40939e9b34a2b0595f94b`, ABI 0.21.0 |
| Artifact | Renderer-independent; measured on `cmake-build-headless` |
| Severity | Contract mismatch that loses a caller's write silently and reports success |
| Blocks | Nothing. The binding reports the behaviour as measured |
| Last measured | 2026-09-01, reproduces |

### Three answers to one question

The header says:

> `@brief` Adds **or replaces** one launch parameter.

The implementation does not replace. `cna_game_launch_parameters_add` calls
`LaunchParameters::Add`, which is:

```cpp
void LaunchParameters::Add(const std::string& key, const std::string& value)
{
    // FNA's Dictionary<string,string>.Add throws on duplicate key; emplace silently ignores it.
    // Parse always guards with ContainsKey first, so this deviation is safe in practice.
    emplace(key, value);
}
```

`emplace` keeps the value already there. And XNA, which both are modelled on,
does a third thing: `Dictionary<string, string>.Add` throws
`ArgumentException` on a duplicate key.

So the same operation has three different contracts:

| | duplicate key |
|---|---|
| XNA / FNA | throws |
| CNA's C++ `LaunchParameters::Add` | keeps the first, silently |
| CNA's C header | says it replaces |

Measured from Rust — add `difficulty=hard`, then add `difficulty=easy`:

```text
CNA_RESULT_SUCCESS both times; the value afterwards is "hard"
```

The second call reports success and does nothing. A C caller has no way to
overwrite a parameter, and no way to learn that their write was dropped.

### The comment's own reasoning does not cover this route

The deviation is deliberate and annotated, and the annotation's argument is
that `Parse` guards with `ContainsKey` first, so `emplace` and a throwing `Add`
behave identically *there*. That is true of `Parse`. It is not true of
`cna_game_launch_parameters_add`, which is a public C entry point reaching the
same method with no guard in front of it — and the header in front of *it*
promises the opposite behaviour again.

### Why this is CNA rather than the binding

The header text, the C entry point and the C++ method are all CNA's, and they
disagree with each other. A binding can only report one of them.

### What a fix looks like

Pick one and make the other two agree:

- **Replace**, as the header says: use `insert_or_assign` in the C entry point
  or in `Add`. This is the most useful for a C caller, and the header already
  documents it.
- **Or refuse**, as XNA does: answer `CNA_RESULT_INVALID_STATE` for a key
  already present, and say so in the header. This keeps FNA parity and still
  lets a caller find out.

Either way the current combination — succeed, do nothing, say nothing — is the
one answer that gives the caller no signal at all.

### Status in the binding

`NativeLaunchParameters::add` documents the measured behaviour, not the
header's. `crates/cna/tests/extensions_game_runtime.rs` asserts that a second
add keeps the first value, so a fix upstream fails the test and says so.

The XNA-shaped Rust dictionary in `crates/cna/src/game/services.rs` is
unaffected and keeps XNA's refusal: `LaunchParametersExt::Add` returns an error
for a duplicate key, which is the third behaviour and the correct one for that
type. The two dictionaries are separate objects and this finding is about
CNA's.
