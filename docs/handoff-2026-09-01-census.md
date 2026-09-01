# CNA-Rust handoff — 2026-09-01, the reachability milestone

The milestone this session closed: **every bound route now either has a safe
caller or says, checkably, why it does not.** `BOUND_WITHOUT_SAFE_CALL_SITE`
went from a number that had been wrong four times to 97 measured routes, all 97
justified, and the census gate fails on an unexplained one for the first time.

The three deferred families closed with it. `RUST-EXT-016`, `RUST-EXT-017` and
`RUST-EXT-018` are all `DONE`, and `DEFERRED_TRACKED` is zero.

Everything here was measured on this checkout. Where a previous document's
claim did not survive re-measurement, that is stated rather than quietly fixed.

---

## 1. Start state

```text
cna-rust HEAD at start   f87b0e362e110605a15488900e8b6a6c9bfd6288  (develop)
working tree             clean
ahead of origin/develop  0   -- the previous session's seven commits were pushed
```

The reported starting state was `UNREVIEWED = 0`, `ACTIONABLE_LOCAL = 0`,
`BOUND_WITHOUT_SAFE_CALL_SITE = 303`. Re-measured against the live checkout,
two of the three did not hold:

| Claim | Reality |
|---|---|
| `CANONICAL_ROUTES = 4054`, `UNREVIEWED = 0` | 4,055 and 1: cnanext had added `cna_network_session_replace_session_properties` (CABI-49) after the census ran |
| `BOUND_WITHOUT_SAFE_CALL_SITE = 303` | 138. The detector was still wrong, in a way the "two-hop" fix had not touched. See §6 |
| Backlog rows `RUST-EXT-015d`, `RUST-EXT-015e`, `RUST-CENSUS-003` = `ACTIONABLE_LOCAL` | All three had been finished by the milestone that wrote them; reconciled to `DONE` before any new work started |

## 2. CPU policy

Every compile, test and tool run this session was pinned with
`taskset -c 0-3` and capped at four jobs: `CARGO_BUILD_JOBS=4`,
`CMAKE_BUILD_PARALLEL_LEVEL=4`, `MAKEFLAGS=-j4`, `RUST_TEST_THREADS=4`,
`cargo -j 4`, `cmake --build --parallel 4`. Nothing else on the machine was
inspected, throttled or waited on.

Both CNA artifacts were rebuilt **incrementally** into the build directories
that already existed, with ccache, rather than reconfigured.

## 3. Dependency identity

```text
cnanext HEAD at start and end   7712534d3d22c7e284714e0e87afebba3f3cb472
cnanext modified files          0
cnanext the artifacts were built from
                                7712534d3d22c7e284714e0e87afebba3f3cb472

sharp-runtimenext at start      bd282d101640005454639b372f67e119ffa5642b
sharp-runtimenext at end        9cc96cd57cde394940cc24d58743edf9bf63d3fb  (another agent)
sharp-runtimenext modified      0

ARTIFACT_ENGINE    cnanext/cmake-build-opengles3   CNA_CNAEXT=ON,  OPENGLES3, SDL3 audio
ARTIFACT_HEADLESS  cnanext/cmake-build-headless    CNA_CNAEXT=OFF, HEADLESS,  SDL3 audio
LIBRARY_SHA256     94078be94dc1f1e6c8787c1cd17b08c9430d1e4bb5699947cd2b7aafee40281d  (headless)
LIBRARY_EXPORTS    4055
HEADER_EXPORTS     4055
```

The previous artifacts were built from `35268971c` and no longer matched the
headers: they exported 4,054 routes and not the one CABI-49 added. Both were
rebuilt before anything was measured against them, and the recorded runtime
capability provenance was re-recorded to name the artifact that was actually
used.

**Relevant CNA code that changed since the last qualification:** `net_sessions`,
the ENet backend, `NetworkSession`, one XNB list-reader registration, and
documentation. **Nothing in `modules/graphics`, `modules/renderers` or
`modules/platform`** -- which is what decides the `RUST-UPSTREAM-023`
revalidation question in §11.

