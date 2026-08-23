# CNA-Rust measured implementation plan

Status date: 2026-08-23

Strict target: XNA 4.0 Windows runtime Rust API projection

MSRV: Rust 1.74

## Governing constraints

- Preserve authoritative XNA identifiers and casing below
  `cna::Microsoft::Xna::Framework`.
- Keep private implementation modules idiomatic, the compatibility allowlist
  empty, and every structural/safety category at zero.
- Add complete ownership/dependency families with real managed or reviewed CNA
  behavior; never add signature-only placeholders.
- Treat the generated `typeScoreboard` as the only authoritative work queue.
- Keep CNA ABI 0.7 and the canonical C ABI as the native boundary. Do not bind
  the C++ ABI, accept ABI 0.8, or add unrelated symbols.

## Current strict measurement

| Measurement | Milestone baseline | Current |
|---|---:|---:|
| XNA reference types | 257 | 257 |
| XNA reference members | 2,964 | 2,964 |
| expected mapped Rust types | 259 | 259 |
| actual strict Rust types | 203 | 216 |
| total diagnostics | 56 | 43 |
| missing types | 56 | 43 |
| missing members | 0 | 0 |

Constructor, overload, property, event, base projection, trait, interface,
parameter, return, generic, generic-bound, ref/out, enum value, flags,
delegate, disposal, unexpected type/member, type-kind, internal-type leak,
raw-handle leak, public unsafe API, allowlist, and unmeasured-category counts
are all zero.

The 43 diagnostics are only whole missing types:

```text
Graphics           0
Framework/core     0
Input               0
Storage             0
GamerServices       0
Design              0
Audio              19
Media              24
```

## Completed milestone

- [x] Implemented complete XNA 4.0 Windows compressed-XNB framing: compressed
  flag/header, exact declared output, short and extended frame headers,
  persistent 64 KiB LZX state, exact termination, and full legal LZX block/
  Huffman decoding.
- [x] Kept the uncompressed pipeline unchanged and ran primitive, shared
  fixup, external-reference, rollback, cache/unload/reload, and complete Model
  graphs through compressed framing. Fourteen malformed framing cases fail
  deterministically before publication.
- [x] Regenerated and recorded the exact eleven-type queue in
  `docs/small-family-queue.md` before implementation.
- [x] Completed Framework/core:
  `GraphicsDeviceInformation`, `GraphicsDeviceManager`,
  `IGraphicsDeviceManager`, and `PreparingDeviceSettingsEventArgs`.
  The manager integrates with Game state/services and CNA's Game-owned device;
  it does not fabricate a second device.
- [x] Completed Input: `GestureSample`, `GestureType`, and `TouchPanel` through
  reviewed raw-touch/gesture/panel routes. HEADLESS legitimately reports no
  hardware and no gesture.
- [x] Completed Storage: `StorageDevice`, `StorageContainer`, and
  `StorageDeviceNotConnectedException`, including deterministic Begin/End
  mapping, one-shot/origin checks, CNA-only filesystem/stream routing,
  containment, events, and nested ownership.
- [x] Completed `GamerServicesComponent` as a real GameComponent lifecycle
  bridge without expanding into the separate Gamer/Guide/Avatar/network
  profile.
- [x] Regenerated the exact Design queue and completed `MathTypeConverter`
  plus all twelve concrete value converters as one dependency-coherent family.
  The formal projection uses a closed value/type vocabulary, explicit culture,
  immutable ordered properties, deterministic creation, and executable closed
  reconstruction descriptors; it publishes no fake CLR component model.
- [x] Verified the six supported component-string converters and the six that
  intentionally reject string input, including XNA Windows Single text,
  invariant/en-US/de-DE, malformed/wrong/null values, Matrix Translation
  asymmetry, nested value snapshots, and all constructor round trips.
- [x] Kept Design managed-only: the reviewed native ABI, ownership inventory,
  and template source are unchanged.
- [x] Rechecked repeated RunOneFrame/Tick and canonical CNA HEAD without
  weakening the milestone. Both external blockers remain documented below.

Focused evidence lives in `docs/lzx-xnb-evidence.md`,
`docs/framework-evidence.md`, `docs/input-touch-evidence.md`,
`docs/storage-evidence.md`, and `docs/design-evidence.md`.

## Ownership and callback model

`GraphicsDeviceManager` has one managed state per Game. It publishes the
manager/device services, owns one native manager plus six registrations only
while the Game runs, and releases them before native Game destruction. Native
event sender identity is the public manager. Event dispatch snapshots handlers
for safe self-removal, catches panic, records it, and returns it at a safe Rust
boundary. `RankDevices` is explicitly CNA/backend blocked because ABI 0.7 has
no candidate-ranking route.

