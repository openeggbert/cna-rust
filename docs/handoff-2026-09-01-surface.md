# CNA-Rust handoff — `RUST-SURFACE-001`, 2026-09-01

CNA's own members left the strict XNA types. The strict verifier went from
**110 diagnostics to 0** on both runtime profiles, and it did so by moving 109
members and one type rather than by learning to expect them.

## 1. Start state

```text
cna-rust            develop @ 84d8751, clean, in sync with origin/develop
cna-rust-template   develop @ 416642b, clean
cnanext             next    @ 7712534d3, clean, untouched
sharp-runtimenext   next    @ 9cc96cd57, clean, untouched

STRICT_SELECTED_TOTAL_DIAGNOSTICS = 110   (109 UNEXPECTED_MEMBER + 1 UNEXPECTED_TYPE)
STRICT_COMPLETE_TOTAL_DIAGNOSTICS = 110   (the identical finding set)
```

## 2. CPU policy

Every compile, test, verifier and canary this session started ran under
`taskset -c 0-3` with `CARGO_BUILD_JOBS=4`, `RUST_TEST_THREADS=4`,
`MAKEFLAGS=-j4`, `CMAKE_BUILD_PARALLEL_LEVEL=4` and
`CNA_MAX_VENDORED_BUILD_JOBS=4`, exported from one policy script sourced by
every command. Allowed affinity was 0-15; four were used. Nothing outside this
session was inspected, throttled or signalled.

Every process also ran against an Xvfb on `:77` with `SDL_VIDEODRIVER=x11`
and `GDK_BACKEND=x11` exported at the top level of that same script, never
against the user's session.

## 3. Dependency identities

Unmodified, and verified so at the end:

```text
cnanext            7712534d3d22c7e284714e0e87afebba3f3cb472   0 tracked modifications
sharp-runtimenext  9cc96cd57cde394940cc24d58743edf9bf63d3fb   0 tracked modifications

ARTIFACT_HEADLESS  cnanext/cmake-build-headless/modules/c-api/libcna_c_api.so
                   sha256 94078be94dc1f1e6c8787c1cd17b08c9430d1e4bb5699947cd2b7aafee40281d
                   the artifact docs/runtime-capabilities.md records; unchanged
ARTIFACT_ENGINE    cnanext/cmake-build-opengles3/modules/c-api/libcna_c_api.so
                   sha256 65ce46a49b754586e8a99406901a9627e38f4473b594c0266400db65e4d73da9
                   rebuilt by another agent at the same commit since the last
                   milestone; it is the engine artifact, not the recorded one
```

## 4. The decision

**CNA-only public surface lives behind CNA extension traits, not as inherent
members of strict XNA types.** The strict verifier was not weakened: no
allowlist entry, no new expected-member rule, no suppressed code, no reduced
feature set. It reports zero because the members are no longer there.

Two things deliberately not done. No deprecated inherent forwarder was left,
because a deprecated inherent member is still an inherent member and the
verifier would still report it. And nothing was renamed: the migration already
breaks a source file at its import line, and a naming change on top would have
turned every call site into a puzzle.

Each of the 109 was proved CNA-only before it moved, and the proof is stronger
than reading the finding: with `MISSING_MEMBER = 0` on both profiles, removing
the 109 leaves **exactly** the pinned XNA contract on all 47 affected types --
checked set-wise, type by type. No member that XNA declares was moved. One impl
block mixed the two -- `SoundEffect`, 23 members, of which `FromAsset` alone
was CNA's -- and only `FromAsset` moved.

## 5. Strict scoreboard

| | before | after |
|---|---:|---:|
| `TOTAL_DIAGNOSTICS` (selected profile) | 110 | **0** |
| `TOTAL_DIAGNOSTICS` (complete runtime profile) | 110 | **0** |
| `UNEXPECTED_MEMBER` | 109 | **0** |
| `UNEXPECTED_TYPE` | 1 | **0** |
| `MISSING_MEMBER` | 0 | 0 |
| `MISSING_TYPE` | 0 | 0 |
| every other measured category | 0 | 0 |
| `allowlistEntries` | 0 | 0 |
| `unmeasuredCategories` | 0 | 0 |

Selected profile: 257 reference types, 2,964 reference members, 259 expected
Rust types, 74 out of profile. Complete runtime profile: 331 reference types,
3,640 reference members, 333 expected Rust types, 0 out of profile. Leak-only
gate: 0 internal leaks, 0 raw handles, 0 public unsafe.

