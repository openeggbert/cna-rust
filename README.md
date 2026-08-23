# CNA-Rust

CNA-Rust is an early, measurable safe Rust projection of Microsoft XNA
Framework 4.0 backed by CNA C++. It is functional for one small native 2D game
slice; it is **not** yet an XNA-complete binding and does not compile XNA C#
source as Rust.

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
  259 Rust types are expected and 91 strict types now exist.
- Strict diagnostics are 263: 168 missing types and 95 missing members.
  Parameter/signature and disposal mismatches are now zero. Type kind,
  base/trait/interface, return, generic/bound, ref/out, enum/value, flags, and
  delegate mismatches remain zero.
- Unexpected types/members, internal leaks, raw-handle leaks, public unsafe
  APIs, allowlist entries, and unmeasured categories remain zero.
- Curves and all remaining packed vectors are real. `GraphicsResource`,
  `Texture`, and `Texture2D` have zero local diagnostics. Managed graphics
  states are complete and `SpriteBatch` implements all texture draw overloads.
- The XNA-derived corpus passes 105 named observations and 106 assertions.
- The reviewed ABI slice is 53 functions. Full compiler-derived prototypes are
  checked for all 53, and all 313 independent C/Rust prototype/layout/callback/
  constant measurements match.
- Linux x86-64 HEADLESS validation freshly passes 60 and 600 template frames,
  143 isolated successful native game lifetimes, and a generated vendored
  consumer build/test plus 60 frames.

`GraphicsDevice` now has durable shared identity while its private CNA handle
remains callback-scoped. Resources retain device association without owning
the native device, share deterministic invalidation, and are released before
parent destruction. User `UnloadContent` remains exactly once and is separate
from internal pre-destroy child cleanup.

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
cargo fmt --check
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
separately instrumented exact ABI-0.7 CNA library. No sanitizer run is part of
the current evidence.

See the [normative mapping](docs/xna-rust-mapping.md),
[architecture](docs/architecture.md), and [measured roadmap](plan.md).

## Packaging

Neither crate is published on crates.io yet. Local consumers must use an exact
path dependency. A released version, native packaging strategy, Windows/macOS
loaders, docs.rs behavior, and license/notice package audits are release gates.

## License

CNA-Rust is licensed under the [Microsoft Public License](LICENSE), matching
CNA. See [NOTICE.md](NOTICE.md) for third-party notices.