## 4. CNA / ABI scoreboard

```text
CNA ABI                        0.21.0
canonical routes               4055
canonical headers              57

cna-sys declarations           3251
symbol acquisitions            3250   (cna_get_abi_version is acquired as a local)
linked declarations            3251
prototype mismatches           0
symbol type mismatches         0
arity mismatches               0
layout field sets checked      187
C/Rust probe measurements      3174
constants probed               902
callback signatures probed     39
unaudited declarations         0
missing declarations           0
ABI findings                   0
```

## 5. Route census — before and after

```text
                          before    after
CANONICAL                   4054     4055
BOUND                       3232     3236
DELIBERATE_NON_BINDING       801      804
BLOCKED_UPSTREAM              15       15
DEFERRED_TRACKED               6        0
UNREVIEWED                     0*       0
ACTIONABLE_LOCAL               0        0
```

\* reported as 0; measured as 1 once the new route arrived.

`BOUND` moved by +7 and -3. The seven: the CABI-49 property write-back, the two
content loads `RUST-EXT-016` needed, and the four XACT disposing subscriptions
`RUST-EXT-017` needed. The three: `cna_keyboard_state_is_key_down`,
`cna_graphics_device_supports_capability` and `cna_vertex_declaration_create`,
which were declared in `cna-sys` and acquired by nothing at all -- not a table
field, not a `symbol!`, nothing that could ever call them.

## 6. RUST-CENSUS-002

### The number was wrong four times

894, then 1,077, then 303, now 138. Every one of the first three came from
matching a C route's *name* against Rust identifiers, and the field that holds a
route's pointer is **not named after the route**:
`cna_audio_category_pause` lives in `AudioApi::category_pause`. The "two-hop"
fix in the previous milestone deepened the search without fixing what it was
searching for.

`tools/c-api-inventory/reachability.py` measures it instead. Three facts make
that exact:

- every route is acquired at exactly one place, `field: symbol!(cna_x, ...)`,
  which ties the C route to the Rust field;
- all 3,250 field names are unique across the crate, so `.category_pause`
  names one route and no other;
- the safe layer is everything outside `native/`.

The walk starts at every file outside `native/`, follows the names they mention
into `native/` functions, follows what *those* mention, and keeps going. No hop
limit. Measured hop distribution: 2,400 routes named by the safe layer
directly, 696 behind one wrapper, 4 behind two. A hop-limited check set at two
would still miss `cna_error_get_last_info`, which every fallible call reaches
through `Native::check` and `Native::last_error_category`.

Fourteen mutation tests, in `tools/c-api-inventory/tests`.

### 138 -> 97

```text
bound without a safe call site      138 -> 97
  justified                                97
  unjustified                                0
  IMPLEMENTED_IN_SAFE_RUST                  59
  OUTSIDE_XNA_SURFACE                       37
  ATOMIC_TABLE_MEMBER                        1

newly reachable (a safe caller added)       38
unbound as never-acquired                    3
```

By family:

| Family | Was | Now | What happened |
|---|---|---|---|
| `net.h` packet reader/writer | 28 | 0 justified | XNA's are `BinaryReader`/`BinaryWriter` over a `MemoryStream`; the projection owns the buffer and writes XNA's byte order, measured against the decompiled assemblies |
| `net.h` session properties | 14 | 0 justified | 5 are XNA mutators that **throw**, 4 are private explicit interface implementations, 5 are an enumerator Rust walks itself |
| `gamer_services.h` collections | 20 | 0 justified | the same shape again for `AchievementCollection`, `GamerCollection<T>` and `PropertyDictionary` |
| versioned struct initialisers | 15 | 0 justified | Rust builds the descriptor as a literal and must set `struct_size`/`struct_version` anyway; the layout is pinned field by field by the ABI verifier |
| `net_gamers.h` gamer facts | 7 | reachable | `NetworkGamerFacts` -- `Id`, `IsHost`, `HasLeftSession`, `RoundtripTime`, `Machine`, the machine itself and the local gamer |
| `net_sessions.h` discovered sessions | 8 | reachable | `DiscoveredSessionInjection` and `DiscoveredSessionExt` |
| `gamer_services.h` platform seams | 12 | reachable | achievements, friends, free-text presence, the dispatcher's async step, the avatar content names and real rendering, the guide renderers |
| `net.h` join error, session counts | 4 | reachable | `LastJoinError`, `LiveSessionCount`, `PendingSessionActionCount`, `clear_packets` |
| the three never-acquired | 3 | unbound | see §5 |
| the rest | 4 | justified | `cna_video_player_get_texture` (atomic table), the CLR type-name pair, `cna_guide_show_achievements_ext` (a declared no-op) |

