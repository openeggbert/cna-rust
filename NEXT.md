# CNA-Rust next work

## 2026-09-01 — `RUST-SURFACE-001`: CNA's members leave the strict XNA types

The reachability milestone left one open question, and it was a product
decision rather than a bug: for three milestones CNA's own members had been
added to strict XNA types as ordinary inherent methods, and the first run of
the strict verifier after they landed reported **110 diagnostics** -- 109
`UNEXPECTED_MEMBER` and one `UNEXPECTED_TYPE`.

The answer taken here is the one `extensions/mod.rs` already claimed: a
CNA-only operation on an XNA object is an **extension-trait method**. The
strict hierarchy contains only what Microsoft XNA declares.

```text
CNANEXT_HEAD=7712534d3d22c7e284714e0e87afebba3f3cb472
SHARP_RUNTIMENEXT_HEAD=9cc96cd57cde394940cc24d58743edf9bf63d3fb

ARTIFACT_ENGINE=cnanext/cmake-build-opengles3   # CNA_CNAEXT=ON, OPENGLES3
ARTIFACT_HEADLESS=cnanext/cmake-build-headless  # CNA_CNAEXT=OFF, HEADLESS
LIBRARY_SHA256=94078be94dc1f1e6c8787c1cd17b08c9430d1e4bb5699947cd2b7aafee40281d

                          before   after
STRICT_SELECTED_TOTAL        110       0
STRICT_COMPLETE_TOTAL        110       0
UNEXPECTED_MEMBER            109       0
UNEXPECTED_TYPE                1       0
MISSING_MEMBER                 0       0
MISSING_TYPE                   0       0
ALLOWLIST                      0       0
UNMEASURED_CATEGORIES          0       0

EXTENSION_TRAITS_ADDED        30
MEMBERS_MOVED                109
MEMBERS_RENAMED                0
INHERENT_FORWARDERS_LEFT       0
VERIFIER_RULES_RELAXED         0

EXTENSION_SURFACE_GATE       283 members, 59 traits, 0 diagnostics
PUBLICLY_NAMEABLE_ITEMS      987
UNNAMEABLE_PUBLIC_TYPES        0    # was 1: PresentationMode

CANONICAL_ROUTES=4055
BOUND=3236
BOUND_WITHOUT_SAFE_CALL_SITE=97      # unchanged by the migration
  JUSTIFIED=97
  UNJUSTIFIED=0
UNREVIEWED=0
ACTIONABLE_LOCAL=0
```

### Why the file a method lives in never mattered

Twenty-eight of the 109 were already inside `crates/cna/src/extensions/`, in
`impl GraphicsDevice { pub fn ... }` blocks. An inherent `pub fn` is part of
`GraphicsDevice`'s public API wherever it is written. Moving source files would
have changed nothing; what changed is `impl Type` becoming
`impl Trait for Type`.

### What it cost a caller

An import line. A receiver method keeps its call exactly, and so does an
associated function: `Song::FromFile(game, name, path)?` still resolves,
because Rust searches the traits in scope for an associated item on a type as
well as its inherent impls. 27 of the 109 are associated functions and every
one keeps its shape. `tools/package-consumer` compiles them from outside the
workspace with the traits imported, and compiles the same file without them,
where the build must be refused with E0599.

The one thing that did change beyond the import: `from_native_value` on
eighteen graphics enums and `Keys::from_key_code` were `const fn`, and a trait
method cannot be `const` on stable Rust. The inherent conversions are still
`const` and still what the crate decodes with; what a consumer reaches through
the trait is not usable in a `const` context.

### A second gate, for the half the first one cannot see

The strict verifier reaches zero by *removing* CNA's members from the XNA
hierarchy, so on its own it cannot tell a member that moved behind a trait from
one that was deleted. `tools/extension-surface/verify.py` answers that: 283
CNA-only members reachable on strict XNA types -- the 109 moved here and 174
that were already extension traits and had no gate at all -- each still
declared by a publicly reachable trait with an unchanged signature, implemented
for its strict type, and absent from that type's inherent surface.

It measures one more thing, because this milestone found the defect: a public
signature that names a crate type no public path reaches.
`GraphicsDeviceManager::PreferredPresentationMode` answered with
`PresentationMode`, which was `pub` inside a private module and re-exported
nowhere, so a caller could invoke the method and not name what came back. The
same defect the previous milestone fixed for `DeviceSettingsObserver` and
`ObservedDeviceSettings`, and missed for the third type. Now zero across the
whole public API, and gated.

### Six planted defects

Each caught by the gate that should catch it, and by no other: a member left
inherent (strict verifier), the test backend re-exported into `Input::Touch`
(strict verifier), a member dropped from its trait with the body kept
(extension gate only -- the strict verifier reports zero), the
`PresentationMode` re-export removed again (extension gate only), the only
trait-method caller of a native route deleted (census gate, 97 -> 98
unreachable with one unjustified), and the trait imports removed from a
consumer (E0599).

The fifth answers the question the migration raised about the call-graph walk:
it finds a call site inside an `impl Trait for Type` body exactly as it found
one inside an inherent impl.

### One documentation defect

`SkinnedEffect::VertexColorEnabled` carried a comment saying XNA declares it
and the strict projection had missed it. The pinned
`Microsoft.Xna.Framework.Graphics.dll` gives it to `BasicEffect` and
`DualTextureEffect` and not to `SkinnedEffect`. It is CNA's own third, and the
comment says so where the member now lives.

### The full record

`docs/handoff-2026-09-01-surface.md` is the seventeen-section record.
`docs/extensions.md` carries the member-by-member migration table and the
before/after call for all 109.

### Do next

Nothing here is locally actionable. What remains is external:

1. **The standing external blockers.** A wasm toolchain for
   `RUST-PLATFORM-003`; a macOS host for `RUST-PLATFORM-002`; a second machine
   for `RUST-BEHAVIOR-012`; a real audio backend for `RUST-BEHAVIOR-008`; a
   legally redistributable video fixture for `RUST-BEHAVIOR-009`.
2. **The ten `RUST-UPSTREAM-*` findings**, each with a reproducer that runs
   without this repository.
3. **`RUST-XNA-004`**, the design-time Content Pipeline: a product-boundary
   decision, not a missing projection.
