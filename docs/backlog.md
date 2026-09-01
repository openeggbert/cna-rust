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
| RUST-ABI-008 | Independent `GraphicsDevice` construction over `cna_graphics_device_create` | `graphics_device.h`, XNA `GraphicsDevice` IL | the public constructor works and the owned device is a distinct ownership case, not a flag. Re-validated on cnanext `0a6158e4f`: it succeeds on `HEADLESS` and is refused on every GL-family renderer with `EasyGLRenderer::CreateContext failed: surface has no platform window id`, which is `NOT_SUPPORTED_BY_RENDERER` rather than a binding fault -- the guarded tests skip on exactly that message and fail on any other | `native_stress` `independent-graphics-device`, 9 planted mutations | DONE |
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
| RUST-XNA-004 | Content Pipeline: 125 missing design-time types | `xna40-windows-pipeline` profile | **decided**: out of scope. CNA's native `.cnj`/`.cnb` tooling replaces it, 17 of the 125 cannot be projected faithfully at all, and the useful overlap is already bound by RUST-EXT-013. See [docs/content-pipeline-decision.md](content-pipeline-decision.md) | pipeline verifier, unchanged at 125 by design | DONE |

## Behaviour

| ID | Subject | Prior state | Live measurement | Status |
|---|---|---|---|---|
| RUST-BEHAVIOR-001 | Unmodified canonical C API build | blocked at `CnaCApiCoreExt.cpp:250` (`49 == 50`) | builds; ABI 0.20.0 moved the renderer maximum to 49 | DONE |
| RUST-BEHAVIOR-002 | `SoundEffectInstance.Apply3D` with several listeners | `UPSTREAM_CNA_BLOCKED`: CNA refused every count but one | measured: any positive count is accepted; nearest listener decides the mix | DONE |
| RUST-BEHAVIOR-003 | `VideoPlayer` frame identity and generation | `UPSTREAM_CNA_BLOCKED`: no stable identity | measured: `cna_video_player_get_frame_ext` bound; frame wrapped as a borrowed `Texture2D` | DONE |
| RUST-BEHAVIOR-004 | Repeated `Game` frame callback-context rebinding | `UPSTREAM_CNA_BLOCKED` | re-measured on cnanext `0a6158e4f` (ABI 0.21.0): `runtime.h` still has no context-rebind route, and `CnaCApiRuntime.cpp` still holds `CNA_GameCallbacks callbacks_` by value, copied at create | BLOCKED_UPSTREAM |
| RUST-BEHAVIOR-005 | `AudioEngine` renderer id and look-ahead | ignored by CNA | re-measured: still accepted and ignored, now stated upstream as a one-backend fact | DONE |
| RUST-BEHAVIOR-006 | `GraphicsDeviceManager.RankDevices` | no candidate-ranking route | re-measured on cnanext `0a6158e4f` (ABI 0.21.0): `runtime_graphics_manager.h` has candidate and preference routes and still no ranking route | BLOCKED_UPSTREAM |
| RUST-BEHAVIOR-007 | Media catalogs, picture tokens and `SavePicture` | was `PLATFORM_PENDING` wholesale | re-measured on cnanext `599d14e5`: 26 real pictures, a real root album, tokens answer absence, `SavePicture` preserves its name and joins the live saved-pictures album. Only cross-instance persistence remains provider-dependent | DONE |
| RUST-BEHAVIOR-008 | Visualization spectrum on the dummy backend | `BACKEND_BLOCKED` | unchanged until a real audio backend is qualified | BLOCKED_HARDWARE |
| RUST-BEHAVIOR-009 | Authored video decode | `BACKEND_BLOCKED`, `ASSET_PENDING` | needs a legal deterministic fixture | BLOCKED_ASSET |
| RUST-BEHAVIOR-010 | `NetworkGamer`'s inherited `Gamer` members | not reachable | re-measured on cnanext `0a6158e4f`: `cna_gamer_*` still answers "The handle does not name a gamer this call can use", and the test asserts that exact message | BLOCKED_UPSTREAM |
| RUST-BEHAVIOR-011 | `LocalNetworkGamer.SignedInGamer` | was `NOT_SUPPORTED`: "Signed-in gamers have no C representation yet" | **fixed upstream**: cnanext `599d14e5` answers the real signed-in gamer, and the test now asserts the published gamertag rather than tolerating either outcome | DONE |
| RUST-BEHAVIOR-012 | A second machine in a session | `NO_LIVE_PEER` | one process can admit a peer only through CNA's own injection routes; a real remote participant needs two hosts | BLOCKED_PLATFORM |
| RUST-UPSTREAM-020 | The camera test backend leaves CNA's platform override dangling | not reachable from Rust before this slice | `cna_camera_create_with_test_backend_ext` hands the *global* override a pointer into the camera resource and `cna_camera_destroy` frees it without clearing the override. `SIGSEGV`, 139, measured from Rust in a child process. Blocks the 15 `cna_camera_*` routes as a family. Written up in [docs/upstream-findings.md](upstream-findings.md); reproducer `crates/cna/tests/upstream_camera_destroy.rs` | BLOCKED_UPSTREAM |
| RUST-UPSTREAM-021 | Destroying a content-loaded `Model` faults | not reachable before `cna_content_manager_load_model` was bound | `~MeshResource` moves an empty `detachedValue` over a loaded part's `value` and `~PartResource` dereferences it two lines later, without the null check its own next line applies. `SIGSEGV` at null plus the offset of `tag_`. The hand-built control destroys cleanly; leaking the handle moves the fault to process exit rather than avoiding it. Reproducers in `tools/reproducers/` and `crates/cna/tests/upstream_model_destroy.rs` | BLOCKED_UPSTREAM |
| RUST-UPSTREAM-022 | A content-loaded skin's skeleton is unreachable | not reachable before this slice | `cna_model_create_skin_skeleton_handle_ext` refuses a skin the content pipeline imported -- "not created through the C API" -- which is not the refusal its header documents. No fault; a capability gap. Asserted as measured in `extensions_native_model.rs`, which fails if it is ever fixed | BLOCKED_UPSTREAM |
| RUST-UPSTREAM-023 | Concurrent `cna_graphics_device_create` corrupts the heap | the binding serialises construction, so nothing is blocked | Six threads in the create call at once die with `SIGSEGV` or glibc `double free or corruption` on the GL renderers, ~1 run in 5. The faulting stack is `GraphicsDevice::createRenderer` -> `EasyGLRenderer` -> `Sdl3GlContext::CreateContext` -> `SDL_GL_CreateContext`; SDL's video subsystem is not safe to enter from several threads. Serialising the create call alone removes it (0/80 across the two serialised variants); destroy is not implicated; HEADLESS never faults. The ABI already has `CNA_RESULT_THREAD` for a disallowed thread and this route neither documents an affinity nor returns it. Reproducer `tools/reproducers/ext015h_concurrent_device_create.c`, regression test `crates/cna/tests/upstream_concurrent_device_create.rs` | BLOCKED_UPSTREAM |
| RUST-UPSTREAM-024 | The morph-target stride list is stale and excludes every tangent-carrying layout | Morph targets on any PBR glTF mesh, through the C API | `ValidateMorphShape` restates a literal `{32, 52, 56}` where CNA's canonical `InferredLayoutForStride` lists eleven strides. The three it accepts are exactly the layouts with no tangent; 48 (unskinned) and 68 (skinned), which GLTF-215 made the ordinary PBR strides, are refused. The blender in `MorphTargetEXT.cpp` had the same literal and was fixed (GLTF-278) to query the table; the C API's validator was not. Measured 3 accepted / 8 refused of the canonical eleven in `crates/cna/tests/upstream_morph_stride.rs` | BLOCKED_UPSTREAM |
| RUST-UPSTREAM-025 | `cna_area_light_brdf_table_get_texture` is the only engine-layer getter that publishes an owned handle | nothing; the binding models it as it is | The header says the handle borrows and the aliasing `shared_ptr` behaves exactly as promised (measured: the table survives a dropped handle, and a handle outlives the table). The defect is the contract, not the implementation: it publishes through `CreateOwnedTexture2D`, which calls `AddOwnedGraphicsResourceFor` and so gates `cna_game_destroy`, where all ten analogous `_get_` routes publish borrowed and cost nothing. Measured in `crates/cna/tests/extensions_engine.rs` | BLOCKED_UPSTREAM |
| RUST-UPSTREAM-026 | `cna_game_launch_parameters_add` silently drops a duplicate key | nothing; the binding reports the measured behaviour | The header says "adds **or replaces**"; the implementation calls `emplace`, which keeps the value already there; XNA's `Dictionary.Add` throws. Three contracts for one operation. The second add returns `CNA_RESULT_SUCCESS` and does nothing, so a C caller can neither overwrite a parameter nor learn that their write was dropped. The deviation is annotated in `LaunchParameters.cpp` and its reasoning covers `Parse`, which guards with `ContainsKey`, not this unguarded entry point. Asserted as measured in `crates/cna/tests/extensions_game_runtime.rs` | BLOCKED_UPSTREAM |
| RUST-UPSTREAM-027 | The sample-duration and sample-size helpers are not XNA's | nothing; the binding keeps its own XNA-faithful arithmetic | `GetSampleDuration` truncates where XNA's `TimeSpan.FromMilliseconds` rounds -- up to a millisecond short, and **zero** for a 100-byte stereo buffer at 44.1 kHz that XNA says lasts one. `GetSampleSizeInBytes` drops XNA's `num + num % Channels` frame alignment and does the rate division in double rather than binary32, so one mono second at 44.1 kHz is 88,200 bytes where XNA says 88,198. 19 of 90 duration cases and 15 of 50 size cases disagree. Found by comparing CNA against the Rust projection, with the decompiled XNA assemblies as arbiter. Reproducer `tools/reproducers/ext015q_sample_math.c`; the Rust answers are pinned to the reference values in `crates/cna/tests/extensions_audio_ext.rs` | BLOCKED_UPSTREAM |

