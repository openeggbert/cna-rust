# CNA-Rust next work

## 2026-08-31 — ABI 0.21, owned devices, CNB content, and two linkage modes

The previous milestone closed the complete XNA 4.0 runtime profile. This one
kept that at strict zero and moved everything else: the binding now admits live
ABI 0.21, constructs graphics devices of its own, reads and writes CNA's
compiled content format, and fills its function tables either through the
platform loader or through symbols the linker resolved.

```text
CNANEXT_HEAD=599d14e54e073b566d77b3d6fb30ac52d3d810b7 (clean)
SHARP_RUNTIMENEXT_HEAD=4a49afb0cfe6a41e6e0af0bb62dc5175976731bb (clean)

ABI=0.21.0 / 0x1500          # was 0.20.0
LIBRARY_SHA256=3a976d2494580ca9af45fbb2be30c13b01d05477f98ae80796ef26898c97d812
LIBRARY_EXPORTS=4054
HEADER_EXPORTS=4054          # artifact and headers now agree exactly
ENGINE_LAYER_VERSION=2

ABI_FUNCTIONS=1591           # was 1326
PROTOTYPE_TYPE_POSITIONS=5488  # was 4574
C_RUST_MEASUREMENTS=2272     # was 1845
LAYOUTS=121                  # was 98
LAYOUT_FIELD_SETS_CHECKED=121  # new gate
CALLBACKS=23                 # was 19
CONSTANTS=790                # was 665
SYMBOL_ACQUISITIONS=1587     # was 1119; +203 the gate had never seen
LINKED_DECLARATIONS=1591     # new: the direct-link mode's typed externs
ABI_FINDINGS=0
UNAUDITED_DECLARATIONS=0

CANONICAL_ROUTES=4054
RUST_SYS_BOUND=1591          # was 1326
CNA_EXTENSION_BACKING=1452
STRICT_XNA_BACKING=271
MANAGED_BY_DESIGN=577
UPSTREAM_NOT_USEFUL_TO_RUST=118  # was 126; eight were misclassified
TOOLING_ONLY=42
PLATFORM_ONLY=3
DEFERRED_RUNTIME=0
UNMAPPED_ROUTES=0

PROFILE_SELECTED_DIAGNOSTICS=0
PROFILE_FULL_MISSING_TYPES=0
PROFILE_FULL_DIAGNOSTICS=0
PROFILE_PIPELINE_MISSING_TYPES=125   # stated product boundary, not a backlog
PROFILE_SUPERSET_MISSING_TYPES=125
LEAK_DIAGNOSTICS=0

WORKSPACE_TEST_SUITES=51
WORKSPACE_TEST_ASSERTIONS=156
```

### What this milestone found

Four defects that had already shipped, each found by writing a test that
asserted a value rather than a success code:

- **`CNA_CnbReadLimits` was missing a field.** Six declared bounds against
  seven in C; padding hid it exactly, so `sizeof`, alignment and every declared
  offset agreed. The layout gate could not have caught it, so the verifier now
  asks Clang for each structure's real field list — 121 checked.
- **203 media routes were outside the symbol gate entirely.** They were `usize`
  slots each call site transmuted, so `SYMBOL_TYPE_MISMATCH` had no declared
  alias to check. Typed now; all 203 proved correctly paired.
- **The symbol gate then went blind.** Moving call sites to identifiers broke
  its regex, and it reported zero acquisitions and zero mismatches — a clean
  pass. There is a floor under the scan now.
- **Tightening one `.cnb` read bound zeroed the others,** because `None` was
  sent as `0` and CNA reads `0` as a literal limit.

And one in the tooling: the project generator dropped `cna-sys`'s new build
script, which the generated-project canary caught.

### Two decisions, both recorded with evidence

- [`docs/content-pipeline-decision.md`](docs/content-pipeline-decision.md) —
  the 125 design-time types are **out of scope**. Seventeen cannot be projected
  faithfully at all, and CNA's own `.cnj`/`.cnb` tooling already does the job,
  through routes this binding now uses.
- [`docs/engine-layer-scope.md`](docs/engine-layer-scope.md) — the engine layer
  is bound one slice at a time, and a slice qualifies when its semantics can be
  asserted exactly. 808 routes remain, blocked on a GPU-backed artifact rather
  than on anything here.

## Do next

Everything locally actionable is done: `docs/backlog.md` has no `READY` row.
What remains is blocked, and each row says on what.

1. **A GPU-backed qualified artifact** is the single highest-value unblock. It
   turns most of the 224 engine-layer families from "constructs" into
   "constructs and can be asserted", and it is what
   `docs/engine-layer-scope.md` names as the trigger.
2. **A wasm Rust target.** The binding's side is done — `direct-link` works and
   is verified — so `RUST-PLATFORM-003` now waits only on a toolchain this host
   does not have and that this session may not install.
3. **Re-measure the three standing upstream blockers** when cnanext moves.
   `RUST-BEHAVIOR-011` was one of four and is now fixed; the others are
   `runtime.h`'s missing context rebind, `runtime_graphics_manager.h`'s missing
   ranking route, and `cna_gamer_*` on a network gamer handle.
4. **A second machine** for `RUST-BEHAVIOR-012`, and **real hardware** for the
   sensor, haptic and audio-backend rows.
