# CNA-Rust

CNA-Rust is an early, measurable safe Rust projection of Microsoft XNA
Framework 4.0 backed by CNA C++. Its Graphics projection, compressed and
uncompressed XNB pipeline, Framework device management, Touch/Gesture,
Storage, GamerServicesComponent, 2D sprite/font, Model graph, stock effects,
and typed device/buffer foundations are functional; it is **not** yet an
XNA-complete binding and does not compile XNA C# source as Rust.

```text
cna::Microsoft::Xna::Framework::*
  -> safe Rust facades
  -> audited private bridge
  -> cna_sys
  -> CNA ABI 0.7 cna_* symbols
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

- Rust 1.74 workspace format, check, all-feature tests, Clippy, and docs exit
  zero. Clippy still reports audited compatibility warnings; they are not
  globally hidden.
- XNA 4.0 Windows runtime inventory remains 257 CLR types and 2,964 members;
  259 Rust types are expected and 203 strict types now exist.
- Strict diagnostics are 56, all of them missing types. Graphics,
  Framework/core, Input, Storage, and GamerServices have zero missing types.
  Only Design (13), Audio (19), and Media (24) remain. Missing members and
  constructor/overload/property/event mapping mismatches are zero.
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
  device. CNA ABI 0.7 has no candidate-ranking route, so `RankDevices` is an
  explicit backend blocker.
- Touch/Gesture uses reviewed CNA state, capability, panel, and gesture routes.
  HEADLESS reports no touch hardware and no queued gesture; none is fabricated.
- Storage maps synchronous CNA selectors into deterministic one-shot Begin/End
  results and routes all container/filesystem/stream work through CNA. Managed
  path containment closes a qualified ABI-0.7 traversal gap; device/container/
  stream ownership and disposing callbacks are tested.
- `GamerServicesComponent` participates in normal GameComponent lifecycle
  without expanding the selected profile into Gamer, Guide, Avatar, or network
  services.
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
- The XNA-derived corpus passes 145 named observations and 146 assertions.
- The reviewed ABI slice is 431 functions. It has 1,509 full prototype type
  positions and 936 independent C/Rust measurements across 56 layouts, five
  callback signatures, scalar representations, and 243 constants, all with
  zero mismatches.
- Linux x86-64 HEADLESS validation covers 209 created native game lifetimes,
  ten buffer-binding cycles, ten SpriteFont/content cycles, ten Effect
  parent/child cycles, ten Model/XNB cycles, and ten stock-effect/Texture3D/
  OcclusionQuery cycles, plus ten Framework/Touch/Storage/GamerServices cycles
  and isolated callback failure/recreation. The unchanged template remains the
  native consumer canary.

`GraphicsDevice` has durable shared identity while its private CNA handle
remains callback-scoped. Resources retain device association without owning
the native device, share deterministic invalidation, and are released before
parent destruction. Bound buffers/render targets cannot be destroyed while CNA
may retain a raw binding. User `UnloadContent` remains exactly once and is
separate from internal pre-destroy child cleanup. ContentManager owns loaded
resources, not the native Game.

Canonical CNA HEAD `1bb2145d99ed572dd4eb15009c34e2e5f410fcf0` still fails an
unmodified C API build at `CnaCApiCoreExt.cpp:250`: the renderer identity
assertion reduces to `49 == 50`. The CNA checkout was not modified. Runtime
evidence therefore uses the labelled experimental ABI-0.7 HEADLESS artifact;
the Rust loader continues to reject ABI 0.8.

| Platform | Status | Evidence |
|---|---|---|
| Linux x86-64, HEADLESS | Experimental runtime verified | 60/600 native frames with the qualification above |
| Windows | Planned | No CNA-Rust runtime run |
| macOS | Planned | No loader/runtime run |
| WebAssembly | Unsupported | No compatible CNA WASM C ABI verified |
| Android | Unsupported | No native lifecycle/window/input integration verified |

## Native setup

Supply a CNA C API library matching ABI 0.7:

```text
CNA_NATIVE_LIBRARY=/absolute/path/to/libcna_c_api.so
```

Alternatively set `CNA_NATIVE_DIR` to a library directory or `CNA_ROOT` to a
CNA source/install root. Missing-library errors list the searched paths and the
expected platform filename. The current dynamic loader supports Unix; Windows
loading remains work.

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

# Records the intentionally incomplete strict scoreboard without hiding it.
XNA_REFERENCE_PATH=/path/to/xna4/windows \
  python3 tools/api-compat/verify.py --report-only \
  --output target/xna-api-report.json

python3 tools/api-compat/verify.py --leak-only

python3 tools/native-abi/verify.py \
  --cna-root /path/to/cna \
  --library /path/to/libcna_c_api.so

CNA_NATIVE_LIBRARY=/path/to/libcna_c_api.so \
  cargo test --workspace --all-features --test native_stress -- --nocapture
```

The normal API verifier intentionally exits nonzero while genuine gaps remain.
It uses Mono for neutral CLR extraction and compiler rustdoc JSON for Rust API
inspection. The mapping transforms CLR concepts before comparison; it does not
compare raw C# syntax to Rust syntax.

`tools/native-stress/run-sanitized.sh` is an optional ASan/UBSan path for a
separately instrumented exact ABI-0.7 CNA library. Sanitizer status for this
run is `not-run`; native crash absence is not allocator-level leak proof.

See the [normative mapping](docs/xna-rust-mapping.md),
[architecture](docs/architecture.md), [Graphics evidence](docs/graphics-evidence.md),
[LZX evidence](docs/lzx-xnb-evidence.md),
[Framework evidence](docs/framework-evidence.md),
[Touch evidence](docs/input-touch-evidence.md),
[Storage evidence](docs/storage-evidence.md), and [measured roadmap](plan.md).

## Packaging

Neither crate is published on crates.io yet. Local consumers must use an exact
path dependency. A released version, native packaging strategy, Windows/macOS
loaders, docs.rs behavior, and license/notice package audits are release gates.

## License

CNA-Rust is licensed under the [Microsoft Public License](LICENSE), matching
CNA. See [NOTICE.md](NOTICE.md) for third-party notices.