### The gate

`boundWithoutSafeCallSite` is no longer report-only. The gate fails on an
*unexplained* dead route rather than on the count -- an atomic table and a
read-only projection both leave routes legitimately uncalled. Four planted
defects, one at a time against a passing gate:

| Planted defect | Gate | Reported |
|---|---|---|
| a justification deleted | fails | `boundWithoutSafeCallSiteUnjustified: [cna_packet_writer_write_matrix]` |
| its `rustEvidence` renamed | fails | `justificationsNamingAbsentRustCode: [cna_packet_writer_create: pub struct PacketScribbler]` |
| a justification for a *reachable* route | fails | `staleUnreachableJustifications: [cna_audio_category_pause]` |
| the only safe caller of a route deleted | fails | `boundWithoutSafeCallSiteUnjustified: [cna_net_get_last_join_error]` |

Deleting `AudioCategory::Pause` alone does **not** fail the gate, and should
not: `Resume`, `SetVolume` and `Stop` reach the same wrapper and the wrapper
still reads `category_pause`. A route is dead when nothing reaches the field.

## 7. Safe API added — what a consumer can now do

Not route names: capabilities.

- **Pump gamer services at all.** `GamerServicesComponent` was an ordinary
  `GameComponent` with the right name. A game that added it got correct update
  ordering and a dispatcher nobody ever initialised -- which is what every
  `Begin*` call waits on. It now does XNA's five calls in XNA's order.
- **Change a live session's properties.** `session.SessionProperties[0] = 5` is
  a reference assignment in XNA and was a copy whose writes went nowhere here.
  `extensions::net::ApplySessionProperties` is the second half.
- **Receive a packet larger than 4 KiB.** `ReceiveData(PacketReader, out
  sender)` truncated silently at an unexplained `vec![0_u8; 4096]`. It now
  states a 64 KiB ceiling and reports a packet that exceeds it.
- **Exercise `AvailableNetworkSession` at all.** `Find` on one machine finds
  nothing, so a whole XNA type -- six properties -- had no way to be reached.
  `DiscoveredSessionInjection` builds one; `DiscoveredSessionExt` reads the
  connect address, port and advertised type XNA never published.
- **Learn why a join failed.** `LastJoinError` reports what XNA carries on
  `NetworkSessionJoinException`, measured returning `SessionNotFound` for a
  real refused join.
- **Read an achievement that has a name.** CNA has no catalog, so nine of
  `Achievement`'s ten properties could only be measured against empty strings.
  `AchievementInjection` publishes one.
- **Read a friend that has states.** Same, for `FriendGamer`'s eight.
- **Set the five gamer facts a transport would have measured**, and build the
  `NetworkMachine` and `LocalNetworkGamer` underneath them.
- **Draw an avatar that is not a placeholder**, from the game's own
  `SkinnedModel`, with the five colours and a named clip -- and ask what content
  a body type or an animation preset wants.
- **Draw the Guide screen CNA leaves pending**, which is the other half of
  `BeginShowMessageBox` on a runtime with no console overlay.
- **Load a `SpriteFont` and a `SoundEffect` through CNA's content manager**,
  which is `RUST-EXT-016`.
