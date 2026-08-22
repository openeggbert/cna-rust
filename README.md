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

## Verified status (2026-08-22)

- Rust 1.74 workspace check, tests, Clippy, and docs pass.
- XNA 4.0 Windows runtime reference inventory: 257 CLR types and 2,964 members.
- Expected mapped Rust types: 259; actual strict types: 26.
- Strict diagnostics: 1,066 (233 missing types, 833 missing members); zero
  unexpected type/member names and an empty allowlist. Several signature/base/
  enum categories are explicitly not measured yet.
- Raw reviewed slice: 26 declarations; all exist with matching arity among
  2,861 canonical CNA ABI 0.7 header exports.
- Linux headless native validation completed 60 and 600 real draw frames using
  game lifecycle, texture decode, clear, SpriteBatch, keyboard capture, and
  clean disposal.

The native test library required two temporary build-only corrections for an
upstream CNA mismatch: C++ added `NanoVg`, while the ABI 0.7 renderer identity
table/header still stop at `PixiJs`. The checked-out CNA repository was not
modified. This is meaningful Linux/headless integration evidence, but release
runtime support remains experimental until canonical CNA builds unmodified.

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

python3 tools/native-abi/verify.py \
  --cna-root /path/to/cna \
  --library /path/to/libcna_c_api.so
```

The normal API verifier intentionally exits nonzero while genuine gaps remain.
It uses Mono for neutral CLR extraction and compiler rustdoc JSON for Rust API
inspection.

See the [normative mapping](docs/xna-rust-mapping.md),
[architecture](docs/architecture.md), and [measured roadmap](plan.md).

## Packaging

Neither crate is published on crates.io yet. Local consumers must use an exact
path dependency. A released version, native packaging strategy, Windows/macOS
loaders, docs.rs behavior, and license/notice package audits are release gates.

## License

CNA-Rust is licensed under the [Microsoft Public License](LICENSE), matching
CNA. See [NOTICE.md](NOTICE.md) for third-party notices.
