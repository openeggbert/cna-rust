# CNA-Rust plan

Status date: 2026-08-30

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
| `xna40-windows-full` | 10 | 331 | 3,640 | 43 |
| `xna40-windows-pipeline` | 7 | 128 | 743 | 125 |
| `xna40-windows-superset` (discovery) | 17 | 459 | 4,383 | 168 |

Every type the Rust projection already declares matches the superset's expected
contract exactly: the only diagnostic in any wider profile is `MISSING_TYPE`.
The complete runtime profile's value identities are done: 22 enums and 7
exception identities, exact managed Rust with no native backing because CLR
metadata is the whole of their contract. `PacketWriter` and `PacketReader` are done too: XNA derives them from
`BinaryWriter`/`BinaryReader` over a `MemoryStream`, and the projection owns
the buffer directly and reproduces XNA's byte order exactly, including the bit
reinterpretation its `Write(float)` override performs. What remains of that
profile's 43 types is its object model -- `Gamer`, `SignedInGamer`, `Guide`,
`AvatarRenderer`, `NetworkSession` and their collections -- which needs the
canonical `gamer_services.h` and `net*.h` routes behind it. The 125-type gap is
the design-time Content Pipeline.

Measurement is profile-scoped. A Rust type some other retained XNA assembly
declares belongs to a profile this run is not measuring and is not a
diagnostic; `UNEXPECTED_TYPE` therefore means what it should -- a type **no**
Microsoft XNA 4.0 assembly declares. That is what lets a wider profile be
implemented incrementally without the selected profile's strict gate reporting
the new types as inventions.

No Windows Phone or Xbox 360 reference assembly is present on this host, so
those profiles do not exist here rather than being claimed untested.

### 3b. Runtime and behaviour evidence

Structural zero is not runtime completion. Every historical ABI-0.7 blocker is
re-measured against the live ABI before it may be carried forward.

### 3c. Full canonical C API accounting

All 4,051 canonical routes carry exactly one classification and there are no
unexplained holes. See
[docs/c-api-classification.md](docs/c-api-classification.md).

| Category | Routes |
|---|---:|
| `RUST_SYS_BOUND` | 792 |
| `CNA_EXTENSION_BACKING` | 1,859 |
| `STRICT_XNA_BACKING` | 626 |
| `MANAGED_BY_DESIGN` | 598 |
| `UPSTREAM_NOT_USEFUL_TO_RUST` | 130 |
| `TOOLING_ONLY` | 42 |
| `PLATFORM_ONLY` | 3 |
| `DEFERRED_RUNTIME` | 1 |
| `INTERNAL_RUNTIME_ONLY` | 0 |
| `UNMAPPED_REQUIRES_REVIEW` | 0 |

### 3d. Modern CNA API under `cna::extensions`

1,859 canonical routes still back CNA concepts XNA 4.0 does not have. They are
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
shader dialect and CNA's own capability report.

## Ownership, threading and safety rules

Unchanged from Phase 1 and re-verified on the live ABI. Every native-backed
handle carries one measured ownership policy; `Drop` never replaces observable
`Dispose` semantics; no raw handle or public `unsafe` reaches the safe API;
no Rust panic unwinds through C.

## Backlog

`docs/backlog.md` holds the durable task list. Planning is not the deliverable:
a ready task is implemented, qualified and committed rather than described.