**Feature set.** `cargo rustdoc -p cna-rust --lib` compiles default features
(`dynamic-loading`). That is not a reduced surface, and it is measured rather
than asserted: **1,030 publicly nameable paths under default features and the
same 1,030 under `--all-features`**, empty difference both ways, and both gates
zero on both rustdoc runs.

## 6. The migrated surface

| Domain | Trait | Members | Assoc. fns | Strict type(s) |
|---|---|---:|---:|---|
| graphics / manager | `DeviceCapabilityExt` | 9 | 0 | `GraphicsDevice` |
| graphics / manager | `DeviceEventExt` | 3 | 0 | `GraphicsDevice` |
| graphics / manager | `DeviceStateExt` | 16 | 0 | `GraphicsDevice` |
| graphics / manager | `GraphicsDeviceManagerExt` | 4 | 0 | `GraphicsDeviceManager` |
| graphics / manager | `OcclusionQueryExt` | 2 | 0 | `OcclusionQuery` |
| effects | `DualTextureEffectExt` | 1 | 0 | `DualTextureEffect` |
| effects | `EffectFactsExt` | 6 | 0 | `Effect` |
| effects | `EffectMaterialExt` | 2 | 0 | `EffectMaterial` |
| effects | `EffectPassExt` | 1 | 0 | `EffectPass` |
| effects | `EffectTechniqueExt` | 2 | 0 | `EffectTechnique` |
| effects | `EnvironmentMapEffectExt` | 1 | 0 | `EnvironmentMapEffect` |
| effects | `SkinnedEffectExt` | 2 | 0 | `SkinnedEffect` |
| graphics | `NativeEnumValue` | 18 | 18 | `Blend`, `BlendFunction`, `CompareFunction`, `CubeMapFace` + 14 more |
| textures | `Texture3DBytes` | 1 | 0 | `Texture3D` |
| textures | `TextureCubeDds` | 1 | 1 | `TextureCube` |
| media | `MediaLibraryExt` | 1 | 0 | `MediaLibrary` |
| media | `MediaQueueExt` | 2 | 0 | `MediaQueue` |
| media | `MediaSourceExt` | 1 | 0 | `MediaSource` |
| media | `PictureExt` | 1 | 0 | `Picture` |
| media | `SongCollectionExt` | 1 | 1 | `SongCollection` |
| media | `SongExt` | 6 | 2 | `Song` |
| media | `VideoExt` | 8 | 2 | `Video` |
| media | `VideoPlayerExt` | 3 | 0 | `VideoPlayer` |
| audio + XACT | `AudioEngineExt` | 3 | 0 | `AudioEngine` |
| audio + XACT | `DynamicSoundEffectInstanceExt` | 4 | 0 | `DynamicSoundEffectInstance` |
| audio + XACT | `MicrophoneExt` | 1 | 1 | `Microphone` |
| audio + XACT | `NativeDisposalState` | 5 | 0 | `AudioEngine`, `SoundBank`, `SoundEffect`, `SoundEffectInstance` + 1 more |
| audio + XACT | `SoundEffectExt` | 1 | 1 | `SoundEffect` |
| storage | `StorageContainerExt` | 2 | 0 | `StorageContainer` |
| input | `KeyFromNativeCode` | 1 | 1 | `Keys` |
| **total** | **30 traits** | **109** | **27** | **47 strict types** |

Runtime behaviour is unchanged for every one of them. The bodies moved
verbatim into `impl Trait for Type` -- same routes, same handles, same SAFETY
reasoning, same ownership and destruction order, same event semantics. Two
private helpers (`GraphicsDevice::flag` and `::toggle`) stayed inherent
because private items are not public surface.

Three members moved *file* as well as impl block, and only to put one type's
CNA-only surface in one trait: `Song::EndedByElapsedTime` from `player.rs`
to `catalog.rs`, and `GraphicsDeviceManager::ObserveDeviceSettings` from its
own impl block into the other three's.

### The call, before and after

```rust,ignore
// before
let song = Song::FromFile(game, "theme", "theme.ogg")?;
let handle = song.HandleText()?;
device.set_string_marker("frame")?;

// after
use cna::extensions::media::SongExt;
use cna::extensions::graphics_device_ext::DeviceStateExt;

let song = Song::FromFile(game, "theme", "theme.ogg")?;   // unchanged
let handle = song.HandleText()?;                          // unchanged
device.set_string_marker("frame")?;                       // unchanged
```

27 of the 109 are associated functions and every one keeps its call shape,
because Rust resolves an associated item on a type through the traits in scope
as well as through inherent impls. The one change beyond the import line:
`from_native_value` on eighteen graphics enums and `Keys::from_key_code`
were `const fn`, and a stable trait method cannot be `const`. The inherent
conversions are still `const` and still what the crate decodes with; what a
consumer reaches through the trait is not usable in a `const` context. Nothing
in this repository or its template used either in one.

