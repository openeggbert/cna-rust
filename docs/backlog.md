# CNA-Rust backlog

Durable task list. Each entry records the exact identity, its authority, the
current state, the target, any blocker, the qualification, and status.

Status values: `READY`, `IN_PROGRESS`, `DONE`, `BLOCKED_UPSTREAM`,
`BLOCKED_PLATFORM`, `BLOCKED_HARDWARE`, `BLOCKED_ASSET`.

## ABI

| ID | Subject | Authority | Target | Tests | Status |
|---|---|---|---|---|---|
| RUST-ABI-001 | Migrate the reviewed slice from ABI 0.7 to live 0.20 | canonical headers | zero prototype/layout/constant findings | `tools/native-abi/verify.py` | DONE |
| RUST-ABI-002 | Replace the constant version gate with CNA's versioning contract | `docs/c-api/ABI_VERSIONING.md` | major/minor/patch policy, `0.x` exact minor | `native::abi` unit tests | DONE |
| RUST-ABI-003 | Audit `cna_vertex_declaration_create`, declared but unmeasured | canonical headers | present in the ABI manifest | ABI verifier | DONE |
| RUST-ABI-004 | Classify all 4,051 canonical routes | canonical headers | zero `UNMAPPED_REQUIRES_REVIEW` | `tools/c-api-inventory/inventory.py` | DONE |
| RUST-ABI-005 | Adopt `cna_error_get_last_info` for structured error identity | `core.h`, `docs/c-api/ERRORS.md` | `CnaError::Native` carries `ErrorCategory` | native stress asserts the reported category | DONE |
| RUST-ABI-006 | Mutation tests for the ABI gate and manifest | this repository | a wrong version, arity, width, signedness, pointer depth, constness, semantic handle or descriptor fails | `tools/native-abi/tests` | DONE |
| RUST-ABI-007 | Re-measure every refusal the safe layer carried from ABI 0.7 | canonical headers | a refusal survives only if 0.20 still cannot do it | native stress, `docs/graphics-evidence.md` | DONE |
| RUST-ABI-008 | Independent `GraphicsDevice` construction over `cna_graphics_device_create` | `graphics_device.h`, XNA `GraphicsDevice` IL | the public constructor works and the owned device is a distinct ownership case, not a flag | `native_stress` `independent-graphics-device`, 9 planted mutations | DONE |
| RUST-ABI-009 | Admit live ABI `0.21` and re-classify the routes it added | canonical headers, `docs/c-api/ABI_VERSIONING.md` | reviewed minor is 21; the three added routes carry a true classification | ABI verifier, route inventory | DONE |
| RUST-ABI-010 | Stop the `*type_name*` rule from absorbing genuine asset data | `tools/c-api-inventory/classification.json` | every route it captures really is CLR reflection | route inventory | DONE |

## XNA surface

| ID | Subject | Authority | Target | Tests | Status |
|---|---|---|---|---|---|
| RUST-XNA-001 | Measure the complete retained XNA 4.0 corpus | 17 SHA-256-admitted Microsoft assemblies | explicit per-profile inventory | `tools/api-compat/verify.py --profile` | DONE |
| RUST-XNA-002 | Fix `!0[]` generic-array parsing in the reference extractor | CLR metadata | the Content Pipeline profile measures | pipeline profile run | DONE |
| RUST-XNA-005 | Scope measurement by profile so a wider profile can land incrementally | 17-assembly superset | `UNEXPECTED_TYPE` means no XNA assembly declares it | all four profile runs | DONE |
| RUST-XNA-003 | GamerServices + Avatar + Net value identities: 22 enums, 7 exceptions | `xna40-windows-full` profile | 74 -> 45 missing types, no other diagnostic | full-profile verifier, behaviour corpus | DONE |
| RUST-XNA-004b | `PacketWriter` and `PacketReader` | `xna40-windows-full` profile | exact XNA byte order and disposal | behaviour corpus | DONE |
| RUST-XNA-004c | `AvatarExpression`, `LeaderboardIdentity`, `NetworkSessionProperties` | `xna40-windows-full` profile | exact CLR value shape | behaviour corpus | DONE |
| RUST-XNA-006a | GamerServices object model: gamers, collections, profiles, achievements, leaderboards, Guide, dispatcher | `xna40-windows-full` profile | 40 -> 18 missing types, no other diagnostic | full-profile verifier, `gamer_services_native` | DONE |
| RUST-XNA-006b | Avatar object model: `AvatarDescription`, `AvatarAnimation`, `IAvatarAnimation`, `AvatarRenderer` | `xna40-windows-full` profile | 18 -> 14 missing types, no other diagnostic | full-profile verifier, `gamer_services_native` | DONE |
| RUST-XNA-006c | Net object model: `NetworkSession` and its 13 companions | `xna40-windows-full` profile | 14 -> 0 missing types, strict zero on the complete runtime profile | full-profile verifier, `net_native` | DONE |
| RUST-XNA-004 | Content Pipeline: 125 missing design-time types | `xna40-windows-pipeline` profile | decide whether a design-time profile belongs in this crate | pipeline verifier | READY |

