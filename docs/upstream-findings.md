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
