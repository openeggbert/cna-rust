# CNA-Rust plan

Status date: 2026-08-31

Scope: a Rust projection of Microsoft XNA Framework 4.0 over the canonical CNA
C ABI, plus a separately namespaced safe Rust API for CNA's own modern
capabilities. Microsoft XNA metadata/IL/reference behavior governs the public
XNA contract; canonical CNA headers govern the native ABI. Other CNA bindings
are engineering evidence, not contract authority.

## Governing constraints

- Bind only `cna_*` C ABI symbols; never the C++ ABI.
- Never hide a missing CNA semantic with fake Rust state, catalog data, audio,
  video frames, hardware or placeholder resources.
- Keep strict structural completeness distinct from runtime/backend/platform
  qualification.
- Never place a CNA-only concept inside `cna::Microsoft::Xna::Framework`.
- Do not reopen a completed family without a concrete regression.

## Phase 1 (complete) — the selected Windows runtime profile

The seven-assembly Microsoft XNA 4.0 Windows runtime profile reached strict
structural zero and remains a hard regression gate:

```text
REFERENCE_TYPES=257
REFERENCE_MEMBERS=2964
EXPECTED_RUST_TYPES=259
ACTUAL_RUST_TYPES=259
TOTAL_DIAGNOSTICS=0
MISSING_TYPES=0
MISSING_MEMBERS=0
ALLOWLIST=0
UNMEASURED_CATEGORIES=0
```

Every measured mapping and safety category is zero: constructor, overload,
property, event, base projection, trait, interface, parameter, return type,
generic, generic bound, ref/out, enum value, flags, delegate, disposal, type
kind, unexpected type/member, internal type leak, raw handle leak, and public
unsafe API. Normal strict and leak-only both exit zero.

Completed dependency families: Graphics, Content/XNB, LZX, Framework/core,
Input/Touch/Gesture, Storage, GamerServicesComponent, Design, Audio/XACT, and
Media/Video.

## Phase 2 (complete) — migration to the live CNA ABI

The binding moved from the historical ABI 0.7 development baseline to the live
CNA development tree at ABI 0.20.0. The reviewed slice survived thirteen minor
versions with zero prototype, layout, callback, constant or export differences;
what changed was the version gate, one unaudited declaration, and the fact that
the canonical surface outside the slice was unaccounted for. Full evidence is
in [docs/abi-migration-evidence.md](docs/abi-migration-evidence.md).

The upstream C API build blocker recorded by Phase 1 is closed: the unmodified
canonical checkout builds.

## Phase 3 (current) — beyond the selected profile

Phase 1's policy of staying inside the selected profile has been superseded.
The selected profile stays frozen at strict zero as a regression gate, and work
now proceeds on four fronts at once.

### 3a. The complete retained XNA 4.0 corpus

Seventeen legally retained original Microsoft assemblies are admitted by
SHA-256 and measured as explicit profiles rather than merged into one:

| Profile | Assemblies | Reference types | Reference members | Missing Rust types |
|---|---:|---:|---:|---:|
| `xna40-windows-runtime` (selected) | 7 | 257 | 2,964 | 0 |
| `xna40-windows-full` | 10 | 331 | 3,640 | 0 |
| `xna40-windows-pipeline` | 7 | 128 | 743 | 125 |
| `xna40-windows-superset` (discovery) | 17 | 459 | 4,383 | 125 |

**The complete Windows runtime profile is closed.** All ten retained runtime
assemblies are projected at strict zero: 333 expected Rust types present, and
zero in every measured category -- missing and unexpected type and member,
signature, parameter, generic and generic bound, ref/out, base projection,
trait, interface, enum and flags value, delegate, disposal, type kind, internal
type leak, raw handle leak and public unsafe API -- with an empty allowlist and
no unmeasured category. What closed it, in three slices:

- **GamerServices** (22 types): `Gamer`, `SignedInGamer`, `FriendGamer`, their
  collections and enumerator, `GamerProfile`, `GamerPresence`,
  `GamerPrivileges`, `GameDefaults`, `Achievement`, `AchievementCollection`,
  `PropertyDictionary`, `LeaderboardEntry`, `LeaderboardReader`,
  `LeaderboardWriter`, `Guide`, `GamerServicesDispatcher` and the three
  `EventArgs` identities.
- **Avatar** (4 types): `AvatarDescription`, `AvatarAnimation`,
  `IAvatarAnimation`, `AvatarRenderer`.
