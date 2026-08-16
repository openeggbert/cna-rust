# Architecture

```text
Rust game
   ↓
cna (safe, idiomatic API; Result and RAII)
   ↓
cna-sys (raw generated/audited C declarations)
   ↓
CNA stable C ABI
   ↓
CNA C++ core → Sharp Runtime, subsystems, renderers
```

This repository is named `cna-sys`, but hosts both layers as one Cargo
workspace so raw declarations and safe wrappers are reviewed and versioned
together. Applications should depend on `cna`; direct `cna-sys` use is reserved
for low-level integration work.

The safe crate keeps value math in Rust, maps native errors to `Result`, and
will own native resources with RAII and `Drop`. Borrowed resources carry Rust
lifetimes and cannot be released. Raw handles never appear in safe public APIs.
High-frequency draw commands and buffers cross in batches; input crosses as
snapshots.

The raw crate remains intentionally empty until the canonical C headers exist.
It will use exact fixed-width layouts, explicit ABI/struct versioning, UTF-8,
opaque generation-checked handles, and callback context pointers. Generated
output must be checked and accompanied by ABI layout/link smoke tests.

Sharp Runtime is below the C ABI. Rust ownership models CNA handles only and
must never depend on Sharp Runtime's objects, exceptions, or layouts.