- **Be told when an XACT object is disposed** (`RUST-EXT-017`) and **when a
  dynamic buffer's content is lost** (`RUST-EXT-018`).
- **Observe candidate device settings** without a raw CNA descriptor crossing
  the safe boundary.

## 8. RUST-EXT-016 — adoption

`cna_content_manager_load_sprite_font` and `cna_content_manager_load_sound_effect`
were deferred for one reason: both hand over a native object already made, and
every constructor on those two types *makes* one.

**`SpriteFont::adopt`** reads the tables back out of the handle --
`cna_sprite_font_get_info` for the layout properties, `_copy_characters` for
the map, `_copy_glyphs` for the bounds, cropping and kerning -- rather than
building a second native font from tables this side does not have.

**Ownership.** The load answers **two** owned handles. The `SpriteFont` owns
the font; the returned `Arc<Texture2D>` owns the atlas. CNA refuses
`cna_texture2d_destroy` while a font still uses the atlas, so the font must go
first, which the type's field order guarantees and the test measures through
the atlas's reference count: 2 while the font lives, 1 after it drops.

**`SoundEffect::adopt`** is the state `FromAsset` already built around a handle
CNA made. **CNA's loader does not cache** -- the header says so and the test
proves it: the first effect is disposed, the second still plays and reports the
same duration.

**ContentManager behaviour.** Both are on `NativeContentManager`, CNA's own
manager, not on the strict XNA `ContentManager`. The strict XNB pipeline is
untouched. `manager.unload()` after both loads releases nothing twice.

### And it found a defect

A real XNA font is the one input a synthetic fixture cannot stand in for,
because the fixtures were written to match what the reader does. MonoGame's
`Default.xnb` stores its 95 characters as `20 21 22 23 ...` -- one byte each,
UTF-8, because `ContentReader` is `new BinaryReader(stream)` and that overload's
encoding is UTF-8 with no BOM.

`ContentReader::ReadChar` read a little-endian **UTF-16** code unit. Against a
real font that is 47 wrong characters and every field after them a byte late;
against this repository's own `.xnb` fixtures, which had the same mistake baked
in, nothing showed at all. Fixed, both fixtures corrected, and reverting it now
fails the SpriteFont XNB case in `native_stress`.

The real font is now loaded, adopted and measured: 95 characters, `' '` to
`'~'`, line spacing 19, no default character, a 128x128 atlas.

## 9. RUST-EXT-017 — XACT `Disposing`

XNA's IL settles the contract, and it is not what CNA's C++ does:

```text
AudioEngine.Dispose(disposing)
  -> NotifyDestroyedEngine(this)
       -> for every Cue, SoundBank and WaveBank of this engine:
            call its PUBLIC Dispose()   -- so its Disposing fires
  -> release the engine handle
  -> if (disposing) raise the engine's own Disposing
```

This projection called the *private* teardown on the engine path: the handle
went away and the handler waited forever. Each state now has a `dispose_raising`
that releases and then raises, with the public object rebuilt from a self-`Weak`
so the sender is what XNA's `this` is.

| Path | Raises | Why |
|---|---|---|
| explicit `Dispose()` | yes, once | `Dispose(true)` |
| a second `Dispose()` | no | XNA's `_isDisposed` guard |
| owner-engine teardown | yes, once per child | `NotifyDestroyedEngine` calls the public `Dispose` |
| `Dispose(false)` | no | XNA raises only when `disposing` |
| `Drop` | no | the finalizer's path |
| game shutdown | no | same |

**Native cross-check.** The four `*_subscribe_disposing_ext` routes are bound
in the same commit, which is why they were deferred rather than bound alone:
binding them without the fix would have made the two events visibly disagree
instead of fixing either. `extensions::audio_ext::NativeDisposalNotice` reports
whether CNA raised its own, and the test asserts both fired on the engine
teardown. The subscription is released *after* the destroy it exists to
observe -- the opposite of the usual rule, and the only order that measures
anything -- with the context box owned by the object for longer than any call.

