# CNA-Rust next work

## 2026-08-31 — the complete XNA 4.0 Windows runtime profile is closed

All ten retained Microsoft runtime assemblies are now projected at strict zero.
The selected seven-assembly profile remains the hard gate and is unchanged; the
only structural gap left in the whole retained corpus is the design-time
Content Pipeline.

```text
CNANEXT_HEAD=17b5a90a0878f3f44c23bc8e3197d5d30373dc72 (dirty: another agent's WIP)
SHARP_RUNTIMENEXT_HEAD=4a49afb0cfe6a41e6e0af0bb62dc5175976731bb

ABI=0.20.0 / 0x1400
LIBRARY_SHA256=092b2d80a775f39a6ad872d084bc09492576c82ac33641faeb4a3036c7fc347b
LIBRARY_EXPORTS=4051
HEADER_EXPORTS=4054            # the live headers moved during the milestone

ABI_FUNCTIONS=1326             # was 886
PROTOTYPE_TYPE_POSITIONS=4574  # was 3019
C_RUST_MEASUREMENTS=1845       # was 1236
LAYOUTS=98                     # was 71
CALLBACKS=19                   # was 8
CONSTANTS=665                  # was 397
SYMBOL_ACQUISITIONS=1119       # new gate
SYMBOL_TYPE_MISMATCHES=0
ABI_FINDINGS=0
UNAUDITED_DECLARATIONS=0

CANONICAL_ROUTES=4054
RUST_SYS_BOUND=1326
CNA_EXTENSION_BACKING=1705
STRICT_XNA_BACKING=275
MANAGED_BY_DESIGN=577
UPSTREAM_NOT_USEFUL_TO_RUST=126
TOOLING_ONLY=42
PLATFORM_ONLY=3
DEFERRED_RUNTIME=0
UNMAPPED_ROUTES=0

PROFILE_SELECTED_DIAGNOSTICS=0
PROFILE_FULL_MISSING_TYPES=0   # was 40
PROFILE_FULL_DIAGNOSTICS=0
PROFILE_PIPELINE_MISSING_TYPES=125
PROFILE_SUPERSET_MISSING_TYPES=125  # was 165
LEAK_DIAGNOSTICS=0

WORKSPACE_TEST_SUITES=45
WORKSPACE_TEST_ASSERTIONS=115
```

Evidence: [docs/graphics-evidence.md](docs/graphics-evidence.md) for the ABI
0.20 capability gaps the version migration left refused,
[docs/backlog.md](docs/backlog.md) for the per-item status, and the three
native suites `gamer_services_native`, `net_native` and `native_stress`.

### What the runtime taught this milestone

Several behaviours were measured rather than assumed, and each is asserted by a
test that would fail if the projection faked it:

- `AvatarDescription.CreateRandom` randomizes nothing -- real XNA answers an
  all-zero, invalid description, and the body-type overload validates its
  argument and then ignores it.
- `AvatarRenderer.BindPose` raises unless the renderer reached `Ready`, which
  nothing in this runtime ever sets.
- `NetworkSession.Create` needs a signed-in gamer, because it makes the first
  one the host; `StartGame`/`EndGame` are queued and land on `Update`; and
  `EndGame` returns the session to `Lobby`, not `Ended`.
- `Guide.IsScreenSaverEnabled` belongs to the platform's display layer, which a
  headless host does not have: CNA answers `true` and its setter changes
  nothing.
- `Guide.EndShowMessageBox` on an unanswered box is CNA's state error, not a
  fabricated button press.

Three new upstream blockers were found and recorded rather than worked around:
`NetworkGamer` cannot answer the `Gamer` members it inherits, a
`LocalNetworkGamer` cannot answer its signed-in gamer, and a genuine second
machine needs a second host.

Two upstream blockers were re-measured on cnanext `17b5a90a` and still stand:
`GraphicsDeviceManager.RankDevices` has no candidate-ranking route, and
`CNA_GameCallbacks` is still copied at create with no context-rebind route.

## Do next

Work the backlog in [docs/backlog.md](docs/backlog.md). The highest-value ready
items, in order:

1. `RUST-ABI-008`: `cna_graphics_device_create` exists at ABI 0.20, so XNA's
   public `GraphicsDevice` constructor no longer has to refuse. It is a real
   ownership design -- an independently owned device beside the Game-owned one
   -- rather than a message fix.
2. `RUST-EXT-013`: `.cnb` Model and the loader registry, then sprite font and
   sound effect.
3. `RUST-PLATFORM-004`: a direct/static linkage mode, the prerequisite for any
   WebAssembly route. The new `SYMBOL_TYPE_MISMATCH` gate and the generated
   `GamerServicesApi`/`NetApi` tables are the shape to generalise: one canonical
   list, two acquisition modes.
4. `RUST-EXT-014`, `RUST-EXT-009`, `RUST-EXT-005`, `RUST-EXT-010`: haptics and
   text input, sensors, PBR and render-pipeline settings, and one coherent
   engine-layer vertical slice.
5. `RUST-BEHAVIOR-007`: re-measure media catalogs, picture tokens and
   `SavePicture` against the live ABI.
6. `RUST-XNA-004`: decide whether the design-time Content Pipeline belongs in
   this crate at all. 125 types either way.

## Toolchain reality on this host

```text
rustc/cargo 1.85.0 (source tarball)
rustfmt=NOT_AVAILABLE
clippy=NOT_AVAILABLE
rustup=NOT_AVAILABLE
MSRV 1.74 toolchain=NOT_INSTALLED -> MSRV_RUNTIME_NOT_RUN
MSRV source denylist=PASS (tools/msrv/audit.py)
sanitizers=NOT_RUN
```

Record those as `NOT_AVAILABLE`, never as passed.