No prelude was added. Thirty traits is a lot of names, but they are grouped by
domain and a consumer normally imports one family; an explicit
`use cna::extensions::media::SongExt;` says which extension family a file is
using, and a prelude would have hidden exactly that.

## 7. `TouchPanelTestBackend`

| | |
|---|---|
| before | `cna::Microsoft::Xna::Framework::Input::Touch::TouchPanelTestBackend` |
| after | `cna::extensions::touch::TouchPanelTestBackend` |
| accessibility | public, not test-only |

The routes decided it. Eight of the nine canonical routes behind it are
`CNA_EXTENSION_BACKING` -- a software panel, mouse-touch emulation, placing a
finger, raising an event through the path a device's would take, enqueuing a
gesture, and the frame boundary that turns queued input into a
`TouchCollection` -- and only `cna_touch_panel_reset_for_tests_ext` is
`TOOLING_ONLY`. It is CNA capability with a test-shaped name, not test
infrastructure.

It still takes `TouchLocationState`, `GestureSample` and `Vector2`.
Accepting an XNA type is not being one.

`extensions::touch` is a module of its own because `extensions::input` is
CNA's raw joystick layer and this is `input_touch.h`.

## 8. The extension-surface gate

The strict verifier reaches zero by *removing* CNA's members from the XNA
hierarchy, so on its own it cannot tell a member that moved from one that was
deleted. `tools/extension-surface/verify.py` answers the other half, and the
two questions stay separate: nothing about CNA's extensions was pushed into the
XNA contract verifier.

```text
EXTENSION_SURFACE_MEMBERS      283    # 109 moved here, 174 already traits and ungated
EXTENSION_SURFACE_TRAITS        59
PUBLICLY_NAMEABLE_ITEMS        987
UNNAMEABLE_PUBLIC_TYPES          0    # was 1
TOTAL_DIAGNOSTICS                0
```

Two things it does that the strict verifier does not.

**Reachability is walked, not read.** Every `cna::extensions` type is
re-exported out of a private module, and rustdoc's own `paths` records no
canonical path for one of those -- `DeviceSettingsObserver`, public since the
last milestone, looks unreachable there. The gate walks public modules and
public `use` items from the crate root instead.

**A public signature naming an unreachable crate type is a diagnostic.** That
is how `PresentationMode` shipped.

## 9. Route reachability after the migration

Unchanged, exactly:

```text
CANONICAL_ROUTES              4055
BOUND                         3236
DELIBERATE_NON_BINDING         804
BLOCKED_UPSTREAM                15
UNREVIEWED                       0
ACTIONABLE_LOCAL                 0

BOUND_WITHOUT_SAFE_CALL_SITE    97   # unchanged
  JUSTIFIED                     97
  UNJUSTIFIED                    0
NATIVE_FUNCTIONS_REACHED       638   # unchanged
```

The walk collects identifiers from every file outside `native/` and follows
them with no hop limit, so a call site inside an `impl Trait for Type` body is
found exactly as one inside an inherent impl was. Proved rather than assumed:
deleting `DeviceStateExt::set_string_marker`, the only safe caller of
`cna_graphics_device_set_string_marker_ext`, moves the count to 98 with one
unjustified route and fails the gate. The property is now permanent rather than
a one-off: two tests in `tools/c-api-inventory/tests` build a synthetic crate
whose only callers live in `impl Trait for Type`, and assert both that the
routes are reachable and that deleting the caller kills one.

## 10. ABI scoreboard

```text
RUST_SYS_DECLARATIONS      3251      PROTOTYPE_MISMATCHES        0
LINKED_DECLARATIONS        3251      SYMBOL_TYPE_MISMATCHES      0
SYMBOL_ACQUISITIONS        3250      ARITY_MISMATCHES            0
PROTOTYPE_POSITIONS       11586      LAYOUT_FIELD_SET_MISMATCHES 0
C_RUST_MEASUREMENTS        3174      MISSING_DECLARATIONS        0
LAYOUT_FIELD_SETS           187      UNAUDITED_DECLARATIONS      0
CALLBACKS_PROBED             39      ABI_FINDINGS                0
CONSTANTS_PROBED            902      HEADER/LIBRARY EXPORTS   4055 / 4055
```

## 11. Behaviour and ownership