Three planted defects each fail it: a silent engine teardown, a raising `Drop`,
and a missing native subscription.

## 10. RUST-EXT-018 — `ContentLost`

`DynamicVertexBuffer::AddContentLostHandler` and its index counterpart added
handlers and emitted them nowhere. Each buffer now installs **one** native
subscription, lazily on its first handler, and the trampoline delivers into the
buffer's own handler list in registration order with the buffer as XNA's sender.

**Subscription lifetime.** One per buffer, not per handler: a second handler
joins the same list. Withdrawn on `Dispose` and on `Drop`, before the handle it
names. CNA also stops tracking a destroyed resource, so the withdrawal is the
ordering guarantee rather than the only thing standing between a disposed
buffer and a callback.

**Semantic qualification.**

```text
EVENT_BRIDGE_VERIFIED      yes -- through cna_graphics_device_notify_content_lost_resources_ext
REAL_DEVICE_LOSS_VERIFIED  no  -- GraphicsDevice::Reset succeeds on HEADLESS and raises nothing
```

The test prints the second rather than dressing it up. Measured: two handlers
fire in order, a removed one does not, a second loss reaches them again, a
disposed buffer's do not, and everything a handler captured is released.

Its own mutation run found the first version's defect: the temporary sender
shared the buffer's bridge, and `Drop` withdraws -- so the first delivery
cancelled the second.

## 11. Upstream findings

Re-measured against the artifacts built from cnanext `7712534d3`.

| Finding | Status | Measured |
|---|---|---|
| `RUST-UPSTREAM-020` camera destroy leaves the platform override dangling | reproduces | the child process still faults; the test asserts it |
| `RUST-UPSTREAM-021` content-loaded `Model` teardown faults | reproduces | `SIGSEGV` 139 on both the destroy and the leak stage |
| `RUST-UPSTREAM-022` a content-loaded skin's skeleton is unreachable | reproduces | asserted as measured |
| `RUST-UPSTREAM-023` concurrent `cna_graphics_device_create` | see below | |
| `RUST-UPSTREAM-024` morph stride list stale | reproduces | 3 accepted, 8 refused of the canonical eleven |
| `RUST-UPSTREAM-025` the one owned-handle engine getter | reproduces | asserted as measured |
| `RUST-UPSTREAM-026` `launch_parameters_add` drops a duplicate | reproduces | asserted as measured |
| `RUST-UPSTREAM-027` sample duration/size are not XNA's | reproduces | the Rust answers stay pinned to the reference values |
| `RUST-UPSTREAM-028` **new** — a queued packet's size is unreachable | new | see `docs/upstream-findings.md` |
| `RUST-UPSTREAM-029` **new** — CNA's `GamerServicesComponent` skips the base | new | source-level, from XNA's IL |

### RUST-UPSTREAM-023

**Relevant CNA code did not change.** `git diff 35268971c..7712534d3` touches
`net_sessions`, the ENet backend, `NetworkSession`, one XNB reader registration
and documentation. Nothing under `modules/graphics`, `modules/renderers` or
`modules/platform`, and nothing in the SDL context path. Per the revalidation
rule, no new stochastic campaign was run: the existing evidence stands
unchanged.

```text
6 threads, create + destroy, free-for-all     13 aborts / 70
6 threads, create only, handles leaked         8 / 30
1 thread                                       0 / 30
HEADLESS, 6 threads                            0 / 30
create SERIALISED, destroy free                0 / 120
create AND destroy both serialised             0 / 40
```

Serialising `cna_graphics_device_create` **alone** removes the observed
corruption; destroy serialisation adds no measured benefit. `GraphicsDevice::new`
therefore still holds its process-wide construction mutex, and the regression
test `six_threads_may_build_a_device_at_once` passes with it (6 built, 0
refused, no fault).

The `0/120` figure has a history worth not rewriting: it was first written from
expectation, corrected down to `0/40` because that was what had actually been
measured at the time, and restored to `0/120` only once the supposedly-killed
80-run job was found to have completed cleanly. It is a measurement, and the
correction was a correction.

