# CNA-Rust

CNA-Rust is an early, measurable safe Rust projection of Microsoft XNA
Framework 4.0 backed by CNA C++. Its Graphics projection, compressed and
uncompressed XNB pipeline, Framework device management, Touch/Gesture,
Storage, GamerServicesComponent, managed Design converters, 2D sprite/font,
Model graph, stock effects, typed device/buffer foundations, Audio/XACT, and
Media/Video projection are functional. The selected XNA 4.0 Windows runtime
Rust projection is **structurally complete**. This is not a claim that every
XNA platform/profile or runtime backend is complete, and XNA C# source is not
Rust source compatible.

```text
cna::Microsoft::Xna::Framework::*
  -> safe Rust facades
  -> audited private bridge
  -> cna_sys
  -> CNA ABI 0.21 cna_* symbols
  -> CNA C++
```

## Crates

- Package `cna-rust`, crate `cna`: safe XNA-facing API and CNA extensions.
- Package `cna-rust-sys`, crate `cna_sys`: raw declarations for the reviewed
  CNA C ABI slice. Applications normally should not use it directly.

XNA identifiers keep their original casing inside
`cna::Microsoft::Xna::Framework`. For example:

```rust
use cna::Microsoft::Xna::Framework::{Color, GameTime, Matrix, Vector2, Vector3};

let origin = Vector2::Zero;
let up = Vector3::Up;
let identity = Matrix::Identity;
let clear = Color::CornflowerBlue;
```

Rust host helpers such as `cna::run` live outside the strict hierarchy. CNA-only
features live under `cna::extensions`.

Implementation files are private and organized by coherent value, geometry,
input, lifecycle, graphics-resource, and native-ABI concerns. Those modules do
not create public namespaces: consumers continue to see only the mapped XNA
hierarchy and deliberate CNA extensions.

## Verified status (2026-08-23)

- Rust 1.85 check, all-feature tests, and docs exit zero. This source-tarball
  toolchain has neither `cargo-fmt` nor `cargo-clippy`, so this run records
  `RUSTFMT_STATUS=NOT_AVAILABLE` and `CLIPPY_STATUS=NOT_AVAILABLE` rather than
  claiming either gate passed.
- XNA 4.0 Windows runtime inventory remains 257 CLR types and 2,964 members;
  all 259 expected Rust types exist.
- Normal strict verification exits zero: total diagnostics, missing types,
  missing members, and every constructor/overload/property/event mapping
  mismatch are zero. Graphics, Framework/core, Input, Storage, GamerServices,
  Design, Audio, and Media all have zero local diagnostics.
  `Game`, `GraphicsDevice`, and `SpriteBatch` each have zero local diagnostics.
  Parameter/signature, disposal, type-kind, base/trait/interface, return,
  generic/bound, ref/out, enum/value, flags, and delegate mismatches remain
  zero.
- Unexpected types/members, internal leaks, raw-handle leaks, public unsafe
  APIs, allowlist entries, and unmeasured categories remain zero.
- `ContentManager` and the managed XNB reader pipeline are real:
  typed cache/disposal, custom readers, existing/shared/external resources,
  primitive readers, textures, SpriteFont, Effect, all five stock effects, and
  the complete Model graph are covered. XNA 4.0 LZX framing supports short and
  extended headers, single/multi-frame persistent decoder state, exact output
  and termination, fourteen negative cases, and compressed Model ownership.
  Raw PNG `Texture2D::FromStream` remains a separate route.
- `GraphicsDeviceManager` is Game-associated, publishes the manager/device
  services, retains CNA's Game-owned device, synchronizes preferences, and
  bridges preparing/device lifecycle events without constructing a second
  device. Re-measured on ABI 0.21: `runtime_graphics_manager.h` still has no
  candidate-ranking route, so `RankDevices` remains an explicit backend
  blocker.
- Touch/Gesture uses reviewed CNA state, capability, panel, and gesture routes.
  HEADLESS reports no touch hardware and no queued gesture; none is fabricated.
- Storage maps synchronous CNA selectors into deterministic one-shot Begin/End
  results and routes all container/filesystem/stream work through CNA. Managed
  path containment closes a traversal gap the canonical container routes still
  do not state; device/container/stream ownership and disposing callbacks are
  tested.
- `GamerServicesComponent` participates in normal GameComponent lifecycle
  without expanding the selected profile into Gamer, Guide, Avatar, or network
  services.
- All thirteen Design converter types are complete through a small managed
  Rust projection: stable type identities, explicit cultures, ordered property
  metadata/values, deterministic CreateInstance, and executable reconstruction
  descriptors. No fake ComponentModel hierarchy, arbitrary reflection, or CNA
  ABI route is exposed. Six converters accept XNA component strings; six
  deliberately reject string input while retaining value-string output.
