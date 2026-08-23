# Session evidence and next work

## 2026-08-23 — LZX and all small families complete

### Exact strict handoff

```text
                                      BEFORE -> AFTER
REFERENCE_TYPES                           257 -> 257
REFERENCE_MEMBERS                        2964 -> 2964
EXPECTED_RUST_TYPES                       259 -> 259
ACTUAL_RUST_TYPES                         192 -> 203
TOTAL_DIAGNOSTICS                          67 -> 56
MISSING_TYPES                              67 -> 56
MISSING_MEMBERS                             0 -> 0

CONSTRUCTOR_MAPPING_MISMATCH                0
OVERLOAD_MAPPING_MISMATCH                   0
PROPERTY_MAPPING_MISMATCH                   0
EVENT_MAPPING_MISMATCH                      0
BASE_PROJECTION_MISMATCH                    0
TRAIT_MISMATCH                              0
INTERFACE_MISMATCH                          0
PARAMETER_MISMATCH                          0
RETURN_TYPE_MISMATCH                        0
GENERIC_MISMATCH                            0
GENERIC_BOUND_MISMATCH                      0
REF_OUT_MISMATCH                            0
ENUM_VALUE_MISMATCH                         0
FLAGS_MISMATCH                              0
DELEGATE_MISMATCH                           0
DISPOSAL_MISMATCH                           0
UNEXPECTED_TYPES                            0
UNEXPECTED_MEMBERS                          0
TYPE_KIND_MISMATCH                          0
INTERNAL_TYPE_LEAK                          0
RAW_HANDLE_LEAK                             0
PUBLIC_UNSAFE_API                           0
ALLOWLIST                                   0
UNMEASURED_CATEGORIES                       0
```

Normal strict mode exits 1 only for the 56 genuine whole missing types.
Report-only generation records the same result; leak-only exits 0 with zero
findings. Verifier self-tests pass 23/23.

```text
Graphics             0 -> 0
Framework/core       4 -> 0
Input                3 -> 0
Storage              3 -> 0
GamerServices        1 -> 0
Design              13 -> 13
Audio               19 -> 19
Media               24 -> 24
```

The authoritative pre-implementation queue is preserved in
`docs/small-family-queue.md`.

### LZX compressed XNB

Status is `MANAGED_COMPLETE` for XNA 4.0 Windows XNB v5 framing.

```text
compressed flag/header                         PASS
declared decompressed size exact               PASS
single extended frame                          PASS
multi-frame persistent decoder                 PASS
32 KiB short-header frame                      PASS
exact termination / zero marker-padding        PASS
focused negative cases                           14
independent legal fixture output sizes       16561, 44032 bytes exact
```

One stateful 64 KiB LZX decoder spans the asset. It implements verbatim,
aligned, and uncompressed blocks, all XNA-used Huffman tables, repeated
offsets, and window wrap. It rejects the optional CAB Intel transform. The
frame layer rejects truncated short/extended headers and blocks, zero/invalid
sizes, oversized frames, decoder failures, short/long output, malformed
trailing data, compressed-header truncation, and file-size mismatch.

Compressed ordinary reader graphs cover primitive/value data, shared-resource
fixups/identity, compressed external references, reader failure and partial
rollback, cache identity, `Unload`, and reload. A complete compressed Model
with shared buffers/effect ran through load/draw/unload/reload ownership for
ten crash-isolated native cycles. The uncompressed path remains green and
unchanged. No Microsoft-owned XNB is committed; optional independent fixtures
were read-only evidence.

See `docs/lzx-xnb-evidence.md`.

### Framework/core

Exact types:

- `GraphicsDeviceInformation`
- `GraphicsDeviceManager`
- `IGraphicsDeviceManager`
- `PreparingDeviceSettingsEventArgs`

All have zero local strict diagnostics. Device information and event args are
managed shared-reference graphs; explicit XNA Clone deep-copies presentation
state. The manager registers once per Game, publishes both selected services,
retains the existing Game-owned `GraphicsDevice`, synchronizes all selected
preferences, and owns/disposes its CNA manager and six callback registrations
inside Game lifetime.

HEADLESS natively verified attachment, preference propagation,
`ApplyChanges`, mutable `PreparingDeviceSettings`, public event-sender identity,
exactly-once disposal, callback self-removal, panic containment, cleanup, and
Game recreation. Native reset/resetting did not originate on HEADLESS;
protected managed dispatch was tested separately. CNA ABI 0.7 has no
device-candidate ranking route, so `RankDevices` returns the precise
`UnsupportedRuntime` blocker. No second device or fake state change exists.

See `docs/framework-evidence.md`.

### Input Touch/Gesture

Exact types: `GestureSample`, `GestureType`, and `TouchPanel`. All have zero
local strict diagnostics.

Reviewed native routes cover capabilities/state snapshots, enabled gestures,
gesture availability/read, display dimensions/orientation, and window handle.
Timestamp/deltas and touch snapshots are copied exactly; undefined flags or
state are rejected. Qualified HEADLESS status is no connected touch hardware,
zero touches, no queued gesture, successful enabled-gesture round trip, and a
real native error from `ReadGesture`. Hardware touch and recognized gestures
remain platform pending; no recognizer or hardware is fabricated.

See `docs/input-touch-evidence.md`.

### Storage

Exact types: `StorageDevice`, `StorageContainer`, and
`StorageDeviceNotConnectedException`. All have zero local strict diagnostics.

The documented Rust async result completes synchronously, retains state,
orders callback before Begin returns, validates operation/device origin, and
permits End once. Foreign and repeated End fail. Callback panic becomes
`CnaError::Callback`; no CLR async machinery is fabricated.