Nothing moved but placement. Native handle ownership, destruction order,
callback lifetimes, thread affinity, XNA event behaviour, content caching,
graphics state and media/audio semantics are untouched; `RUST-EXT-017`'s XACT
`Disposing` path, `RUST-EXT-018`'s per-buffer subscriptions and the
`DeviceSubscription` unsubscribe-before-free order all keep their bodies
verbatim. `RUST-UPSTREAM-023`'s serialised `GraphicsDevice::new` was not
touched and was not re-run: no CNA graphics or platform code changed.

278 tests pass with 0 failures under `--all-features` against both the
HEADLESS and the OPENGLES3 artifact.

## 12. Defects found

**CNA-Rust.** `PresentationMode` was `pub` inside the private
`game::device_manager` module and re-exported nowhere, while
`GraphicsDeviceManager::PreferredPresentationMode` answered with one: a
consumer could call the method and had no way to name what came back. The same
defect the previous milestone found and fixed for `DeviceSettingsObserver` and
`ObservedDeviceSettings`, and missed for this third type. Fixed, and the class
of defect is now measured across the whole public API.

**Verifier / tooling.** The strict verifier's `INTERNAL_TYPE_LEAK` looks for
`cna_sys` and `CNA_` identities, so it cannot see the above -- absence of a
public path is not an identity it recognises. Rather than widen it and blur the
XNA contract question, the check went into the new extension-surface gate.

**Stale documentation.** `SkinnedEffect::VertexColorEnabled` carried a comment
saying XNA declares it and the strict projection had missed it. The pinned
`Microsoft.Xna.Framework.Graphics.dll` gives it to `BasicEffect` and
`DualTextureEffect` and not to `SkinnedEffect`; `MISSING_MEMBER = 0` proves
nothing was missed. Corrected. A second stale comment sat on the
`SoundEffect` block that held `NativeIsDisposed` and described
`GetSampleDuration`/`GetSampleSizeInBytes` instead; it moved to those two.

**CNA upstream.** None found. **Sharp Runtime.** None found; not exercised
beyond being the runtime the artifacts link.

## 13. Qualification

| Gate | Result |
|---|---|
| `cargo check --workspace --all-targets` | PASS, 151 lib warnings -- the pre-migration count exactly |
| `cargo test --workspace --all-features` (HEADLESS) | PASS: 278 passed, 0 failed, 19 ignored |
| `cargo test --workspace --all-features` (OPENGLES3) | PASS: 278 passed, 0 failed, 19 ignored |
| `cargo doc --workspace --no-deps` | PASS, no warnings |
| strict verifier, selected profile | PASS: 0 diagnostics, 0 allowlist |
| strict verifier, complete runtime profile | PASS: 0 diagnostics, 0 allowlist |
| strict verifier on the `--all-features` rustdoc | PASS: 0 diagnostics |
| leak verifier | PASS: 0 / 0 / 0 |
| extension-surface gate | PASS: 283 members, 59 traits, 0 diagnostics |
| C API inventory + census gate | PASS: 4,055 canonical, 0 unmapped, 0 unreviewed, 0 actionable |
| bound-without-safe-site gate | PASS: 97 unreachable, 97 justified, 0 unjustified |
| native ABI verifier | PASS: 0 findings, 0 unaudited |
| runtime capability provenance | PASS: 35 rows, artifact and ABI confirmed |
| API-compat / ABI / reachability / extension-surface tool tests | PASS: 28 / 33 / 16 / 8 |
| packaged-source consumer | PASS: 7 + 204 files, 0 path leaks; extension calls compile with the traits and are refused without them |
| direct-link consumer | PASS: links, no loader calls, runs a real route |
| MSRV source audit | PASS: 0 findings |
| `git diff --check` | clean, both writable repositories |
| MSRV 1.74 runtime | NOT_RUN -- no 1.74 toolchain on this host |
| `rustfmt`, `clippy` | NOT_AVAILABLE on this source-tarball toolchain |
| sanitizers | NOT_RUN -- no instrumented artifact built |
| wasm | NOT_RUN -- no wasm target installed |

## 14. Template and consumers

The checked-in template needed **no change**, which is the separation working:
it is deliberately pure XNA, and it referenced none of the 109. Its
`--extensions-smoke` path uses `extensions::runtime`, `::pbr` and
`::content`, none of which moved.

| Canary | HEADLESS | OPENGLES3 (Xvfb) |
|---|---|---|
| template build | PASS | PASS |
| template `--smoke-test` (60 frames) | PASS | PASS |
| template `--stability-test` (600 frames) | PASS | PASS |
| template `--extensions-smoke` | PASS | PASS |
| generated standalone build | PASS | PASS |
| generated standalone 60 / 600 / `--extensions-smoke` | PASS | PASS |

The generated standalone was re-vendored from the migrated binding, into the
shared `build-consumer/` directory with its existing target tree preserved.