- All nineteen Audio types are complete. SoundEffect, instances, dynamic PCM,
  microphone facades, and XACT use reviewed ABI-0.21 routes with explicit
  ownership and the existing owner-thread FrameworkDispatcher. Multi-listener
  Apply3D reaches its canonical route, with CNA's single-gain-pair mixer as the
  remaining fidelity limit; renderer/look-ahead fidelity and malformed-bank
  failure propagation are recorded CNA blockers; physical capture is hardware
  pending and authored XACT playback is asset pending. No device, sample, bank, or callback delivery is fabricated.
- All 24 Media types are complete as one ownership-safe graph. MediaLibrary,
  seven read-only collection facades, Song, MediaSource, process-global
  MediaPlayer/MediaQueue, owner-thread events, fixed visualization buffers,
  Video, and VideoPlayer use reviewed CNA ABI-0.21 routes. Catalog population,
  picture providers, decoded video, and assets retain explicit platform,
  backend, or asset qualifications. GetTexture wraps a decoded frame in a
  borrowed `Texture2D` that is never destroyed by Rust and is refused once a
  later player call replaces it.
- Typed vertex declarations and vertex/index buffers, device binding/draw/
  reset/back-buffer routes, TextureCube, and render targets are complete. CNA
  calls are never replaced by no-ops; HEADLESS limitations are explicit.
- The base Effect reflection/parameter/pass/technique graph is complete, and
  `EffectPass.Apply` plus both Effect-bearing SpriteBatch Begin overloads use
  real CNA execution routes. Compiled effect bytecode is unsupported by the
  current HEADLESS renderer and returns its exact error.
- BasicEffect, AlphaTestEffect, DualTextureEffect, EnvironmentMapEffect, and
  SkinnedEffect use their distinct real CNA routes. Their construction and
  pass application are HEADLESS-verified; visible shader output is not.
- Model preserves stable parent-owned bone/mesh/part identities without strong
  ownership cycles. Its legal XNB fixture uses real shared buffers and a
  shared BasicEffect; Model.Draw submits through the ordinary indexed device
  route. HEADLESS verifies that command path, not visible Model rendering.
- Texture3D is strict-complete with exact Color transfers and XNB support, but
  the qualified HEADLESS renderer explicitly rejects volume storage with CNA
  error 6. OcclusionQuery's real native state machine is verified.
- `SpriteFont` loads through XNB with one atlas owner, measures strings, and all
  six `DrawString` projections submit native glyph commands.
- The XNA-derived corpus passes 215 named observations and 216 assertions,
  including 20 deterministic Audio and 10 deterministic Media observations.
- The reviewed ABI slice is 730 functions. It has 2,492 full prototype type
  positions and 1,028 independent C/Rust measurements across 62 layouts, seven
  callback signatures, scalar representations, and 262 constants, all with
  zero mismatches.
- Pre-Audio Linux x86-64 HEADLESS validation covers 209 created native game lifetimes,
  ten buffer-binding cycles, ten SpriteFont/content cycles, ten Effect
  parent/child cycles, ten Model/XNB cycles, and ten stock-effect/Texture3D/
  OcclusionQuery cycles, plus ten Framework/Touch/Storage/GamerServices cycles
  and isolated callback failure/recreation. Dedicated Audio stress adds seven
  Game lifetimes, at least 75 effect/instance/dynamic cycles, 50 callback
  deliveries, 60 microphone iterations, 21 engine cycles, and 60 malformed
  bank constructions. The unchanged template remains the native consumer
  canary. Dedicated Media stress adds 20+ library, Song, queue-generation,
  Video, VideoPlayer, and frame-route cycles plus 50 event deliveries.

`GraphicsDevice` has durable shared identity while its private CNA handle
remains callback-scoped. Resources retain device association without owning
the native device, share deterministic invalidation, and are released before
parent destruction. Bound buffers/render targets cannot be destroyed while CNA
may retain a raw binding. User `UnloadContent` remains exactly once and is
separate from internal pre-destroy child cleanup. ContentManager owns loaded
resources, not the native Game.

The unmodified canonical CNA checkout now builds its C API. The previous
milestone's blocker at `CnaCApiCoreExt.cpp:250` -- a renderer identity
assertion reducing to `49 == 50` -- is exactly what ABI 0.20.0 repaired when it
removed eleven renderer identities and moved `CNA_GRAPHICS_RENDERER_MAXIMUM`
from 50 to 49. Runtime evidence uses an out-of-tree HEADLESS build of that
checkout; see [docs/abi-migration-evidence.md](docs/abi-migration-evidence.md).

| Platform | Status | Evidence |
|---|---|---|
| Linux x86-64, HEADLESS | Experimental runtime verified | 60/600 native frames with the qualification above |
| Windows | Loader implemented, not compiled here | `LoadLibraryW`/`GetProcAddress`/`FreeLibrary` are in the source. No Windows Rust target is installed on this host, so nothing here compiles or runs a Windows binary. The path encoding is unit-tested on every host, and the loader body type-checks on Linux against stubbed OS pieces. |
| macOS | Loader shared with Unix, not run | The `#[cfg(unix)]` loader covers macOS in source; no macOS host was available |
| WebAssembly | Blocked by the toolchain | CNA's WebAssembly C ABI exists and the binding now has a direct-linkage mode, so the architecture no longer blocks it; no wasm standard library is installed here and there is no `rustup` to add one. See [docs/platform-evidence.md](docs/platform-evidence.md) |
| Android | Unsupported | No native lifecycle/window/input integration verified |

