# Audio/XACT evidence

Status date: 2026-08-23

Status: strict complete. Audio has zero local diagnostics. The contract counts
below are the Audio milestone snapshot; Media was subsequently completed in
[media-video-evidence.md](media-video-evidence.md). Runtime limits are separate from structural completeness in
[runtime-capabilities.md](runtime-capabilities.md).

## Strict contract

```text
REFERENCE_TYPES=257
REFERENCE_MEMBERS=2964
EXPECTED_RUST_TYPES=259
ACTUAL_RUST_TYPES=235
TOTAL_DIAGNOSTICS=24
MISSING_TYPES=24
MISSING_MEMBERS=0

Audio=0
Media=24
```

Every mapping and public-safety category is zero. All nineteen selected Audio
types have zero local diagnostics:

```text
AudioCategory                 AudioChannels
AudioEmitter                  AudioEngine
AudioListener                 AudioStopOptions
Cue                           DynamicSoundEffectInstance
InstancePlayLimitException    Microphone
MicrophoneState               NoAudioHardwareException
NoMicrophoneConnectedException RendererDetail
SoundBank                     SoundEffect
SoundEffectInstance           SoundState
WaveBank
```

The three exception identities follow the established Rust mapping: public
marker/error values preserve XNA type identity, while ordinary operations
return `Result<T, CnaError>` rather than throwing CLR exceptions.

## XNA-managed behavior

Microsoft XNA 4.0 metadata, decompiled IL, and reference observations are the
behavioral authority. CNA-CS and CNA-Java supplied engineering evidence for
ownership and native gaps, but neither binding defines the Rust contract.

`SoundEffect.GetSampleDuration` and `GetSampleSizeInBytes` preserve XNA's
mixed binary32 arithmetic. This deliberately produces 88,198 bytes, not the
mathematical 88,200, for 44.1 kHz mono and one second. Mono/stereo, zero and
odd byte counts, invalid rates/channels/durations, rounding, and overflow are
covered by the platform-neutral corpus.

`AudioListener` and `AudioEmitter` retain exact mutable CLR reference-type
semantics while copying `Vector3` values at public boundaries. Emitter
`DopplerScale` rejects negative values but preserves XNA's NaN and signed-zero
behavior. `RendererDetail` implements XNA equality over display name and
renderer ID, XNA's UTF-16 string-hash XOR, and XNA `ToString`; CNA's ID-only
behavior is not exposed.

The final selected-profile corpus passes 215 named observations and 216
assertions, including the 20 deterministic Audio observations, with zero
failures. Backend state, hardware
enumeration, playback timing, callbacks, and malformed-bank behavior remain
native qualification rather than golden XNA behavior.

## SoundEffect and instances

`SoundEffect` owns one native effect handle. PCM16 construction and legal
deterministic mono/stereo WAV `FromStream` use canonical CNA routes; malformed
RIFF/fmt/data, unsupported encoding, truncation, failed creation, and rollback
are tested. Duration and Name remain cached where XNA permits after disposal.
The four process-static properties, Play overloads, CreateInstance, explicit
Dispose, Drop, repeated disposal, Game shutdown, and recreation all use real
native state.

`SoundEffectInstance` owns one native instance and strongly retains its parent
effect state. Volume, Pitch, Pan, IsLooped, State, Play/Pause/Resume, Stop,
and Apply3D reproduce XNA validation and failure timing. Cached scalar
properties remain readable after disposal where XNA does; transport/state
operations still require a live native handle. Parent-first, child-first,
multiple-child, double-dispose, wrong-thread refusal, and owner-thread retry
are stress tested without duplicate owners.

Single-listener Apply3D uses the canonical native route. The strict
multi-listener overload is present, but CNA ABI 0.7 rejects every listener
count except one. Rust returns `CnaError::UnsupportedRuntime` for multiple
listeners instead of selecting or averaging them.

## Dynamic audio and callbacks

`DynamicSoundEffectInstance` composes one `SoundEffectInstance` state and one
native owner. Constructor validation, full/ranged SubmitBuffer, PCM alignment,
empty/bad ranges, reusable caller buffers, PendingBufferCount, transport,
disposal, and queue drain are covered. CNA copies submitted PCM during the
call, so Rust does not retain caller slices.

The native BufferNeeded trampoline never invokes arbitrary user Rust code. It
catches at the FFI boundary, upgrades only a weak registration, and queues
work into the existing Game/FrameworkDispatcher owner-thread pump. Handler
snapshots preserve identity/order and support duplicates, self-removal,
reentrant submission, panic containment with later handlers, disposal,
shutdown, and recreation. No second Audio dispatcher exists and no stale
native callback owns a Rust object.

## Microphone

`Microphone.All` and `Default` use real CNA enumeration and stable per-Game
device facades; no synthetic default is inserted. The qualified environment
validly reports zero physical devices. Name, State, BufferDuration,
Start/Stop, GetData, static sample helpers, and BufferReady registration
lifetime use reviewed routes and exact validation. Enumeration/no-device
behavior and registration cleanup are native verified. Physical capture and
callback timing remain `HARDWARE_PENDING`; no samples are fabricated.