- **Net** (14 types): `NetworkSession`, `NetworkGamer`, `LocalNetworkGamer`,
  `NetworkMachine`, `AvailableNetworkSession`,
  `AvailableNetworkSessionCollection`, `QualityOfService` and the seven
  session `EventArgs`.

The 125-type gap that remains in the superset is exactly the design-time
Content Pipeline, which is a product-boundary question rather than a missing
projection.

Nothing in the family fabricates a service the host does not have. A headless
host reports an empty signed-in roster, no friends, no Guide screen and no
remote participant, and each of those is a measured answer rather than a
refusal or a placeholder. Where CNA cannot answer at all -- a network gamer's
inherited `Gamer` members, a local gamer's signed-in gamer -- the projection
reports CNA's refusal and the behaviour is recorded as `BLOCKED_UPSTREAM`.
Where only a second machine could supply the answer, it is `NO_LIVE_PEER`.

### 3b. Runtime and behaviour evidence

Structural zero is not runtime completion. Every historical ABI-0.7 blocker is
re-measured against the live ABI before it may be carried forward.

### 3c. Full canonical C API accounting

Every canonical route carries exactly one **purpose** and exactly one **binding
decision**, and there are no unexplained holes in either. The two are separate
questions -- what a route is for, and why Rust does or does not bind it -- and
conflating them is what once left 1,170 routes with no purpose at all. The live
canonical surface moves as CNA adds routes; the inventory measures the live
headers, so these totals move with them. See
[docs/c-api-classification.md](docs/c-api-classification.md).

Measured against cnanext `7712534d3`, 4055 canonical routes:

| Purpose | Routes |
|---|---:|
| `CNA_EXTENSION_BACKING` | 1,854 |
| `STRICT_XNA_BACKING` | 1,374 |
| `MANAGED_BY_DESIGN` | 648 |
| `UPSTREAM_NOT_USEFUL_TO_RUST` | 132 |
| `TOOLING_ONLY` | 43 |
| `PLATFORM_ONLY` | 4 |
| `DEFERRED_RUNTIME` | 0 |
| `INTERNAL_RUNTIME_ONLY` | 0 |
| `UNMAPPED_REQUIRES_REVIEW` | 0 |

| Binding decision | Routes |
|---|---:|
| `BOUND` | 3,236 |
| `DELIBERATE_NON_BINDING` | 804 |
| `BLOCKED_UPSTREAM` | 15 |
| `DEFERRED_TRACKED` | 0 |
| `ACTIONABLE_LOCAL` | 0 |
| `UNREVIEWED` | 0 |

Bound is not the same as reachable, and the census measures that separately:
of the 3,236 bound routes, 97 have no safe call site, every one of them
justified with a reason and one outcome from a closed set. The gate fails on an unexplained one.

### 3d. Modern CNA API under `cna::extensions`

1,854 canonical routes back CNA concepts XNA 4.0 does not have. They are
exposed as safe, idiomatic Rust under `cna::extensions`, never inside the
strict XNA hierarchy, and never as raw `cna_*` calls. See
[docs/extensions.md](docs/extensions.md).

Two families are complete. `runtime` covers CNA's process-level identity and
renderer selection in 35 routes: platform and desktop-OS identity, renderer
identity, availability, backend category and maturity, preferred selection and
its latch, the fallback chain and CNA's own fallback history. `logging` covers
the process log in 19 routes, including a Rust sink whose panic is contained at
the FFI boundary. `graphics` gained CNA's renderer capability reporting: 6
routes over per-feature support, numeric limits, per-format usage masks, the
shader dialect and CNA's own capability report. `devices` covers the device
layer in 16 routes -- power, system facts, locale, display and clipboard --
including the availability query that separates a compiled-out layer from a
missing device. `graphics` also carries CNA's extended effects: CRT,
depth and ASCII post-processing, 24 routes, all three created and exercised on
this host's HEADLESS build. `input` covers raw joystick
enumeration, capabilities and snapshot capture in 20 routes. `content` covers
the `.cnb` container's first complete vertical in 33 routes: build texture data, encode a document, parse one back,
read its metadata, and decode a texture, with bounded parsing of untrusted
input.

## Ownership, threading and safety rules

Unchanged from Phase 1 and re-verified on the live ABI. Every native-backed
handle carries one measured ownership policy; `Drop` never replaces observable
`Dispose` semantics; no raw handle or public `unsafe` reaches the safe API;
no Rust panic unwinds through C.

## Backlog

`docs/backlog.md` holds the durable task list. Planning is not the deliverable:
a ready task is implemented, qualified and committed rather than described.
