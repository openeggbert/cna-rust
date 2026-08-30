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

## XNA surface

| ID | Subject | Authority | Target | Tests | Status |
|---|---|---|---|---|---|
| RUST-XNA-001 | Measure the complete retained XNA 4.0 corpus | 17 SHA-256-admitted Microsoft assemblies | explicit per-profile inventory | `tools/api-compat/verify.py --profile` | DONE |
| RUST-XNA-002 | Fix `!0[]` generic-array parsing in the reference extractor | CLR metadata | the Content Pipeline profile measures | pipeline profile run | DONE |
| RUST-XNA-005 | Scope measurement by profile so a wider profile can land incrementally | 17-assembly superset | `UNEXPECTED_TYPE` means no XNA assembly declares it | all four profile runs | DONE |
| RUST-XNA-003 | GamerServices + Avatar + Net value identities: 22 enums, 7 exceptions | `xna40-windows-full` profile | 74 -> 45 missing types, no other diagnostic | full-profile verifier, behaviour corpus | DONE |
| RUST-XNA-004b | `PacketWriter` and `PacketReader` | `xna40-windows-full` profile | exact XNA byte order and disposal | behaviour corpus | DONE |
| RUST-XNA-004c | `AvatarExpression`, `LeaderboardIdentity`, `NetworkSessionProperties` | `xna40-windows-full` profile | exact CLR value shape | behaviour corpus | DONE |
| RUST-XNA-006 | GamerServices + Avatar + Net object model: 40 remaining types | `xna40-windows-full` profile | strict zero on the complete runtime profile | full-profile verifier, native stress | READY |
| RUST-XNA-004 | Content Pipeline: 125 missing design-time types | `xna40-windows-pipeline` profile | decide whether a design-time profile belongs in this crate | pipeline verifier | READY |

## Behaviour

| ID | Subject | Prior state | Live measurement | Status |
|---|---|---|---|---|
| RUST-BEHAVIOR-001 | Unmodified canonical C API build | blocked at `CnaCApiCoreExt.cpp:250` (`49 == 50`) | builds; ABI 0.20.0 moved the renderer maximum to 49 | DONE |
| RUST-BEHAVIOR-002 | `SoundEffectInstance.Apply3D` with several listeners | `UPSTREAM_CNA_BLOCKED`: CNA refused every count but one | measured: any positive count is accepted; nearest listener decides the mix | DONE |
| RUST-BEHAVIOR-003 | `VideoPlayer` frame identity and generation | `UPSTREAM_CNA_BLOCKED`: no stable identity | measured: `cna_video_player_get_frame_ext` bound; frame wrapped as a borrowed `Texture2D` | DONE |
| RUST-BEHAVIOR-004 | Repeated `Game` frame callback-context rebinding | `UPSTREAM_CNA_BLOCKED` | re-measured: `CNA_GameCallbacks` is still copied at create with no context rebind route; XNA's own `RunOneFrame` no-ops without a host | BLOCKED_UPSTREAM |
| RUST-BEHAVIOR-005 | `AudioEngine` renderer id and look-ahead | ignored by CNA | re-measured: still accepted and ignored, now stated upstream as a one-backend fact | DONE |
| RUST-BEHAVIOR-006 | `GraphicsDeviceManager.RankDevices` | no candidate-ranking route | re-measured: `runtime_graphics_manager.h` still has no ranking route | BLOCKED_UPSTREAM |
| RUST-BEHAVIOR-007 | Media catalogs, picture tokens and `SavePicture` | `PLATFORM_PENDING` | re-measure | READY |
| RUST-BEHAVIOR-008 | Visualization spectrum on the dummy backend | `BACKEND_BLOCKED` | unchanged until a real audio backend is qualified | BLOCKED_HARDWARE |
| RUST-BEHAVIOR-009 | Authored video decode | `BACKEND_BLOCKED`, `ASSET_PENDING` | needs a legal deterministic fixture | BLOCKED_ASSET |

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
| RUST-EXT-013 | `.cnb` Model, sprite font, sound effect and the loader registry | `cnb.h` | READY |
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
