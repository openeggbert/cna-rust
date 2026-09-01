# CNA-Rust next work

## 2026-09-01 — the reachability milestone: every bound route says why

The previous milestone gave all 4,054 routes a binding decision. This one asks
the harder question the census had only ever *reported* on: of the routes Rust
binds, which can a consumer actually call, and for the rest, why not?

The number had been wrong four times -- 894, 1,077, 303 -- because all four
came from matching a C route's *name* against Rust identifiers, and the field
holding a route's pointer is not named after the route.
`cna_audio_category_pause` lives in `AudioApi::category_pause`.

```text
CNANEXT_HEAD=7712534d3d22c7e284714e0e87afebba3f3cb472
CNANEXT_AT_QUALIFICATION=7712534d3d22c7e284714e0e87afebba3f3cb472
                             # both artifacts rebuilt for this milestone
SHARP_RUNTIMENEXT_HEAD=9cc96cd57cde394940cc24d58743edf9bf63d3fb

ARTIFACT_ENGINE=cnanext/cmake-build-opengles3   # CNA_CNAEXT=ON, OPENGLES3
ARTIFACT_HEADLESS=cnanext/cmake-build-headless  # CNA_CNAEXT=OFF, HEADLESS
LIBRARY_SHA256=94078be94dc1f1e6c8787c1cd17b08c9430d1e4bb5699947cd2b7aafee40281d
LIBRARY_EXPORTS=4055
HEADER_EXPORTS=4055

CANONICAL_ROUTES=4055        # was 4054; CABI-49 added one
BOUND=3236                   # was 3232: +7 bound, -3 unbound as never-acquired
DELIBERATE_NON_BINDING=804   # was 801
BLOCKED_UPSTREAM=15
DEFERRED_TRACKED=0           # was 6: EXT-016, EXT-017 and EXT-018 all closed
UNREVIEWED=0
ACTIONABLE_LOCAL=0

RUST_SYS_DECLARATIONS=3251
SYMBOL_ACQUISITIONS=3250
LINKED_DECLARATIONS=3251
PROTOTYPE_MISMATCHES=0
SYMBOL_TYPE_MISMATCHES=0
LAYOUT_FIELD_SETS_CHECKED=187
C_RUST_MEASUREMENTS=3174
ABI_FINDINGS=0
UNAUDITED_DECLARATIONS=0

BOUND_WITHOUT_SAFE_CALL_SITE=97      # measured; was reported as 303
  JUSTIFIED=97
  UNJUSTIFIED=0                      # and now gated
  IMPLEMENTED_IN_SAFE_RUST=59
  OUTSIDE_XNA_SURFACE=37
  ATOMIC_TABLE_MEMBER=1
```

### What the walk measures

`tools/c-api-inventory/reachability.py` ties each route to its field through
its own `symbol!` acquisition -- all 3,250 field names are unique across the
crate -- and walks the call graph from every file outside `native/`, with no
hop limit. Measured: 2,400 routes named by the safe layer directly, 696 behind
one wrapper, 4 behind two. A two-hop rule still cannot see
`cna_error_get_last_info`.

The gate now fails on an *unexplained* dead route rather than on the count. Four
planted defects each fail it, including one that must *not*: deleting
`AudioCategory::Pause` leaves `category_pause` reachable through `Resume`,
`SetVolume` and `Stop`, and the gate is right to pass.

### The three deferrals, closed

- **`RUST-EXT-016`** `SpriteFont::adopt` reads its tables back out of the
  handle; `SoundEffect::adopt` is the state `FromAsset` already had. Two owned
  handles, one asset, and the font releases before its atlas because CNA
  requires it. Measured against MonoGame's own `Default.xnb` -- which is what
  found `ContentReader::ReadChar` reading UTF-16 where `BinaryReader`'s reads
  UTF-8.
- **`RUST-EXT-017`** XNA's `AudioEngine.Dispose` calls the **public** `Dispose`
  on every child, so each one's `Disposing` fires; this projection called the
  private teardown. Fixed, with CNA's own notification bound in the same commit
  as the cross-check.
- **`RUST-EXT-018`** one native subscription per dynamic buffer, delivering into
  the buffer's handler list in registration order. `EVENT_BRIDGE_VERIFIED`;
  `REAL_DEVICE_LOSS_VERIFIED` is **false** and the test says so.

### Two new upstream findings

- **`RUST-UPSTREAM-028`** — no route reports a queued packet's size, and the
  array receive truncates where XNA throws. The `PacketReader` overload also
  reports zero bytes always, which is FNA's `uint len = 0` preserved.
- **`RUST-UPSTREAM-029`** — CNA's `GamerServicesComponent` skips
  `base.Initialize()` and `base.Update()` to match FNA; XNA's IL calls both.

### One thing the documentation got wrong

README's `Verified status (2026-08-23)` said the strict verifier reports zero
unexpected types and members. It reports **110**, and has since the
`RUST-EXT-015d`/`015e`/`015q` milestones put CNA's own members on strict XNA
types. Nothing had re-run it. Corrected, and opened as `RUST-SURFACE-001`.

### The full handoff

`docs/handoff-2026-09-01-census.md` is the seventeen-section record.

### Do next

1. **`RUST-SURFACE-001`.** The one open question about the projection's shape:
   do CNA's own members belong on a strict XNA type, or behind an extension
   trait? 109 members, and the two answers give different public APIs. This is
   a decision, not a fix.
2. **The standing external blockers.** A wasm toolchain for
   `RUST-PLATFORM-003`; a macOS host for `RUST-PLATFORM-002`; a second machine
   for `RUST-BEHAVIOR-012`; a real audio backend for `RUST-BEHAVIOR-008`; a
   legally redistributable video fixture for `RUST-BEHAVIOR-009`.
3. **The ten `RUST-UPSTREAM-*` findings**, each with a reproducer that runs
   without this repository.
