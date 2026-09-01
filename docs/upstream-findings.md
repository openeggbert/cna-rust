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
