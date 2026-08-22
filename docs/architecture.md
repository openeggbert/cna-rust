# Architecture

## Layers

```text
Rust game
  -> cna::Microsoft::Xna::Framework::*   strict XNA projection
  -> cna safe facades + cna::extensions CNA-only surface
  -> crate-private native bridge         audited dynamic loading/callbacks
  -> cna_sys                             raw ABI 0.7 declarations
  -> CNA stable C ABI                    cna_* only
  -> CNA C++
```

The published package is `cna-rust` and its library crate is `cna`. The raw
package is `cna-rust-sys` and its crate is `cna_sys`. Dependencies use explicit
Cargo `package = ...` renaming; neither identity relies on Cargo's hyphen
normalization.

The compatibility hierarchy preserves XNA identifiers and casing. Its formal
language rules are normative in [xna-rust-mapping.md](xna-rust-mapping.md).
CNA-only renderer information lives under `cna::extensions`, not on strict XNA
types as an inherent member.

## Native boundary

`cna-sys` contains a reviewed ABI 0.7 slice: fixed-width aliases, `repr(C)`
structures, callbacks, constants, and function-pointer declarations. The safe
crate loads only unmangled `cna_*` symbols and rejects any ABI other than the
declared version before resolving the operational symbols.

Library discovery checks, in order:

1. `CNA_NATIVE_LIBRARY` as an exact file;
2. platform library names inside `CNA_NATIVE_DIR`;
3. common build/install locations derived from `CNA_ROOT`;
4. the executable directory and platform library name.

A failure reports every attempted path and loader diagnostic. There is no fake
fallback. The current loader is implemented and runtime-tested on Unix; other
platform loaders are pending and return `UnsupportedPlatform`.

Unsafe code is confined to `native.rs` and callback trampolines in `game.rs`.
Each operation states its pointer/handle/lifetime invariant, and
`unsafe_op_in_unsafe_fn` is denied. The strict public API contains no unsafe
function, raw pointer, integer native handle, or `cna_sys` type.

## Game and lifetimes

The `Game` trait is the user lifecycle contract. `GameContext<'callback>`
composes the native, host-owned portion of XNA `Game`. A `GraphicsDevice`
borrowed from it inherits the callback lifetime, matching CNA's rule that the
C handle is invalid after callback return.

Owned resources (`Texture2D`, `SpriteBatch`) hold an `Arc` keeping the dynamic
library loaded plus one native handle. `Dispose` is idempotent; `Drop` calls it.
The handle becomes invalid only after successful native destruction, so an
error cannot silently lose ownership state. Traits model the public
`GraphicsResource` and `Texture` base contracts.

CNA ABI 0.7 checks that C-owned child resources are absent before
`cna_game_destroy`, but invokes `unload_content` inside shutdown after that
check. The runner therefore gives the Rust game a pre-destroy `UnloadContent`
release point after the loop and then accepts CNA's normal shutdown callback.
Resource cleanup must be idempotent. The 60/600-frame tests cover explicit
dispose followed by drop and this double unload notification.

## Verification

The API verifier hashes the seven XNA 4.0 Windows runtime assemblies, extracts a
neutral CLR contract with Mono, applies the formal mapping rules, and inspects
the Rust surface through compiler rustdoc JSON. Rust 1.74 requires unstable
rustdoc JSON, so only the tool subprocess receives `RUSTC_BOOTSTRAP=1`; the
library and tests remain stable Rust 1.74. Unimplemented comparison categories
are reported as unmeasured rather than false zeroes.

The ABI verifier parses all canonical CNA C headers, compares the reviewed
inventory by symbol and arity, optionally scans ELF exports, and calls
`cna_get_abi_version`. It is platform-honest: the ELF export check is Linux
evidence only.

## Current scope

The implemented coherent runtime slice covers native game callbacks, signed
game time, callback-scoped graphics device access, viewport/clear, encoded
texture creation, sprite batching, keyboard capture, renderer facts, and clean
shutdown. Pure Rust value work covers an initial subset of vector, quaternion,
matrix, color, point/rectangle, plane/ray, and bounding behavior.

It does not yet implement the full Game object model, XNB content, remaining
graphics, complete input, models, audio/XACT, media/storage, GamerServices, or
the complete value method surface. The strict verifier quantifies that gap.
