# CNA-Rust next work

## 2026-08-30 — live CNA ABI 0.20, wider XNA profiles, and seven extension families

The binding no longer targets the historical ABI 0.7 development baseline. It
is built and qualified against the live CNA development tree, its canonical C
API is fully accounted for, and CNA's own capabilities beyond XNA now have a
safe Rust surface.

```text
CNANEXT_HEAD=72262a33ed5ae7657024c7f1251338748a3feee5
SHARP_RUNTIMENEXT_HEAD=eebebd862121953538e3b84d43384d70a8a1728d

ABI_OLD=0.7.0 / 0x0700
ABI_NEW=0.20.0 / 0x1400
LIBRARY_SHA256=195924825a12290cdd2244fc845e119295de515cf27d1f6b31e1ecc84e93f05d
LIBRARY_EXPORTS=4051

REVIEWED_SYMBOLS_REMOVED=0
REVIEWED_SYMBOLS_ARITY_CHANGED=0
ABI_FUNCTIONS=886
PROTOTYPE_TYPE_POSITIONS=3019
C_RUST_MEASUREMENTS=1236
LAYOUTS=71
CALLBACKS=8
CONSTANTS=397
ABI_FINDINGS=0
UNAUDITED_DECLARATIONS=0

CANONICAL_ROUTES=4051
RUST_SYS_BOUND=886
CNA_EXTENSION_BACKING=1770
STRICT_XNA_BACKING=626
MANAGED_BY_DESIGN=598
UPSTREAM_NOT_USEFUL_TO_RUST=126
TOOLING_ONLY=42
PLATFORM_ONLY=3
DEFERRED_RUNTIME=0
UNMAPPED_ROUTES=0

REFERENCE_TYPES=257
REFERENCE_MEMBERS=2964
EXPECTED_RUST_TYPES=259
ACTUAL_RUST_TYPES=293
OUT_OF_PROFILE_RUST_TYPES=34
TOTAL_DIAGNOSTICS=0
LEAK_DIAGNOSTICS=0

PROFILE_FULL_MISSING_TYPES=40
PROFILE_PIPELINE_MISSING_TYPES=125
PROFILE_SUPERSET_MISSING_TYPES=165
```

The selected Windows runtime profile is at strict zero over the new runtime
boundary; the 34 extra Rust types belong to the wider profile and are measured
as such rather than as inventions. Evidence:
[docs/abi-migration-evidence.md](docs/abi-migration-evidence.md),
[docs/c-api-classification.md](docs/c-api-classification.md),
[docs/extensions.md](docs/extensions.md) and
[docs/platform-evidence.md](docs/platform-evidence.md).

Three historical `UPSTREAM_CNA_BLOCKED` rows are closed by measurement rather
than assertion: the unmodified canonical C API build, multi-listener `Apply3D`,
and `VideoPlayer` frame identity. Three were re-measured and stand:
`AudioEngine` renderer/look-ahead (now stated upstream as a one-backend fact),
`GraphicsDeviceManager.RankDevices`, and repeated ticking of one live `Game`.

## Do next

Work the backlog in [docs/backlog.md](docs/backlog.md). The highest-value ready
items, in order:

1. Close the remaining 40-type `xna40-windows-full` gap: the GamerServices,
   Avatar and Net object model over `gamer_services.h` and `net*.h`. Its value
   types and exception identities are done; what remains needs native backing.
2. Continue `cna::extensions`: `.cnb` Model and the loader registry, then
   haptics, text input, sensors and the engine layer.
3. Decide whether the design-time Content Pipeline profile belongs in this
   crate at all, or in a separate one. 125 types either way.
4. A static-linkage mode, which is the prerequisite for any WebAssembly route.

## Toolchain reality on this host

```text
rustc/cargo 1.85.0 (source tarball)
rustfmt=NOT_AVAILABLE
clippy=NOT_AVAILABLE
rustup=NOT_AVAILABLE
MSRV 1.74 toolchain=NOT_INSTALLED -> MSRV_RUNTIME_NOT_RUN
MSRV source denylist=PASS (tools/msrv/audit.py)
```

Record those as `NOT_AVAILABLE`, never as passed.