Touch has no owned native handle. Every static panel access requires the
active callback-scoped `GameContext`, and touch/gesture data is copied into
managed snapshots.

Storage ownership nests `StorageDevice -> StorageContainer -> StorageStream`.
Streams retain containers and are closed before container disposal; containers
retain devices. Native `Disposing` is required synchronously and delivered
exactly once. Static `DeviceChanged` registration is verified, while an actual
OS-originated transition remains platform pending. Off-owner notification is
queued until the next owner-thread Storage boundary. Rust validates XNA path
containment before CNA because the qualified ABI-0.7 `RelativePath` helper
still accepts strings without enforcing every child-path containment rule.
Wrong-thread stream/container release is refused by CNA and preserves the
handle for a successful owner-thread retry.

`GamerServicesComponent` composes the existing `GameComponent`; it owns no
native GamerServices handle and fabricates no unavailable service.

Design is pure managed code over existing copied XNA values. The strict Design
namespace contains only its thirteen reference types; crate-root support
abstractions replace the observable TypeConverter vocabulary without exposing
`System.ComponentModel`, reflection objects, arbitrary `Any`, handles, or raw
pointers. Property order is immutable, creation performs explicit name lookup,
and nested values are snapshots.

## Measured evidence

| Measurement | Previous | Current |
|---|---:|---:|
| named XNA-derived observations | 145 | 185 |
| assertions including final count | 146 | 186 |
| behavior failures | 0 | 0 |
| reviewed ABI functions | 431 | 431 |
| prototype type positions | 1,509 | 1,509 |
| independent C/Rust ABI measurements | 936 | 936 |
| layouts / callbacks / constants | 56 / 5 / 243 | 56 / 5 / 243 |
| ABI mismatches | 0 | 0 |
| native game lifetimes with a created game | 209 | 209 |
| owned native child-handle constructions | 1,012 | 1,012 |
| native crashes / observed double-free or UAF | 0 / 0 | 0 / 0 |

The 40 new Design observations cover capability support, every property shape,
string culture and error behavior, creation, Matrix asymmetry, nested snapshot
semantics, constructor identities, descriptor execution, and deterministic
base/map failures. Storage filesystem, hardware Touch, and native lifecycle
transitions stay in native qualification rather than being mislabeled as
neutral golden data.

The native handle total is derived from explicit constructors: each of ten
small-family cycles adds one manager, six manager registrations, one storage
device, one container, one container registration, and one stream (110); the
process-wide DeviceChanged registration adds one; the isolated callback-panic
case adds one manager plus six registrations and one selector device (8).
Thus `893 + 110 + 1 + 8 = 1,012`. Design creates no handles, so the suite
remains at 209 created Game lifetimes and 1,012 child constructions. Sanitizer
status remains `not-run`; crash absence is not a leak proof.

## External blockers

Canonical read-only CNA HEAD remains
`1bb2145d99ed572dd4eb15009c34e2e5f410fcf0`. Its tracked source still contains
the C API build blocker at `CnaCApiCoreExt.cpp:250`, where the renderer identity
assertion reduces to `49 == 50`. Runtime evidence therefore continues to use
the labelled qualified exact ABI-0.7 HEADLESS artifact. The Rust loader rejects
ABI 0.8.

Arbitrary repeated borrowed-game `RunOneFrame`/`Tick` remains blocked. CNA
retains `CNA_GameCallbacks::context` from creation; the later frame-hook table
has a separate mutable context but does not rebind Update/Draw/content callback
context. The minimal safe ABI addition is an owner-thread-only operation that
atomically replaces the complete core callback table/context while the Game is
idle, with an explicit guarantee that the prior context is no longer retained.
No `transmute`, fake `'static`, leaked allocation, mutable global raw pointer,
or unsupported mutation is used.

## Next dependency-ordered work

This milestone stops here. The only remaining selected families are separate
future milestones:

1. Audio (19)
2. Media (24)

Each requires its own regenerated native dependency/ownership/callback review.
Do not reopen completed Graphics, Framework/core, Input, Storage,
GamerServices, Design, or LZX merely to begin one of them.

## Definition of complete compatibility

The selected profile is complete only when all mapped types and members and
every structural category reach zero; public-surface safety gates remain zero;
behavior is XNA-derived; native prototypes/layouts are compiler-verified;
ownership and sanitizer evidence pass; canonical CNA builds unmodified; and
every claimed platform has fresh runtime evidence.
