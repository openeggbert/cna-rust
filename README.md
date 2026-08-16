# CNA Rust

This repository hosts the Rust binding for
[CNA](https://github.com/openeggbert/cna), the native C++ XNA-inspired game
framework. It follows the two-layer design recommended by the CNA binding
analysis:

```text
Rust game → cna (safe API) → cna-sys (raw ABI) → CNA C++ → native renderer
```

## Status

**Early scaffold.** The Cargo workspace, documentation, safe lifecycle shape,
error model, and initial Rust-local values are present. The `cna-sys` crate has
no guessed declarations because `openeggbert/cna` has not implemented its
stable C ABI yet. Calling `cna::run` returns `CnaError::NativeUnavailable`.

## Workspace crates

- `cna-sys`: raw, generated or audited declarations matching CNA's C headers.
  It will contain `unsafe` FFI but no high-level policy.
- `cna`: safe public wrapper using Rust ownership, `Drop`, borrowed lifetimes,
  `Result`, native strings, and collections.

Applications should use `cna`. Keeping both crates here ensures their versions
and ABI expectations cannot drift independently.

## Development

The scaffold targets Rust 1.74 or newer and has no third-party dependencies:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

See [the architecture](docs/architecture.md) and [implementation plan](plan.md).

## License

CNA Rust is licensed under the [Microsoft Public License](LICENSE), matching
CNA. See [NOTICE.md](NOTICE.md) for compatibility and attribution notices.