## 12. Defects found

**In CNA-Rust (this crate):**

1. `ContentReader::ReadChar` read UTF-16 where `BinaryReader`'s reads UTF-8.
   Fixed; the repository's own fixtures had the same mistake and were corrected
   with it.
2. `GamerServicesComponent` did none of XNA's five calls. Fixed.
3. `NetworkSession.SessionProperties` was a copy whose writes went nowhere.
   Fixed, through an extension, because CNA cannot publish XNA's reference.
4. `ReceiveData(PacketReader, out sender)` truncated at 4,096 bytes and
   reported the short read as a success. Fixed.
5. XACT `Disposing` did not fire on engine teardown. Fixed.
6. `ContentLost` handlers fired nowhere. Fixed.
7. The first `ContentLost` bridge cancelled itself after one delivery, because
   the temporary sender shared the buffer's subscription. Caught by its own
   mutation run, fixed.
8. Two internal type leaks: `Keys::from_key_code` and
   `GraphicsDeviceManager::ObserveDeviceSettings`. Fixed.
9. `ObserveDeviceSettings` returned a type that was never exported, so nothing
   outside the crate could hold what it answered. Fixed.

**In the inventory and tooling:**

10. The reachability detector had never worked. Replaced by a call-graph walk
    with fourteen mutation tests.
11. Three routes counted as `BOUND` were declared and acquired by nothing.
    Unbound.

**In CNA upstream:** `RUST-UPSTREAM-028` and `RUST-UPSTREAM-029`, both new and
both written up with the IL or the source that settles them.

**In Sharp Runtime:** none found.

**Stale documentation:**

12. Three backlog rows were `ACTIONABLE_LOCAL` after the milestone that closed
    them. Reconciled.
13. README's `Verified status (2026-08-23)` claimed the strict verifier reports
    zero unexpected types and members. It reports **110**, and has since the
    `RUST-EXT-015d`/`015e`/`015q` milestones added CNA's own members to strict
    XNA types. Nothing had re-run it. Corrected, and opened as
    `RUST-SURFACE-001` -- a product decision, not a bug with one right fix.

## 13. Qualification

| Gate | Result |
|---|---|
| `cargo check --workspace --all-targets` | pass |
| `cargo test --workspace --all-features` | see §13.1 |
| `cargo doc --workspace --no-deps` | pass (rustdoc runs inside the API verifier) |
| C API inventory + census gate | **exit 0**, first time with the reachability gate armed |
| census mutation controls | 4 planted defects, 4 caught |
| reachability mutation tests | 14 |
| native ABI verifier | 0 findings, 3,251 declarations, 187 layout field sets |
| ABI mutation controls | 33 tests |
| strict XNA verifier, selected profile | 110 diagnostics, all `UNEXPECTED_*`; see §12.13 |
| strict XNA verifier, complete profile | 110, the same ones |
| leak verifier | **0** -- internal leaks, raw handles, public unsafe, allowlist |
| MSRV source audit | `MSRV_SOURCE_AUDIT=PASS`, `MSRV_RUNTIME=NOT_RUN` (no 1.74 toolchain) |
| `rustfmt` | `NOT_AVAILABLE` on this source-tarball toolchain |
| `clippy` | `NOT_AVAILABLE` on this source-tarball toolchain |
| sanitizers | `NOT_RUN` |
| wasm | `NOT_RUN`, no target installed |
| `git diff --check`, both repositories | clean |

Every windowed, SDL or GTK process this session started ran on an Xvfb display
(`:92`), enforced by the session policy script rather than per command. One
background run early in the session inherited the real display and opened GTK
windows on the user's session; that is what made it a policy rather than a
habit.

## 14. Template and consumers