Test-side imports changed in seven files -- `extensions_graphics_device`,
`extensions_device_surface`, `extensions_effects`, `extensions_media_ext`,
`extensions_audio_ext`, `extensions_content_adoption` and
`extensions_content_manifest` -- and nowhere else. The whole test-side diff is
import lines; not one call was rewritten.

## 15. Git

```text
cna-rust           develop, tree clean
                   14 local commits, ahead 14 / behind 0 of origin/develop
                   NOTHING PUSHED
cna-rust-template  develop @ 416642b, tree clean, untouched
cnanext            next    @ 7712534d3, 0 tracked modifications
sharp-runtimenext  next    @ 9cc96cd57, 0 tracked modifications

git diff --check    clean, both writable repositories
```

The fourteen commits, oldest first:

```text
06903c4  the graphics device's own surface, behind four traits     110 -> 80
db1f103  the effect family's CNA-only surface, seven traits         80 -> 65
4142546  the media family's CNA-only surface, eight traits          65 -> 42
fd897f4  the audio, XACT and storage CNA-only surface               42 -> 26
1e7010d  the graphics device manager's four CNA-only members        26 -> 22
29c7cba  the last twenty-one members -- UNEXPECTED_MEMBER is zero   22 ->  1
1fb5135  TouchPanelTestBackend leaves the strict Touch namespace     1 ->  0
ce92e64  a gate for the surface the strict verifier cannot see
bffa599  prove the extension call shapes from outside the crate
79c919e  close RUST-SURFACE-001, with the numbers the verifier reports
51840a5  the seventeen-section record for the surface milestone
602dfe9  a trait-impl caller is a safe call site, permanently
5311763  what the naming rule now covers, and the split
(tip)    the ending git state, the commits, and where the artifacts are
```

### Build artifacts

Everything built under the repository's own shared directories, never in the
session scratchpad or `/tmp`. `build-consumer/` is 3.7 GB and holds the
packaged-source stage, the direct-link consumer and its 725 MB target tree, the
two new 20 KB extension-import projects, and the re-vendored generated
standalone. The standalone was re-vendored **into its existing directory with
its target tree preserved**, so it rebuilt incrementally rather than from
scratch. `build-consumer/generated/my-cna-game` (597 MB) is an older run's
project that nothing here used; it is the one candidate for deletion if space
is wanted.

## 16. Remaining work

```text
ACTIONABLE_LOCAL                     = 0
UNREVIEWED                           = 0
UNJUSTIFIED_BOUND_WITHOUT_SAFE_SITE  = 0
STRICT_SELECTED_TOTAL_DIAGNOSTICS    = 0
STRICT_COMPLETE_TOTAL_DIAGNOSTICS    = 0
UNEXPECTED_MEMBER                    = 0
UNEXPECTED_TYPE                      = 0
ALLOWLIST                            = 0
PRODUCT_DECISION_REQUIRED            = 0
TODO / FIXME / unimplemented!        = 0
```

| Remaining | Class | Needs |
|---|---|---|
| `RUST-PLATFORM-003` wasm | BLOCKED_PLATFORM | a wasm toolchain, not installed and no `rustup` to add one |
| `RUST-PLATFORM-002` macOS | BLOCKED_PLATFORM | a macOS runtime host |
| `RUST-BEHAVIOR-012` | BLOCKED_HARDWARE | a second machine for a live network peer |
| `RUST-BEHAVIOR-008` | BLOCKED_HARDWARE | a real audio capture/playback backend |
| `RUST-BEHAVIOR-009` | BLOCKED_ASSET | a legally redistributable video fixture |
| 10 `RUST-UPSTREAM-*` findings | BLOCKED_UPSTREAM | CNA fixes; each has a reproducer that runs without this repository |
| `RUST-BEHAVIOR-004`, `-006`, `-010` | BLOCKED_UPSTREAM | CNA routes that refuse or are absent |
| `RUST-PACKAGE-003` | BLOCKED_UPSTREAM | publishing `cna-rust-sys` before `cna-rust` |
| `RUST-XNA-004` Content Pipeline | DELIBERATE_NON_BINDING | a product-boundary decision: 125 design-time types |
| 804 canonical routes | DELIBERATE_NON_BINDING | nothing; each carries a reason |

## 17. Next frontier

Only external unblockers remain: a wasm toolchain, a macOS host, a second
machine, real audio hardware, a redistributable video fixture, and upstream CNA
acting on the ten findings. Publishing `cna-rust-sys` before `cna-rust`
is the one ordering question, and it is a release decision rather than
engineering work.