All selectors, containers, filesystem operations, enumeration, and streams use
canonical CNA Storage functions. The qualified ABI-0.7 `RelativePath` helper
copies UTF-8 without enforcing every child containment rule, so Rust rejects
absolute/drive/UNC paths and escaping traversal before native dispatch while
allowing valid normalized nested relative paths and single-component wildcard
patterns.

Ownership is `device -> container -> stream`; streams retain their container,
containers retain their device, and Dispose closes streams first. Explicit/
double dispose, device-before-container, live stream shutdown, synchronous
exactly-once `Disposing`, disposing-handler panic cleanup, and deletion are
verified. Wrong-thread stream/container release is refused without losing the
handle, and owner-thread retry succeeds. Native static `DeviceChanged`
subscription/unsubscription/stale-removal is verified; no OS-originated
transition occurred on the platform. A deterministic callback-shape test proves
that an off-owner notification queues until an owner-thread Storage boundary
and that handler panic is deferred as `CnaError::Callback`.

See `docs/storage-evidence.md`.

### GamerServices

The exact type is `GamerServicesComponent`; it has zero local strict
diagnostics. It composes `GameComponent` and participates in Game association,
Initialize, Update, Enabled/UpdateOrder events, disposal, and component
lifecycle. It owns no native GamerServices resource. The profile deliberately
does not add Gamer, SignedInGamer, Guide, Avatar, networking, achievements, or
leaderboards.

### ABI

```text
reviewed functions                    347 -> 431
prototype type positions             1220 -> 1509
independent C/Rust measurements       840 -> 936
layouts                                51 -> 56
callbacks                               3 -> 5
constants                             206 -> 243
prototype/layout/callback mismatches          0
missing header/library symbols                0
header exports / library exports       2861 / 2861
ABI version                           0x0700 / 1792
```

The additions are only the Framework manager, Touch/Gesture, and Storage
routes needed by this milestone. GamerServicesComponent is managed-only.
Compiler-backed prototypes, C/Rust layouts, constants, callbacks, exact loaded
symbols, and ABI version all pass.

Qualified runtime artifact:

```text
/tmp/cna-rust-native-070/modules/c-api/libcna_c_api.so
size:   16889928 bytes
sha256: 6dcefcadb7aa0233da98682bdbc343581a9f1e754a09c641078d1bef97afd7ca
```

### Behavior and ownership

```text
XNA-derived observations                  140 -> 145
assertions including final count           141 -> 146
failures                                          0

native Game lifetimes                      197 -> 209
owned native child-handle constructions    893 -> 1012
small-family combined cycles                        10
compressed Model cycles                            10
native crashes                                      0
observed double-free/UAF                            0
sanitizer status                               not-run
```

New platform-neutral observations cover Gesture flag/sample structures and
GraphicsDeviceInformation defaults, shared identity, and explicit deep-clone
semantics. Storage filesystem behavior and hardware/native transitions remain
native/platform evidence.

The 119 added child handles are explicit: ten cycles each construct a manager,
six registrations, device, container, disposing registration, and stream
(110); one process-wide DeviceChanged registration; and the isolated callback
case's manager/six registrations/selector device (8). The full stress suite
passes all isolated children, including callback panic and Game recreation.
No allocator leak claim is made without sanitizers.

### CNA and repeated frame blocker

Canonical CNA HEAD remains:

```text
1bb2145d99ed572dd4eb15009c34e2e5f410fcf0
```

Tracked source is unchanged; the checkout contains one untracked CMake test
discovery JSON. The prior unmodified C API build remains blocked at
`CnaCApiCoreExt.cpp:250`, renderer identity `49 == 50`; that exact assertion
and HEAD were rechecked read-only. CNA was not modified.

Arbitrary repeated borrowed-game RunOneFrame/Tick remains blocked. The
creation-time `CNA_GameCallbacks::context` is retained; the mutable frame-hook
context is separate and cannot rebind Update/Draw/content callbacks. The
minimal CNA ABI requirement is an idle, owner-thread-only atomic replacement
of the full core callback table/context with a guarantee that the old context
is no longer retained. No unsafe lifetime workaround was added.

### Template and consumer

```text
template source changed                          no
template git HEAD          86612449a2414663f0e17dac98c1bd5239712559
template tests                                  PASS
template native smoke                     60 / PASS
template native stability                600 / PASS
fresh vendored consumer workspace tests        PASS
fresh consumer native smoke               60 / PASS
developer/sibling absolute-path findings         0
symlinks in generated consumer                    0
```

The sibling template checkout was read-only to this process, so Cargo output
was directed to `/tmp`; its source and Git status remained unchanged. The
generated consumer used only `vendor/cna` and `vendor/cna-sys`; the relative
path between those two vendored crates is intentional and self-contained.

### Final gates

Passed:

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo clippy --workspace --all-targets --all-features` (existing audited
  compatibility warnings, exit 0)
- `cargo test --workspace --all-features`
- `cargo doc --workspace --no-deps`
- verifier self-tests, report-only, expected-failing normal strict, leak-only
- XNA behavior corpus
- complete native ABI verifier
- full crash-isolated native stress
- `git diff --check`
- template tests, 60/600 native runs, fresh vendored tests/smoke, and path/
  symlink audits

### Remaining work

Only these selected families remain, each as a separate milestone:

```text
Design   13
Audio    19
Media    24
```

Do not reopen completed Graphics, Framework/core, Input, Storage,
GamerServices, or LZX when starting the next queue. Regenerate the scoreboard
and perform a fresh dependency/ownership review for the chosen family.
