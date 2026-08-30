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
ABI_FUNCTIONS=793
PROTOTYPE_TYPE_POSITIONS=2680
C_RUST_MEASUREMENTS=1129
LAYOUTS=65
CALLBACKS=8
CONSTANTS=340
ABI_FINDINGS=0

CANONICAL_ROUTES=4051
UNMAPPED_ROUTES=0

REFERENCE_TYPES=257
REFERENCE_MEMBERS=2964
EXPECTED_RUST_TYPES=259
ACTUAL_RUST_TYPES=259
TOTAL_DIAGNOSTICS=0
```

The first modern `cna::extensions` family is complete: CNA's process-level
runtime identity and renderer selection, 35 canonical routes with a native
regression test and an opt-in template canary. See
[docs/extensions.md](docs/extensions.md).

Two historical `UPSTREAM_CNA_BLOCKED` rows are closed by measurement rather
than by assertion: multi-listener `Apply3D` now reaches its canonical route,
and `VideoPlayer.GetTexture` wraps a decoded frame in a borrowed `Texture2D`
using the frame identity ABI 0.9.0 added.

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

1. Continue `cna::extensions`: the `.cnb` content vertical slice, then the
   device layer and modern input.
2. Close the remaining 43-type `xna40-windows-full` gap: the GamerServices,
   Avatar and Net object model over `gamer_services.h` and `net*.h`.
3. Windows loader source support, packaging qualification, and the template's
   modern-extension canary.
4. Re-measure the remaining historical blockers: `AudioEngine` renderer and
   look-ahead, `GraphicsDeviceManager.RankDevices`, and repeated `Game` frame
   callback-context rebinding.

## Toolchain reality on this host

```text
rustc/cargo 1.85.0 (source tarball)
rustfmt=NOT_AVAILABLE
clippy=NOT_AVAILABLE
rustup=NOT_AVAILABLE
MSRV 1.74 toolchain=NOT_INSTALLED -> MSRV_RUNTIME_NOT_RUN
```

Record those as `NOT_AVAILABLE`, never as passed.