## Behaviour

| ID | Subject | Prior state | Live measurement | Status |
|---|---|---|---|---|
| RUST-BEHAVIOR-001 | Unmodified canonical C API build | blocked at `CnaCApiCoreExt.cpp:250` (`49 == 50`) | builds; ABI 0.20.0 moved the renderer maximum to 49 | DONE |
| RUST-BEHAVIOR-002 | `SoundEffectInstance.Apply3D` with several listeners | `UPSTREAM_CNA_BLOCKED`: CNA refused every count but one | measured: any positive count is accepted; nearest listener decides the mix | DONE |
| RUST-BEHAVIOR-003 | `VideoPlayer` frame identity and generation | `UPSTREAM_CNA_BLOCKED`: no stable identity | measured: `cna_video_player_get_frame_ext` bound; frame wrapped as a borrowed `Texture2D` | DONE |
| RUST-BEHAVIOR-004 | Repeated `Game` frame callback-context rebinding | `UPSTREAM_CNA_BLOCKED` | re-measured on cnanext `17b5a90a`: `CNA_GameCallbacks` is still copied at create and `runtime.h` has no context-rebind route; XNA's own `RunOneFrame` no-ops without a host | BLOCKED_UPSTREAM |
| RUST-BEHAVIOR-005 | `AudioEngine` renderer id and look-ahead | ignored by CNA | re-measured: still accepted and ignored, now stated upstream as a one-backend fact | DONE |
| RUST-BEHAVIOR-006 | `GraphicsDeviceManager.RankDevices` | no candidate-ranking route | re-measured on cnanext `17b5a90a`: `runtime_graphics_manager.h` still has no ranking route | BLOCKED_UPSTREAM |
| RUST-BEHAVIOR-007 | Media catalogs, picture tokens and `SavePicture` | `PLATFORM_PENDING` | re-measure | READY |
| RUST-BEHAVIOR-008 | Visualization spectrum on the dummy backend | `BACKEND_BLOCKED` | unchanged until a real audio backend is qualified | BLOCKED_HARDWARE |
| RUST-BEHAVIOR-009 | Authored video decode | `BACKEND_BLOCKED`, `ASSET_PENDING` | needs a legal deterministic fixture | BLOCKED_ASSET |
| RUST-BEHAVIOR-010 | `NetworkGamer`'s inherited `Gamer` members | not reachable | measured: `cna_gamer_*` refuses a network gamer handle, so `Gamertag`, `DisplayName`, `Tag`, `GetProfile` and `ToString` report CNA's refusal | BLOCKED_UPSTREAM |
| RUST-BEHAVIOR-011 | `LocalNetworkGamer.SignedInGamer` | not reachable | measured: `cna_local_network_gamer_get_signed_in_gamer` answers `NOT_SUPPORTED` -- "Signed-in gamers have no C representation yet" | BLOCKED_UPSTREAM |
| RUST-BEHAVIOR-012 | A second machine in a session | `NO_LIVE_PEER` | one process can admit a peer only through CNA's own injection routes; a real remote participant needs two hosts | BLOCKED_PLATFORM |

## Extensions

| ID | Subject | Canonical authority | Status |
|---|---|---|---|
| RUST-EXT-001 | Renderer selection, availability and fallback chain | `core_ext.h` | DONE |
| RUST-EXT-002 | Logging sink and minimum level | `core_ext.h` | DONE |
| RUST-EXT-011 | Platform and desktop-OS identity | `core_ext.h` | DONE |
| RUST-EXT-012 | Backend category and maturity classification | `core_ext.h` | DONE |
| RUST-EXT-003 | Renderer capability report, limits and feature support | `graphics.h`, `graphics_device.h` | DONE |
| RUST-EXT-004 | Post-processing effects: CRT, depth, ASCII | `graphics_ext.h` | DONE |
| RUST-EXT-005 | PBR material and render-pipeline settings | `graphics_ext.h` | READY |
| RUST-EXT-006 | `.cnb` container: open, metadata, Texture2D | `cnb.h` | DONE |
| RUST-EXT-013a | `.cnb` Model: graph, geometry, materials | `cnb.h` | DONE |
| RUST-EXT-013b | `.cnb` loader registry, writer and native content manager | `cnb.h`, `content.h` | DONE |
| RUST-EXT-013c | `.cnb` SpriteFont and SoundEffect decode | `cnb.h` | READY |
| RUST-EXT-007 | Device layer: power, locale, clipboard, display info | `devices.h` | DONE |
| RUST-EXT-008 | Raw joystick enumeration, capabilities and capture | `input_joystick.h` | DONE |
| RUST-EXT-014 | Haptics, text input, cursor and device hot-plug | `input_*.h` | READY |
| RUST-EXT-009 | Sensors: accelerometer, compass, gyroscope | `sensors.h` | READY |
| RUST-EXT-010 | CNAEXT engine layer, 857 routes | `engine_layer.h` | READY |

