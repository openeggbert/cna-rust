# CNA-Rust next work

## 2026-08-23 — selected-profile structural zero

The Microsoft XNA 4.0 Windows runtime Rust projection is structurally complete.
Normal strict and leak-only verification both exit zero at 259/259 types and
2,964 reference members. Every mapping/safety category, allowlist count, and
unmeasured-category count is zero.

```text
MEDIA_MILESTONE_COMPLETE=true
STRICT_ZERO=true
```

Media/Video completed the last 24-type dependency family. Its process-global
MediaPlayer architecture, native graph and collections, queue generation,
events, Song route, MediaLibrary/source lifecycle, visualization, Video,
VideoPlayer, and safe GetTexture boundary are documented in
[docs/media-video-evidence.md](docs/media-video-evidence.md).

## Do next

Stay in maintenance and qualification mode:

1. Reconcile CNA blockers without modifying CNA from this binding milestone:
   Video frame identity/generation, repeated Game callback-context rebinding,
   multi-listener Apply3D, and XACT constructor/parser semantics.
2. Qualify real platform media catalogs, picture/token/SavePicture providers,
   physical audio, authored XACT assets, and a legal deterministic video
   decoder fixture on supported backends.
3. Qualify Windows/macOS loaders and runtime behavior, packaging/release,
   docs.rs, notices, and real games.
4. Treat any new strict diagnostic as a regression. Reopen a frozen family
   only with concrete evidence.

Do not expand into wider GamerServices/Avatar, Net, Content Pipeline, Xbox, or
Windows Phone unless a future milestone explicitly selects that profile.

## Persistent evidence

```text
REFERENCE_TYPES=257
REFERENCE_MEMBERS=2964
EXPECTED_RUST_TYPES=259
ACTUAL_RUST_TYPES=259
TOTAL_DIAGNOSTICS=0
MISSING_TYPES=0
MISSING_MEMBERS=0

BEHAVIOR_OBSERVATIONS=215
BEHAVIOR_ASSERTIONS=216
BEHAVIOR_FAILURES=0

ABI_FUNCTIONS=730
PROTOTYPE_TYPE_POSITIONS=2492
C_RUST_MEASUREMENTS=1028
LAYOUTS=62
CALLBACKS=7
CONSTANTS=262
ABI_FINDINGS=0
ABI=0.7 / 0x0700

NATIVE_CRASHES=0
OBSERVED_DOUBLE_FREE=0
OBSERVED_UAF=0
SANITIZER_STATUS=NOT_RUN
```

Canonical read-only CNA HEAD:
`1bb2145d99ed572dd4eb15009c34e2e5f410fcf0`. The unmodified build blocker is
still `CnaCApiCoreExt.cpp:250` (`49 == 50`). Qualified artifact SHA-256:
`6dcefcadb7aa0233da98682bdbc343581a9f1e754a09c641078d1bef97afd7ca`.

The source-tarball toolchain remains Rust 1.85.0 without rustfmt or clippy;
record their status as `NOT_AVAILABLE`, not passed. The sibling template source
must remain unchanged and continues as the native/fresh-consumer canary.
