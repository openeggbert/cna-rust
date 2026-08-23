# Native lifetime stress and sanitizer path

The ordinary crash-isolated suite runs when `CNA_NATIVE_LIBRARY` identifies an
exact ABI-0.7 library:

```bash
CNA_NATIVE_LIBRARY=/path/to/libcna_c_api.so \
  cargo test --workspace --all-features --test native_stress -- --nocapture
```

It covers repeated parent/resource lifetimes, double `Dispose`, `Dispose` plus
`Drop`, live-child shutdown, a contained callback panic, and test-bridge fault
injection for game creation, texture-information rollback, and the reported
game-destroy failure path. Fault injection is compiled only by the
`native-fault-injection` feature and remains inert unless an isolated test child
sets `CNA_RUST_TEST_FAULT`.

For sanitizer evidence, build CNA C/C++ separately with AddressSanitizer and
UndefinedBehaviorSanitizer enabled, while preserving the exact ABI 0.7 exported
contract, then run:

```bash
CNA_NATIVE_LIBRARY=/path/to/sanitized/libcna_c_api.so \
  bash tools/native-stress/run-sanitized.sh
```

This is deliberately opt-in and is not part of Rust 1.74's normal build. A run
against an unsanitized library is not sanitizer evidence. Canonical CNA HEAD
currently fails its own renderer-identity compile assertion before such a
sanitized library can be produced unmodified, so no sanitizer pass is claimed
until the upstream blocker is resolved or an exact reviewed ABI-0.7 sanitized
artifact is supplied.