Full platform evidence: [docs/platform-evidence.md](docs/platform-evidence.md).

## Native setup

Supply a CNA C API library matching ABI 0.21:

```text
CNA_NATIVE_LIBRARY=/absolute/path/to/libcna_c_api.so
```

Alternatively set `CNA_NATIVE_DIR` to a library directory or `CNA_ROOT` to a
CNA source/install root. Missing-library errors list the searched paths and the
expected platform filename. The loader covers Unix and Windows in source; only
Unix is runtime-qualified here.

That is the default, `dynamic-loading`. A consumer that must resolve CNA at
link time instead -- a target with no dynamic loader, WebAssembly being the
reason the mode exists -- selects `direct-link`:

```toml
cna = { package = "cna-rust", default-features = false, features = ["direct-link"] }
```

The same environment variables tell the build script where to link from, and
the safe API is identical. See
[docs/platform-evidence.md](docs/platform-evidence.md) for what is verified.

The CNA canonical headers are `modules/c-api/include/CNA/C` in the CNA
repository. The full upstream ABI is much larger than the currently reviewed
Rust slice.

## Verification

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets --all-features
cargo test --workspace --all-features
cargo doc --workspace --no-deps

XNA_REFERENCE_PATH=/path/to/xna4/windows \
  python3 tools/api-compat/verify.py

# Optionally records the same complete strict scoreboard as JSON.
XNA_REFERENCE_PATH=/path/to/xna4/windows \
  python3 tools/api-compat/verify.py --report-only \
  --output target/xna-api-report.json

python3 tools/api-compat/verify.py --leak-only

python3 -m unittest discover -s tools/api-compat/tests
python3 -m unittest discover -s tools/native-abi/tests

# Fails on a library or language item newer than the declared MSRV.
python3 tools/msrv/audit.py

python3 tools/native-abi/verify.py \
  --cna-root /path/to/cna \
  --library /path/to/libcna_c_api.so

# Fails if the recorded evidence names a different artifact than this one.
python3 tools/runtime-capabilities/generate.py --check \
  --library /path/to/libcna_c_api.so

CNA_ROOT=/path/to/cna python3 tools/c-api-inventory/inventory.py

# Builds an outside consumer from exactly the files the crates would ship.
python3 tools/package-consumer/verify.py

# Direct linkage: builds an out-of-tree consumer, proves CNA resolves at link
# time and that no dynamic loader is imported, and runs a real route.
CNA_NATIVE_LIBRARY=/path/to/libcna_c_api.so \
  python3 tools/direct-link-consumer/verify.py

CNA_NATIVE_LIBRARY=/path/to/libcna_c_api.so \
  cargo test --workspace --all-features --test native_stress -- --nocapture
```

`--all-features` enables `direct-link`, which makes CNA a **link-time**
dependency, so those test binaries need the library on the run-time path as
well as on the link path:

```bash
CNA_NATIVE_LIBRARY=/path/to/libcna_c_api.so \
LD_LIBRARY_PATH=/path/to:$LD_LIBRARY_PATH \
  cargo test --workspace --all-features
```

The default `cargo test --workspace` is dynamic-loading only and needs no
`LD_LIBRARY_PATH`: it resolves the library itself from `CNA_NATIVE_LIBRARY`.

The normal API verifier exits zero for the selected profile. It uses Mono for
neutral CLR extraction and compiler rustdoc JSON for Rust API inspection. The
mapping transforms CLR concepts before comparison; it does not compare raw C#
syntax to Rust syntax or imply other XNA profiles are selected.

`tools/native-stress/run-sanitized.sh` is an optional ASan/UBSan path for a
separately instrumented exact ABI-0.21 CNA library. Sanitizer status for this
run is `NOT_RUN`; native crash absence is not allocator-level leak proof.

See the [normative mapping](docs/xna-rust-mapping.md),
[architecture](docs/architecture.md), [Graphics evidence](docs/graphics-evidence.md),
[LZX evidence](docs/lzx-xnb-evidence.md),
[Framework evidence](docs/framework-evidence.md),
[Touch evidence](docs/input-touch-evidence.md),
[Storage evidence](docs/storage-evidence.md),
[Design evidence](docs/design-evidence.md),
[Audio/XACT evidence](docs/audio-xact-evidence.md),
[Media/Video evidence](docs/media-video-evidence.md),
[runtime capabilities](docs/runtime-capabilities.md), and
[measured roadmap](plan.md).

## Packaging

Neither crate is published on crates.io yet. Local consumers must use an exact
path dependency. A released version, native packaging strategy, Windows/macOS
loaders, docs.rs behavior, and license/notice package audits are release gates.

## License

CNA-Rust is licensed under the [Microsoft Public License](LICENSE), matching
CNA. See [NOTICE.md](NOTICE.md) for third-party notices.
