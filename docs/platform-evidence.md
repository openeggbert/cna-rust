# Platform evidence

Status date: 2026-08-30

What this binding is known to do on each platform, and what is merely written
down. A row says which of the two it is.

| Platform | Loader source | Compiled here | Run here |
|---|---|---|---|
| Linux x86-64 | `dlopen`/`dlsym`/`dlclose` | yes | yes: 60- and 600-frame native runs, the whole workspace suite, native and ownership stress |
| macOS | the same `#[cfg(unix)]` path | no | no: no macOS host |
| Windows | `LoadLibraryW`/`GetProcAddress`/`FreeLibrary` | no Windows Rust target on this host | no |
| WebAssembly | none, and none is possible | no | no |
| Android | none | no | no |

## Windows

The loader is written and the platform gates that previously made a Windows
build impossible are gone: `Library`, `Native::_library` and
`Native::from_library` were all `#[cfg(unix)]`, so the crate could not compile
for Windows at all regardless of whether a loader existed.

Two things are verified rather than asserted, because no Windows Rust target is
installed here:

- The NUL-termination helper is compiled and unit-tested on **every** host,
  including that an interior NUL is refused rather than silently truncating a
  path, and that an unpaired surrogate survives. That is the mistake a Windows
  loader is most likely to make, and it does not need Windows to catch.
- The loader body type-checks on Linux against stubbed OS pieces, which catches
  a wrong signature or a borrow error before a Windows host sees it.

`extern "system"` rather than `extern "C"`: the two differ on 32-bit Windows.
The path is encoded with `OsStrExt::encode_wide` rather than through `str`,
because a lossy conversion would corrupt a path containing an unpaired
surrogate.

## WebAssembly

The previous milestone recorded "no compatible CNA WASM C ABI verified". That
is no longer true and was worth re-measuring: CNA builds a real WebAssembly C
ABI artifact from its `cna_c_api_wasm` target -- an ES module factory beside a
`.wasm` -- and a current one on this host exposes **4,053** `cna_*` names.
Emscripten is installed here.

The blocker is on the Rust side, and it is two things rather than one:

1. **No Rust WebAssembly target is installed.** This host has `std` for
   `x86_64-unknown-linux-gnu` only, and `rustup` is absent, so no wasm target
   can be added. Nothing here can compile Rust to WebAssembly.
2. **The binding's linkage model does not exist under WebAssembly.** `cna-sys`
   declares function-pointer types and the safe crate resolves every symbol at
   runtime through `dlopen`/`LoadLibraryW`. WebAssembly has no such loader: a
   consumer links the C API into its module and calls the symbols directly.

The second is the real work, and it is a scoped design task rather than a
missing dependency. A WebAssembly route needs a static-linkage mode in which
`Native` is populated from `extern "C"` declarations instead of from symbol
lookups. The reviewed slice's declarations already exist and
`tools/native-abi/generate.py` derives them from the canonical headers, so the
declarations are not the hard part; the hard part is a second linkage
architecture that cannot be runtime-qualified on this host.

Recorded status: `WASM_CANONICAL_ABI=AVAILABLE`,
`WASM_RUST_TARGET=NOT_INSTALLED`, `WASM_BINDING_LINKAGE=NOT_IMPLEMENTED`.

## Android

No native lifecycle, window or input integration is verified, and no toolchain
for it is present.