## Extensions

| ID | Subject | Canonical authority | Status |
|---|---|---|---|
| RUST-EXT-001 | Renderer selection, availability and fallback chain | `core_ext.h` | DONE |
| RUST-EXT-002 | Logging sink and minimum level | `core_ext.h` | DONE |
| RUST-EXT-011 | Platform and desktop-OS identity | `core_ext.h` | DONE |
| RUST-EXT-012 | Backend category and maturity classification | `core_ext.h` | DONE |
| RUST-EXT-003 | Renderer capability report, limits and feature support | `graphics.h`, `graphics_device.h` | DONE |
| RUST-EXT-004 | Post-processing effects: CRT, depth, ASCII | `graphics_ext.h` | DONE |
| RUST-EXT-005 | PBR material, effect and render-pipeline settings | `graphics_ext.h`, `effects.h`, `engine_layer.h` | DONE |
| RUST-EXT-005b | The extended `PbrMaterialEXT` with its texture slots and per-slot transforms | `graphics_ext.h`, `engine_layer.h` | DONE |
| RUST-EXT-006 | `.cnb` container: open, metadata, Texture2D | `cnb.h` | DONE |
| RUST-EXT-013a | `.cnb` Model: graph, geometry, materials | `cnb.h` | DONE |
| RUST-EXT-013b | `.cnb` loader registry, writer and native content manager | `cnb.h`, `content.h` | DONE |
| RUST-EXT-013c | `.cnb` SpriteFont and SoundEffect, encode and decode | `cnb.h` | DONE |
| RUST-EXT-007 | Device layer: power, locale, clipboard, display info | `devices.h` | DONE |
| RUST-EXT-008 | Raw joystick enumeration, capabilities and capture | `input_joystick.h` | DONE |
| RUST-EXT-014a | Text input, IME composition and candidate lists | `input_text.h` | DONE |
| RUST-EXT-014b | Device enumeration, hot-plug events and mouse cursors | `input_devices.h`, `input_cursor.h` | DONE |
| RUST-EXT-014c | Haptics: enumeration, capabilities, rumble and gain | `input_haptics.h` | DONE, real forces `HARDWARE_PENDING` |
| RUST-EXT-014d | Haptic effects: thirteen families, custom samples, the create/run/update/status lifecycle | `input_haptics.h` | DONE, real playback `HARDWARE_PENDING` |
| RUST-ABI-013 | Gate that both acquisition modes declare the same routes | `tools/native-abi/verify.py` | DONE |
| RUST-EXT-009 | Sensors: accelerometer, compass, gyroscope | `sensors.h`, `input_devices.h` | DONE, real readings `HARDWARE_PENDING` |
| RUST-EXT-010a | Engine layer: render-pipeline settings, presets, normalization, text | `engine_layer.h` | DONE |
| RUST-EXT-010c | Engine layer: the `KHR_materials_*` extension set | `engine_layer.h` | DONE |
| RUST-EXT-010b | The other 808 engine-layer routes across 224 families | `engine_layer.h` | **all 857 routes of `engine_layer.h` are bound**, qualified on a GPU-backed `OPENGLES3` artifact with `CNA_CNAEXT=ON` and on the headless one, plain and `--all-features`. The trigger the scope decision named arrived and was taken: see [docs/gpu-evidence.md](gpu-evidence.md) for the host evidence and [docs/engine-layer-scope.md](engine-layer-scope.md) for the family-by-family table and the upstream contracts the tests measured. The last fourteen routes needed a native mesh-part handle the XNA projection cannot produce, so `cna::extensions::engine::NativeMeshPart` builds one | DONE |
| RUST-EXT-015a | CNB's primitive writer, cursor, chunk navigation, codecs and `.cnj` | `cnb.h` | **DONE**: all 178 remaining routes bound as `cna::extensions::cnb`. The typed byte writer and cursor carry CNB's canonical encoding *and* its checks -- length-prefixed UTF-8 against a read limit, the fixed 48-byte keyframe, seconds a `TimeSpan` can hold -- so a game with an asset type of its own no longer hand-rolls the format on both sides. `compile_cnj` is the reason this mattered most: `.cnj` is what CNA's glTF import writes and this crate has no reader for it. Eleven `curve.h` routes are bound as a marshalling bridge, recorded in the `value-maths-in-rust` rule | DONE |
| RUST-EXT-015b | Skinned models, morph targets, animation players and skinning data | `models.h` | **DONE for everything reachable**: 96 routes bound across `cna::extensions::models` -- `SkinnedModel` with its clips and pose evaluation, `SkinningData` and `AnimationPlayer`, `MorphTargetData` with its weight track and blend, `ModelAnimations` for a scene's own clips, the per-slot samplers and the infinite projection. `SkinnedPbrEffect`'s bone palette now has a source. What is left is 78 `STRICT_XNA_BACKING` routes the Rust projection implements itself and 34 `cna_model_*_ext` routes that hang off a native `CNA_ModelHandle` the managed `Model` cannot produce -- tracked as RUST-EXT-015g | DONE |
| RUST-EXT-015g | The model CNA loads, and its glTF facts | `models.h` | **DONE**: 80 routes bound as `cna::extensions::native_model`, 30 deliberate non-bindings. The premise was wrong: `crates/cna/src/graphics/model.rs` already *is* the XNA `Model`, so binding the construction routes would have added a second, weaker one. What was missing is the model CNA's own pipeline loads -- its import report, cameras, skins and material variants. Views carry no lifetime because a bone view outlives `cna_model_destroy`, measured rather than assumed. Teardown faults: `RUST-UPSTREAM-021` | DONE |
| RUST-EXT-015k | The XNA base every graphics resource shares | `graphics_resource.h` | **DONE**: 12 of 12 bound. `Name`/`SetName` now go through CNA instead of a Rust `Mutex` that CNA never heard about -- the divergence that made the device's `ResourceDestroyed` event report an empty name for every resource a Rust caller had named. The four properties with no XNA counterpart to fold into live on a new `NativeGraphicsResource` trait, each kept separate from its XNA neighbour because they answer different questions: CNA's `ToString` (bare type name vs XNA's qualified one), CNA's disposal flag (the object) vs Rust's `IsDisposed` (the handle), CNA's `uint64` tag vs XNA's `Arc<dyn Any>`, and CNA's disposing event vs XNA's. Measured in `crates/cna/tests/extensions_graphics_resource.rs` | DONE |
| RUST-EXT-015l | The keyboard layout, and what the platform calls each key | `input_keyboard.h` | **DONE**: 10 bound, 2 deliberate non-bindings. The eight `_ext` routes ask the platform's *live* layout which key a scancode produces and what each is named -- nothing XNA had and nothing Rust can derive. `KeyboardState`'s own set operations stay Rust, over the same bit words, with a `to_native` that hands the identical struct to the value routes. `copy_string` is bound and measured: it answers the qualified type name for every snapshot, which is exactly what XNA's unoverridden `ToString` does -- and the opposite of `graphics_resource`, where CNA gives the bare name and XNA the qualified one. Measured in `crates/cna/tests/extensions_keyboard_layout.rs` | DONE |
| RUST-EXT-015m | The desktop mouse, and CNA's dead-zone algorithm | `input_mouse.h`, `input.h` | **DONE**: 10 bound, 8 deliberate non-bindings. XNA's `Mouse` knows only where the cursor is inside the window; the nine `_ext` routes reach past that -- the desktop position read and warped, relative (pointer-lock) mode, capture, and a process-wide clicked event with the hooks to raise one and reset the state. `set_capture` and `warp_global` answer whether the backend *accepted* the request, which the projection returns rather than folding into the `Ok`, because a platform that declines is a documented outcome. `cna_gamepad_apply_dead_zone` is bound too: it is CNA's canonical clamping algorithm over caller-supplied raw values, so the binding calls it rather than restating it. The snapshot queries on all three input headers stay Rust, over the bits CNA set. Measured in `crates/cna/tests/extensions_mouse_desktop.rs` | DONE |
| RUST-EXT-015n | CNA's format arithmetic, and textures with no device | `texture.h` | **DONE**: 14 bound, taking the header to 19 of 21. Two halves. The format routes answer what a `SurfaceFormat` costs, how many texels a block covers, what alignment it wants and whether an element size divides a unit -- all arithmetic a binding *could* restate, which is exactly the mistake `RUST-UPSTREAM-024` documents upstream, so they are called. The creation routes make textures with **no graphics device**: a standalone default, one from pixels, one decoded from a file. Those get their own `StandaloneTexture` type rather than being forced into `Texture2D`, because an XNA `Texture2D` is a `GraphicsResource` and a `GraphicsResource` has a device. Qualified by a real PNG round-trip through CNA's own encoder and decoder, and by the measured difference between the two kinds: a device texture holds a renderer resource, a standalone one does not. `crates/cna/tests/extensions_texture.rs` | DONE |
| RUST-EXT-015o | The per-game runtime: frame budget, launch parameters, title container | `runtime.h` | **DONE**: 18 bound, taking the header to 40 of 42. The launch parameters were the reason: `Game::LaunchParameters` in Rust is a `HashMap` CNA never sees, and CNA keeps its own per-game dictionary that a command line actually lands in. Both are published, with `import_into` as the bridge, rather than one pretending to be the other. Measuring the parser pinned XNA's real rules -- a `:` separator, leading `/` and `-` trimmed, first key wins -- and found `RUST-UPSTREAM-026`, where the header's "adds or replaces" is neither. Also the frame budget in all three of CNA's spellings, the run loop's own flag, the title path override and `TitleContainer::read`. Measured in `crates/cna/tests/extensions_game_runtime.rs` | DONE |
| RUST-EXT-015p | The XNB reader, and the extension point that is safe to project | `content_readers.h` | **DONE**: 35 bound, taking the header to 49 of 62 and to zero undecided. This is the follow-through on a promise the `reflective-reader-writes-at-caller-offsets` rule already made: that rule rules out the builder which writes at caller-supplied byte offsets, and names `cna_content_type_reader_manager_register` as the other extension point, which inverts the dangerous half -- the callback is handed a borrowed reader and reads *from* it. That is now bound as `TypeReader` + `register_type_reader`. The callback-scoped reader carries a lifetime, because upstream invalidates it before the callback returns. Qualified end to end: the read callback is driven through CNA's own `read_untyped` rather than called directly, over a real `StorageStream`, and `read_bytes_exact` and both limit checks are measured against bytes on disk. `crates/cna/tests/extensions_content_reader.rs` | DONE |
| RUST-EXT-015q | The media and audio a game builds rather than loads | `video.h`, `media.h`, `audio.h`, `xact.h` | **DONE**: 37 bound, 2 deliberate non-bindings, 4 deferred to RUST-EXT-017, taking four headers to zero undecided. XNA hands a game its `Video`, `Song` and `SongCollection`; CNA lets it build them, which is what a caller with a file on disk needs. Also the disposal facts CNA holds for every XACT object, the engine's renderer identity, `SoundEffect::FromAsset`, the float/queue/clear/update streaming paths, `Microphone::CheckAllBuffers` and the process audio capability. Two routes are *not* bound: `RUST-UPSTREAM-027` found CNA's sample arithmetic disagreeing with XNA where the Rust projection agrees, so binding them would have replaced a correct answer with a wrong one. Measured in `crates/cna/tests/extensions_audio_ext.rs` and `extensions_media_ext.rs` | DONE |
| RUST-EXT-015r | The last of the ABI | `vertex_resources.h`, `index_resources.h`, `render_target.h`, `graphics.h`, `display.h`, `runtime_window.h`, `runtime_graphics_manager.h`, `input_joystick.h`, `input_devices.h`, `input_text.h`, `storage.h`, `media_player.h`, `media_library.h`, `texture_volume.h`, `core_ext.h`, `graphics_ext.h` | **DONE**: 61 bound, 3 deliberate non-bindings. This is the slice that took **UNREVIEWED to zero**: every one of CNA's 4,054 routes now carries a binding decision and the census gate passes. `ContentLost` for the dynamic buffers and render targets; the back buffer and the packed-colour paths; render-target binding; presentation parameters cloned, measured and applied; a buffer's own vertex layout read back; borderless, minimize, restore and the platform window handles; the presentation mode; process-wide joystick hotplug with its raise and reset hooks; the device sensors, clipboard and battery; the storage root and app name; the media queue, the song-ended predicate and the picture token; the assembly title; and the ASCII post-process draw. Measured in `crates/cna/tests/extensions_device_surface.rs` | DONE |
| RUST-EXT-016 | Adoption constructors for `SpriteFont` and `SoundEffect` | `content.h`'s `cna_content_manager_load_sprite_font` and `load_sound_effect` | DEFERRED_TRACKED: both routes publish an owned handle, and neither Rust type can currently adopt one -- `SpriteFont` and `SoundEffect` are built from their own constructors and have no path that takes a handle CNA made. Publishing a handle those types cannot own would be worse than waiting, so the loads stay unbound until the adoption path exists |
| RUST-EXT-017 | Make the XACT disposing event fire on engine teardown, and bind CNA's | the four `*_subscribe_disposing_ext` routes in `xact.h` | DEFERRED_TRACKED: measured -- `WaveBankState::dispose_xact_child` and `SoundBankState`'s equivalent destroy the handle without emitting the Rust `disposing` event, so a handler registered through `AddDisposingHandler` does **not** fire when the engine that owns the bank is disposed, which is exactly when XNA's own `Disposing` fires. CNA raises its event on that path. Binding the native subscription alone would paper over the Rust defect; the fix and the binding belong in one slice, with the native event as the cross-check |
| RUST-EXT-018 | Make the XNA-shaped `ContentLost` handlers fire | `DynamicVertexBuffer`/`DynamicIndexBuffer::AddContentLostHandler` | DEFERRED_TRACKED: measured -- `crates/cna/src/graphics/buffer.rs` adds and removes those handlers and emits them **nowhere**, so a caller that registers one waits forever. Nothing in Rust knows the device was reset, which is why CNA's own event is now bound as `NotifiesContentLost` and is the route that works. The remaining work is bridging the XNA-shaped event onto it, which means the buffer holding a subscription for its own lifetime |
| RUST-EXT-015c | `ShaderEffect`, `ColorMatrixEffect`, and what an effect can say about itself | `effects.h` | **DONE**: 66 routes bound as `cna::extensions::shader_effect` and `cna::extensions::effects`, 12 deliberate non-bindings. Construction and compilation are separate questions and the test says so rather than asserting which renderer built the artifact. Caught the transposed PBR texture slots | DONE |
| RUST-EXT-015d | System tray, file dialogs, camera capture, message boxes and the vibrate controller | `devices.h` | 38 routes still unbound; `RUST-EXT-007` bound power, locale, clipboard and display info and stopped there | ACTIONABLE_LOCAL |
| RUST-EXT-015e | The rest of the extension surface, by header | `graphics_device.h` (40), `input_gamepad.h` (37), `runtime_components.h` (34), `devices.h` (34), `content.h` (30), `input_touch.h` (25) and a long tail | 416 routes still undecided, down from 1,074. `sensors.h`, `cnb.h`, `effects.h`, `models.h` and the object dictionary came out of this row and have their own. Each remaining family needs the same treatment: read what the Rust projection already has *before* deciding anything is missing, which is what changed the answer for `models.h` | ACTIONABLE_LOCAL |
| RUST-EXT-015f | .NET value-type boilerplate: `_equals`, `_get_hash_code`, `_init`, `_copy_type_name`, `_copy_string` | across the extension headers | **DONE as four rules with checked evidence**, 197 routes. Each has its own true reason rather than one blanket claim, and `_copy_string` is named route by route because the same routes exist for types this crate has no `ToString` for. Where it did not, the `ToString` was added: the five sensor readings now format exactly as upstream does | DONE |

| RUST-EXT-015h | The motion sensor, sensor events, and the deterministic backends | `sensors.h` | **DONE**: 50 routes bound, 12 answered by Rust `ToString` implementations, none left undecided. The motion sensor is a whole device the projection did not have. The `_for_tests_ext` hooks are bound because no machine here has any of these sensors, and upstream says of the compass backend that "without it there is no compass on any verification machine and no way to reach a single line past the unsupported refusal". What each hook actually does is measured rather than assumed -- `set_supported_for_tests` does not move `is_supported` or `state`, and `start()` still refuses behind it | DONE |
| RUST-EXT-015i | The content `Tag` a processor wrote | `content_readers.h` | **DONE**: 14 routes bound as `cna::extensions::object_dictionary`, which closed the `RUST-EXT-015g` deferral. Safe to project because every entry is tagged: the caller asks the kind, CNA answers, and this side picks the destination type from that answer | DONE |
| RUST-CENSUS-002 | Bound routes with no safe call site | this repository | 894 routes are declared in `cna-sys` and resolved at load but have no caller in the safe layer. Reported by the inventory rather than gated: declaring a whole family so a missing symbol fails at load is deliberate, and a read-only projection legitimately leaves the C mutators uncalled. Working the list down is what turns "bound" from a declaration into a measurement | ACTIONABLE_LOCAL |
| RUST-CENSUS-003 | Decide the remaining 416 undecided routes | canonical headers | every route needs a binding status with a reason, and a task when the reason is a block or a deferral. The gate fails while any route is `UNREVIEWED` | ACTIONABLE_LOCAL |

## Platform and packaging

| ID | Subject | Target | Status |
|---|---|---|---|
| RUST-PLATFORM-001 | Windows dynamic loader | source complete; no Windows Rust target on this host, so `COMPILE_NOT_VERIFIED_PLATFORM` | DONE |
| RUST-PLATFORM-002 | macOS loader | already `#[cfg(unix)]`; needs a runtime run | BLOCKED_PLATFORM |
| RUST-PLATFORM-003 | WebAssembly | re-measured: CNA's wasm C ABI exists and the binding now has direct linkage, so only the toolchain blocks it -- no wasm std is installed and there is no `rustup` to add one | BLOCKED_PLATFORM |
| RUST-PLATFORM-004 | Direct-linkage mode so the tables fill from typed `extern "C"` declarations | one route inventory, two acquisition mechanisms, one gate over both | DONE |
| RUST-ABI-011 | Type `MediaApi`'s 203 routes, which were outside the symbol gate entirely | `SYMBOL_TYPE_MISMATCH` covers 1,387 acquisitions, up from 1,184 | DONE |
| RUST-ABI-012 | Put a floor under the acquisition scan so a blind gate fails loudly | the scan cannot silently match nothing | DONE |
| RUST-PACKAGE-001 | `cargo package` file list, notices, no native binary | LICENSE and NOTICE.md now ship with each crate; an outside consumer builds from the packaged file set alone | DONE |
| RUST-PACKAGE-003 | `cargo package -p cna-rust` needs `cna-rust-sys` published first | publish order recorded | BLOCKED_UPSTREAM |
| RUST-PACKAGE-002 | MSRV 1.74 evidence | `tools/msrv/audit.py` denylist; two real violations found and fixed. Compiling with 1.74 is still `MSRV_RUNTIME_NOT_RUN` | DONE |
| RUST-TEMPLATE-001 | Template against the live artifact | 60/600 frames, re-run on ABI 0.21 against both the `HEADLESS` artifact and the GPU-backed `OPENGLES3` one under Xvfb | DONE |
| RUST-TEMPLATE-002 | Template modern-extension canary | `--extensions-smoke` | DONE, and re-qualified on a renderer that draws: the canary used to fail outright on every GL-family renderer because its standalone-device half needs a windowless `GraphicsDevice`. It now tolerates exactly that refusal, keeps the `.cnb` model half that was being lost with it, and reports the engine-layer version |
| RUST-TEMPLATE-003 | Generated standalone project on the live ABI | re-verified on ABI 0.21; the canary caught the generator dropping `cna-sys`'s new build script, now taken from the manifest. Re-generated and re-run against the `OPENGLES3` artifact with the engine layer bound: `--extensions-smoke`, `--frames 60` and `--stability-test` all exit zero | DONE |

## Qualification, 2026-08-31 (ABI 0.21 milestone)

Run against the HEADLESS **ABI 0.21** artifact with SHA-256
`3a976d2494580ca9af45fbb2be30c13b01d05477f98ae80796ef26898c97d812`, built out
of tree from `cnanext` at `599d14e5` (clean) and `sharp-runtimenext` at
`4a49afb0` (clean). Library exports and header declarations agree exactly at
4,054.

| Gate | Result |
|---|---|
| `cargo check --workspace --all-targets` | PASS, no warnings |
| `cargo test --workspace --all-features` | PASS: 51 suites, 156 assertions, 0 failures |
| `cargo test --workspace` (dynamic linkage) | PASS: 40 suites, 133 assertions |
| `cargo test --workspace --no-default-features --features direct-link` | PASS: 40 suites, 133 assertions -- identical |
| `cargo doc --workspace --no-deps` | PASS, no warnings |
| native ABI verifier | PASS: 1,591 functions, 5,488 prototype positions, 2,272 C/Rust measurements, 121 layouts, 121 layout field sets, 23 callbacks, 790 constants, 1,587 symbol acquisitions, 1,591 linked declarations, 0 findings, 0 unaudited |
| ABI mutation tests | PASS: 33 |
| API-compat mutation tests | PASS: 28 |
| canonical route inventory | PASS: 4,054 canonical, 1,591 bound, 0 unmapped, 0 stale overrides, 0 unused rules |
| runtime capability provenance | PASS: 35 rows, artifact and ABI 0.21 confirmed |
| selected XNA profile (strict) | PASS: 257 types, 2,964 members, 0 diagnostics, 0 unmeasured categories |
| complete XNA runtime profile | PASS: 331 types, 3,640 members, 0 missing, 0 diagnostics |
| Content Pipeline profile | 125 missing types -- **stated product boundary**, see [content-pipeline-decision.md](content-pipeline-decision.md) |
| superset discovery profile | 125 missing types, all of them the pipeline |
| leak verifier | PASS: 0 diagnostics, empty allowlist, 0 out-of-profile types, 0 unmeasured |
| MSRV source audit | PASS |
| packaged-source consumer | PASS: 7 sys files, 158 crate files, 0 workspace path leaks |
| direct-link consumer | PASS: links `libcna_c_api.so`, imports no `dlopen`/`dlsym`/`dlclose`, runs a real `GraphicsDevice` lifecycle |
| template: build, 60, 600, `--extensions-smoke` | PASS |
| generated standalone project: build, 60, 600, `--extensions-smoke`, no developer path | PASS |
| `git diff --check`, both writable repositories | clean |
| `cnanext` / `sharp-runtimenext` modified by this session | 0 files, both clean |
| MSRV 1.74 runtime | NOT_RUN -- no 1.74 toolchain on this host |
| `rustfmt`, `clippy` | NOT_AVAILABLE |
| sanitizers | NOT_RUN -- no instrumented artifact was built |
| WebAssembly target | NOT_AVAILABLE -- no wasm std installed, no `rustup` to add one |

## Qualification, 2026-08-31 (previous, ABI 0.20 milestone)

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