## XACT ownership and qualification

| Type | Classification | Retained dependency |
|---|---|---|
| AudioEngine | OWNED | Game generation |
| AudioCategory | PARENT_OWNED facade | AudioEngine |
| WaveBank | OWNED child | AudioEngine |
| SoundBank | OWNED child | AudioEngine |
| Cue | OWNED child | AudioEngine and SoundBank state |
| RendererDetail | managed value | copied strings |

AudioEngine validates an XGS header, owns the XACT root, exposes renderer
values, Update, category lookup, and Disposing. Category identity is stable
within an engine and category operations use real native routes. WaveBank and
SoundBank validate signatures, retain the engine, own exactly one native
child, and roll back failed construction. Cue exposes the complete selected
state, transport, variables, Apply3D, and disposal contract and retains CNA's
required dependencies.

A deterministic parser-level XGS fixture exercises engine/category lifecycle.
Malformed but correctly signed XWB/XSB inputs exercise native parsing,
ownership, and cleanup. CNA logs parse failure but still returns handles;
Rust safely disposes those real handles rather than fabricating an error. No
legally redistributable authored XGS/XWB/XSB playback fixture is known, so
successful authored banks, cue acquisition, and playback remain
`ASSET_PENDING`.

## Native ABI and stress

```text
reviewed functions                  431 -> 528
prototype type positions          1509 -> 1862
independent C/Rust measurements    936 -> 1004
layouts                              56 -> 61
callbacks                             5 -> 6
constants                           243 -> 253
missing symbols                               0
prototype/layout/callback mismatches          0
ABI                                      0x0700
```

Every added symbol is measured against canonical C headers for prototype,
pointer depth/constness, scalar and enum representation, callback ABI, and
export presence. No C++ ABI or Media symbol was added by the Audio milestone;
the final reviewed slice is recorded separately in the Media evidence.

Crash-isolated Audio stress covers at least 75 SoundEffect, 75 instance, 75
dynamic, 50 callback-delivery, 60 microphone-lifetime/enumeration, 21 engine,
and 60 malformed-bank cycles. It includes parent/child order, explicit Dispose
and Drop, repeated disposal, failed creation, callback panic,
self-removal/reentrancy, Game shutdown/recreation, wrong-thread release, and
owner-thread retry.

```text
NATIVE_CRASHES=0
OBSERVED_DOUBLE_FREE=0
OBSERVED_UAF=0
SANITIZER_STATUS=NOT_RUN
```

Crash absence is not allocator-level leak evidence.

## Qualified limits and blockers

| AFFECTED_XNA_API | CURRENT_CNA_ROUTE | CURRENT_RUST_BEHAVIOR | MISSING_CNA_SEMANTIC | OWNERSHIP_REQUIREMENT | THREAD/CALLBACK_REQUIREMENT | WHAT_WOULD_UNBLOCK_IT |
|---|---|---|---|---|---|---|
| Apply3D listeners[] | instance Apply3D | one native; multiple UnsupportedRuntime | true multi-listener mixing | retain instance/effect | owner thread | route mixing all listeners |
| AudioEngine renderer/look-ahead | engine create | exact validation and native call | CNA discards both values | engine sole owner | owner-thread create | CNA honoring both arguments |
| malformed XWB/XSB | bank create | preserve/dispose returned handles | parser failure not propagated | child retains engine | owner create/destroy | atomic parser failure result |
| microphone capture | capture routes | no device/samples fabricated | qualified host has no hardware | stable facade | queue worker callback to owner | physical-device qualification |
| authored XACT playback | bank/cue routes | validation/error/ownership only | no CNA defect asserted | full dependency graph | owner lifecycle | legal deterministic authored fixture |

The first three are `UPSTREAM_CNA_BLOCKED`, microphone capture is
`HARDWARE_PENDING`, and authored playback is `ASSET_PENDING`. The sandbox's
default PulseAudio route cannot wake its mainloop; native stress uses SDL's
dummy driver and classifies the default route `BACKEND_BLOCKED`. Linux x86-64
is measured; other platforms are `PLATFORM_PENDING`.

The later process-global Media registration architecture retains the exact
qualified CNA library generation across Game recreation. Requalification then
showed that SoundEffect's process-static values persist as XNA requires; the
earlier reset observation was an artifact of unloading/reloading the native
library, not a remaining CNA semantic blocker.

Canonical read-only CNA HEAD is
`1bb2145d99ed572dd4eb15009c34e2e5f410fcf0` and still fails its unmodified C
API build at `CnaCApiCoreExt.cpp:250` (`49 == 50`). Runtime evidence uses the
qualified exact ABI-0.7 artifact with SHA-256
`6dcefcadb7aa0233da98682bdbc343581a9f1e754a09c641078d1bef97afd7ca`.