| Subject | HEADLESS | OPENGLES3 |
|---|---|---|
| checked-in template, build | pass | pass |
| template, 60 frames | pass | pass |
| template, 600 frames | pass | pass |
| template, extensions canary | pass, engine layer absent | pass, engine layer 2 |
| generated standalone, build | pass | pass |
| generated standalone, 60 frames | pass | pass |
| generated standalone, 600 frames | pass | pass |
| generated standalone, extensions canary | pass | pass |
| packaged-source consumer | `PASS`, 0 workspace path leaks | — |
| direct-link consumer | `PASS`, `LOADER_CALLS=none` | — |

The standalone was regenerated from the current template against the current
`cna-rust`, and carries no developer absolute path and no sibling-repository
dependency. Its existing build directory was preserved rather than rebuilt from
scratch.

## 15. Git

```text
cna-rust
  HEAD at start   f87b0e362e110605a15488900e8b6a6c9bfd6288
  HEAD at end     see `git log -1`
  local commits   8
  ahead           8
  pushed          NO
  working tree    clean

cna-rust-template
  HEAD            416642b9365198e7ee9cd46820e36fab9b2617be  (unchanged)
  working tree    clean

cnanext           modified files 0
sharp-runtimenext modified files 0
```

## 16. Remaining work

Every row is blocked, deliberate, or a decision for a person.

| Item | Class | Why |
|---|---|---|
| `RUST-SURFACE-001` — where CNA-only members live on a strict XNA type | PRODUCT_DECISION_REQUIRED | 109 members; moving them behind extension traits and accepting them give different public APIs |
| `RUST-BEHAVIOR-004` `Game` callback-context rebinding | BLOCKED_UPSTREAM | no context-rebind route; callbacks copied by value at create |
| `RUST-BEHAVIOR-006` `GraphicsDeviceManager.RankDevices` | BLOCKED_UPSTREAM | no candidate-ranking route |
| `RUST-BEHAVIOR-010` `NetworkGamer`'s inherited `Gamer` members | BLOCKED_UPSTREAM | `cna_gamer_*` refuses the handle |
| `RUST-UPSTREAM-020` … `-029` | BLOCKED_UPSTREAM | ten findings, each with a reproducer |
| `RUST-BEHAVIOR-008` visualization spectrum | BLOCKED_HARDWARE | no real audio backend here |
| `RUST-BEHAVIOR-009` authored video decode | BLOCKED_ASSET | needs a legal deterministic fixture |
| `RUST-BEHAVIOR-012` a second machine in a session | BLOCKED_PLATFORM | one process cannot supply a real peer |
| `RUST-PLATFORM-002` macOS loader | BLOCKED_PLATFORM | needs a runtime run on macOS |
| `RUST-PLATFORM-003` WebAssembly | BLOCKED_PLATFORM | no wasm std, no `rustup` |
| `RUST-PACKAGE-003` `cargo package -p cna-rust` | BLOCKED_UPSTREAM | needs `cna-rust-sys` published first |
| the 801+3 deliberate non-bindings | DELIBERATE_NON_BINDING | each with a reason in `classification.json` |
| the 97 justified unreachable routes | DELIBERATE_NON_BINDING | each with a reason and a checked outcome |

```text
ACTIONABLE_LOCAL                          0
UNREVIEWED                                0
UNJUSTIFIED_BOUND_WITHOUT_SAFE_CALL_SITE  0
```

## 17. Next frontier

Only things this machine and this session cannot supply:

1. **A person's answer to `RUST-SURFACE-001`.** Everything else in the census
   is decided; this is the one open question about the projection's shape.
2. **A wasm toolchain** for `RUST-PLATFORM-003`. The direct-link architecture
   is done and CNA's wasm C ABI exists; only the target is missing.
3. **A second machine** for `RUST-BEHAVIOR-012`. Injection can supply a peer's
   *effects* and now supplies its facts too, but not a peer.
4. **A macOS host** for `RUST-PLATFORM-002`.
5. **A real audio backend** for `RUST-BEHAVIOR-008`, and a legally
   redistributable video fixture for `RUST-BEHAVIOR-009`.
6. **Upstream fixes** for the ten `RUST-UPSTREAM-*` findings, each of which has
   a reproducer that runs without this repository.
