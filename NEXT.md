# CNA-Rust next work

## 2026-08-30 — live CNA ABI 0.20 migration

The binding no longer targets the historical ABI 0.7 development baseline. It
is built and qualified against the live CNA development tree.

```text
CNANEXT_HEAD=72262a33ed5ae7657024c7f1251338748a3feee5
SHARP_RUNTIMENEXT_HEAD=eebebd862121953538e3b84d43384d70a8a1728d

ABI_OLD=0.7.0 / 0x0700
ABI_NEW=0.20.0 / 0x1400
LIBRARY_SHA256=195924825a12290cdd2244fc845e119295de515cf27d1f6b31e1ecc84e93f05d
LIBRARY_EXPORTS=4051

REVIEWED_SYMBOLS_REMOVED=0
REVIEWED_SYMBOLS_ARITY_CHANGED=0
ABI_FUNCTIONS=731
PROTOTYPE_TYPE_POSITIONS=2496
C_RUST_MEASUREMENTS=1028
LAYOUTS=62
CALLBACKS=7
CONSTANTS=262
ABI_FINDINGS=0

CANONICAL_ROUTES=4051
UNMAPPED_ROUTES=0

REFERENCE_TYPES=257
REFERENCE_MEMBERS=2964
EXPECTED_RUST_TYPES=259
ACTUAL_RUST_TYPES=259
TOTAL_DIAGNOSTICS=0
```

The selected Windows runtime profile is back at strict zero over the new
runtime boundary and the whole workspace test suite passes against the live
library. Evidence: [docs/abi-migration-evidence.md](docs/abi-migration-evidence.md)
and [docs/c-api-classification.md](docs/c-api-classification.md).

The Phase 1 upstream build blocker is closed: the unmodified canonical checkout
builds its C API, because ABI 0.20.0 is exactly the version that moved
`CNA_GRAPHICS_RENDERER_MAXIMUM` from 50 to 49.

## Do next

Work the backlog in [docs/backlog.md](docs/backlog.md). The highest-value ready
items, in order:

1. Re-measure the historical ABI-0.7 runtime blockers that the live ABI's own
   release notes say are closed: multi-listener `Apply3D` (0.9.0) and
   `VideoPlayer` frame identity (0.9.0). Neither may keep an
   `UPSTREAM_CNA_BLOCKED` row without a fresh measurement.
2. Implement the first modern `cna::extensions` family from `core_ext.h` and
   `graphics.h` capability reporting, which is safe, additive and outside the
   strict XNA hierarchy.
3. Close the 74-type `xna40-windows-full` gap: GamerServices, Avatar and Net.
4. Windows loader source support, packaging qualification, and the template's
   modern-extension canary.

## Toolchain reality on this host

```text
rustc/cargo 1.85.0 (source tarball)
rustfmt=NOT_AVAILABLE
clippy=NOT_AVAILABLE
rustup=NOT_AVAILABLE
MSRV 1.74 toolchain=NOT_INSTALLED -> MSRV_RUNTIME_NOT_RUN
```

Record those as `NOT_AVAILABLE`, never as passed.