## Platform and packaging

| ID | Subject | Target | Status |
|---|---|---|---|
| RUST-PLATFORM-001 | Windows dynamic loader | source complete; no Windows Rust target on this host, so `COMPILE_NOT_VERIFIED_PLATFORM` | DONE |
| RUST-PLATFORM-002 | macOS loader | already `#[cfg(unix)]`; needs a runtime run | BLOCKED_PLATFORM |
| RUST-PLATFORM-003 | WebAssembly | re-measured: CNA's wasm C ABI exists (4,053 names); Rust has no wasm target here and the binding has no static-linkage mode | BLOCKED_PLATFORM |
| RUST-PLATFORM-004 | Static-linkage mode so `Native` can be filled from `extern "C"` declarations | the prerequisite for any WebAssembly route | READY |
| RUST-PACKAGE-001 | `cargo package` file list, notices, no native binary | LICENSE and NOTICE.md now ship with each crate; an outside consumer builds from the packaged file set alone | DONE |
| RUST-PACKAGE-003 | `cargo package -p cna-rust` needs `cna-rust-sys` published first | publish order recorded | BLOCKED_UPSTREAM |
| RUST-PACKAGE-002 | MSRV 1.74 evidence | `tools/msrv/audit.py` denylist; two real violations found and fixed. Compiling with 1.74 is still `MSRV_RUNTIME_NOT_RUN` | DONE |
| RUST-TEMPLATE-001 | Template against the live artifact | 60/600 frames on ABI 0.20 | DONE |
| RUST-TEMPLATE-002 | Template modern-extension canary | `--extensions-smoke` | DONE |
| RUST-TEMPLATE-003 | Generated standalone project on the live ABI | workspace tests, 60/600 frames, extension canary, no developer path | DONE |

## Qualification, 2026-08-31

Run against the HEADLESS ABI 0.20 artifact with SHA-256
`092b2d80a775f39a6ad872d084bc09492576c82ac33641faeb4a3036c7fc347b`, built out
of tree from `cnanext` at `17b5a90a` and `sharp-runtimenext` at `4a49afb0`.

| Gate | Result |
|---|---|
| `cargo check --workspace --all-targets` | PASS, no warnings |
| `cargo build` (template, generated project) | PASS |
| `cargo test --workspace --all-features` | PASS: 45 suites, 115 assertions, 0 failures |
| `cargo doc --workspace --no-deps` | PASS, no warnings |
| native ABI verifier | PASS: 1,326 functions, 4,574 prototype positions, 1,845 C/Rust measurements, 98 layouts, 19 callbacks, 665 constants, 1,119 symbol acquisitions, 0 findings, 0 unaudited |
| ABI mutation tests | PASS: 27 |
| API-compat mutation tests | PASS: 28 |
| canonical route inventory | PASS: 4,054 canonical, 1,326 bound, 0 unmapped, 0 unused rules |
| runtime capability provenance | PASS: 35 rows, artifact and ABI confirmed |
| selected XNA profile (strict) | PASS: 0 diagnostics |
| complete XNA runtime profile | PASS: 0 diagnostics, 0 missing types |
| Content Pipeline profile | 125 missing types (product-boundary decision, `RUST-XNA-004`) |
| superset discovery profile | 125 missing types, all of them the pipeline |
| leak verifier | PASS: 0 |
| MSRV source audit | PASS |
| packaged-source consumer | PASS: 5 sys files, 148 crate files, 0 workspace path leaks |
| template 60 / 600 frames / `--extensions-smoke` | PASS |
| generated standalone project: build, 60, 600, `--extensions-smoke` | PASS |
| `git diff --check`, both writable repositories | clean |
| MSRV 1.74 runtime | NOT_RUN -- no 1.74 toolchain on this host |
| `rustfmt`, `clippy` | NOT_AVAILABLE |
| sanitizers | NOT_RUN -- no instrumented artifact was built |
