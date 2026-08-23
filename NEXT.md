# Session evidence and next work

## 2026-08-23 — Audio/XACT complete; Media only remains

Audio was completed as one native ownership/callback subsystem. Do not reopen
Graphics, Framework/core, Input, Storage, GamerServices, Design, Audio,
Content/XNB, or LZX without a concrete regression. The next and final selected
milestone is Media only.

## Exact strict handoff

```text
                                      BEFORE -> AFTER
REFERENCE_TYPES                           257 -> 257
REFERENCE_MEMBERS                        2964 -> 2964
EXPECTED_RUST_TYPES                       259 -> 259
ACTUAL_RUST_TYPES                         226 -> 235
TOTAL_DIAGNOSTICS                          33 -> 24
MISSING_TYPES                              33 -> 24
MISSING_MEMBERS                             0 -> 0

Graphics                                    0 -> 0
Framework/core                              0 -> 0
Input                                       0 -> 0
Storage                                     0 -> 0
GamerServices                               0 -> 0
Design                                      0 -> 0
Audio                                       9 -> 0
Media                                      24 -> 24
```

Constructor, overload, property, event, base projection, trait, interface,
parameter, return, generic, generic-bound, ref/out, enum value, flags,
delegate, disposal, unexpected type/member, type-kind, internal-type leak,
raw-handle leak, public unsafe API, allowlist, and unmeasured-category counts
are all zero. Normal strict mode exits 1 only for the 24 whole missing Media
types; report-only records the same scoreboard and leak-only is green.

All nineteen Audio types have zero local diagnostics:

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

## Audio implementation evidence

- SoundEffect uses real PCM16/WAV construction, duration/name caching,
  Play/CreateInstance, four native static settings, and exact XNA binary32
  helpers. The 44.1 kHz mono one-second size is 88,198 bytes.
- SoundEffectInstance strongly retains its effect, caches XNA-readable disposed
  properties, uses native transport, and applies single-listener 3D. Multiple
  listeners return UnsupportedRuntime because CNA only supports one.
- DynamicSoundEffectInstance submits copied PCM buffers and uses the existing
  FrameworkDispatcher queue for BufferNeeded. Weak callback contexts, handler
  snapshots, panic containment, self-removal, reentrancy, shutdown, and Game
  recreation are covered.
- Microphone All/Default preserve per-Game facade identity and expose the real
  empty qualified enumeration. Registration lifetime is verified; physical
  capture and BufferReady timing are HARDWARE_PENDING.
- AudioEngine owns XACT; AudioCategory is a parent-owned facade; WaveBank and
  SoundBank own children retaining the engine; Cue retains its required bank
  and engine state. Renderer/look-ahead arguments are forwarded but ignored by
  CNA. Malformed signed XWB/XSB inputs produce CNA handles despite logged parse
  failure. Authored XACT playback is ASSET_PENDING.

Detailed behavioral, ownership, blocker, and fixture evidence is in
`docs/audio-xact-evidence.md`. The generated granular capability table is
`docs/runtime-capabilities.md`, sourced from
`tools/runtime-capabilities/capabilities.json`.

## Measured evidence

```text
XNA-derived observations                  185 -> 205
assertions including final count           186 -> 206
behavior failures                                    0

reviewed ABI functions                    431 -> 528
prototype type positions                 1509 -> 1862
independent C/Rust measurements            936 -> 1004
layouts / callbacks / constants       56/5/243 -> 61/6/253
missing symbols / ABI mismatches                     0
ABI                                      0x0700 / 1792

SoundEffect cycles                                  >=75
SoundEffectInstance cycles                          >=75
DynamicSoundEffectInstance cycles                   >=75
BufferNeeded callback deliveries                    >=50
microphone enumeration/lifetime iterations          >=60
AudioEngine cycles                                    21
malformed bank constructions                         60
native crashes                                        0
observed double-free/UAF                              0
sanitizer status                                NOT_RUN
```

The exact ABI-0.7 stress artifact is:

```text
/tmp/cna-rust-native-070/modules/c-api/libcna_c_api.so
sha256: 6dcefcadb7aa0233da98682bdbc343581a9f1e754a09c641078d1bef97afd7ca
```

Canonical read-only CNA HEAD remains
`1bb2145d99ed572dd4eb15009c34e2e5f410fcf0`. Its unmodified C API build remains
blocked at `CnaCApiCoreExt.cpp:250`, renderer identity `49 == 50`. Do not patch
CNA or accept ABI 0.8.

The qualified sandbox's default PulseAudio route cannot wake its mainloop;
Audio stress uses SDL's dummy backend. This is BACKEND_BLOCKED, not XNA golden
behavior. No sanitizer-compatible exact ABI-0.7 artifact was used.

## Toolchain and template

The active toolchain is Rust 1.85. Its installed components do not contain
rustfmt or Clippy, so the exact statuses are
`RUSTFMT_STATUS=NOT_AVAILABLE` and `CLIPPY_STATUS=NOT_AVAILABLE`; do not claim
either pass. Check, all-feature tests, docs, verifier, ABI, native stress,
capability, and template gates pass. The Audio milestone intentionally made no
source change in `../cna-rust-template`.

```text
TEMPLATE_SOURCE_CHANGED=NO
template HEAD=86612449a2414663f0e17dac98c1bd5239712559
template tests=PASS
template native 60=PASS
template native 600=PASS
fresh vendored consumer tests=PASS
fresh consumer native 60=PASS
developer/sibling absolute-path findings=0
generated-consumer symlinks=0
```

## Final Media queue

Regenerate the scoreboard before implementation; the current exact 24 are:

```text
Album                         AlbumCollection
Artist                        ArtistCollection
Genre                         GenreCollection
MediaLibrary                  MediaPlayer
MediaQueue                    MediaSource
MediaSourceType               MediaState
Picture                       PictureAlbum
PictureAlbumCollection        PictureCollection
Playlist                      PlaylistCollection
Song                          SongCollection
Video                         VideoPlayer
VideoSoundtrackType           VisualizationData
```

Media is a separate dependency family with process-global MediaPlayer state,
queue identity, callbacks, VideoPlayer frame ownership, library collections,
and backend decode limitations. Re-audit its XNA metadata and canonical CNA
ABI before binding it. Do not treat Audio's per-Game callback/ownership choices
as automatic authority for Media.
