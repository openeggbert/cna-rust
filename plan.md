# CNA-Rust selected-profile plan

Status date: 2026-08-23

Scope: Microsoft XNA Framework 4.0 Windows runtime Rust projection over the
canonical CNA C ABI 0.7. Microsoft XNA metadata/IL/reference behavior governs
the public contract; canonical CNA headers govern the native ABI. Other CNA
bindings are engineering evidence, not contract authority.

## Governing constraints

- Bind only `cna_*` C ABI 0.7 symbols; never C++ ABI or ABI 0.8.
- Never hide a missing CNA semantic with fake Rust state, catalog data, audio,
  or video frames.
- Keep strict structural completeness distinct from runtime/backend/platform
  qualification.
- Keep wider GamerServices/Avatar, Net, Content Pipeline, Xbox, and Windows
  Phone outside this profile.
- Do not reopen completed families without a concrete regression.

## Final strict measurement

```text
REFERENCE_TYPES=257
REFERENCE_MEMBERS=2964
EXPECTED_RUST_TYPES=259
ACTUAL_RUST_TYPES=259
TOTAL_DIAGNOSTICS=0
MISSING_TYPES=0
MISSING_MEMBERS=0
```

All measured categories are zero:

```text
CONSTRUCTOR_MAPPING_MISMATCH  OVERLOAD_MAPPING_MISMATCH
PROPERTY_MAPPING_MISMATCH     EVENT_MAPPING_MISMATCH
BASE_PROJECTION_MISMATCH      TRAIT_MISMATCH
INTERFACE_MISMATCH            PARAMETER_MISMATCH
RETURN_TYPE_MISMATCH          GENERIC_MISMATCH
GENERIC_BOUND_MISMATCH        REF_OUT_MISMATCH
ENUM_VALUE_MISMATCH           FLAGS_MISMATCH
DELEGATE_MISMATCH             DISPOSAL_MISMATCH
UNEXPECTED_TYPES              UNEXPECTED_MEMBERS
TYPE_KIND_MISMATCH            INTERNAL_TYPE_LEAK
RAW_HANDLE_LEAK               PUBLIC_UNSAFE_API
ALLOWLIST                     UNMEASURED_CATEGORIES
```

Normal strict and leak-only both exit zero. The selected XNA 4.0 Windows
runtime Rust projection is structurally complete.

```text
MEDIA_MILESTONE_COMPLETE=true
STRICT_ZERO=true
```

## Completed dependency families

| Family | Local diagnostics |
|---|---:|
| Graphics | 0 |
| Content/XNB | 0 |
| LZX | 0 |
| Framework/core | 0 |
| Input/Touch/Gesture | 0 |
| Storage | 0 |
| GamerServicesComponent | 0 |
| Design | 0 |
| Audio/XACT | 0 |
| Media/Video | 0 |

The final Media family consists of exactly 24 types:

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

They form one real graph rather than structural shells: seven read-only native
collection facades, stable related-object identities, generation-safe owners,
real MediaLibrary/source/Song routes, process-global MediaPlayer state, stable
MediaQueue identity, owner-thread events, visualization, Video content/native
metadata, and VideoPlayer control/disposal behavior.

## Media ownership and lifecycle

Media's synchronized process runtime admits one active Game and monotonically
numbers native generations. All Media native resources are generation-bound;
Game destruction invalidates stale handles, queue/song facades, and pending
events before parent destruction. Scalar MediaPlayer settings and event
subscriptions remain process-global. Wrong-thread releases keep their handles
for owner-thread retry. No global raw pointer or `static mut` is exposed.

Song, Artist, Album, Genre, Playlist, Picture, PictureAlbum, collections,
MediaLibrary, MediaSource, Video, and VideoPlayer have one owned native handle.
MediaPlayer is PROCESS_GLOBAL; MediaQueue is its non-owning stable per-
generation facade; VisualizationData is MANAGED_VALUE. A GetTexture frame is
PARENT_OWNED by VideoPlayer. ABI 0.7 provides no stable frame identity or
invalidation generation, so Rust never wraps or destroys that handle and
returns explicit UnsupportedRuntime if one is reported.

Detailed evidence is in [docs/media-video-evidence.md](docs/media-video-evidence.md)
and the generated
[docs/runtime-capabilities.md](docs/runtime-capabilities.md).

## Measured final evidence

| Evidence | Audio handoff | Final |
|---|---:|---:|
| named XNA-derived observations | 205 | 215 |
| assertions including final count | 206 | 216 |
| behavior failures | 0 | 0 |
| reviewed ABI functions | 528 | 730 |
| prototype type positions | 1,862 | 2,492 |
| independent C/Rust measurements | 1,004 | 1,028 |
| layouts | 61 | 62 |
| callbacks | 6 | 7 |
| constants | 253 | 262 |
| ABI mismatches/missing symbols | 0 | 0 |

Dedicated Media stress covers at least 20 MediaLibrary, 20 Song, 20
MediaPlayer/Game, 20 queue-generation, 20 Video, 20 VideoPlayer, 20 frame-route,
and 50 callback-delivery cycles. It includes explicit Dispose, Drop, double
Dispose, wrong-thread retry, shutdown/recreation, stale resources, callback
panic/self-removal/reentrancy, and backend-blocked frame acquisition.

```text
NATIVE_CRASHES=0
OBSERVED_DOUBLE_FREE=0
OBSERVED_UAF=0
SANITIZER_STATUS=NOT_RUN
```

`SANITIZER_STATUS` remains `NOT_RUN` because no instrumented exact ABI-0.7 CNA
artifact was used.

## Runtime qualification, not structural gaps

- Populated music catalog relationships and removable/network sources:
  `PLATFORM_PENDING`.
- Deterministic picture bytes/tokens/SavePicture and nested album providers:
  `PLATFORM_PENDING`. The host's 21 real Picture entries were exercised.
- Real visualization spectrum on the dummy audio backend: `BACKEND_BLOCKED`.
- Authored video decode/frame production: `BACKEND_BLOCKED` and
  `ASSET_PENDING`; no pixels are fabricated.
- Stable VideoPlayer frame Texture2D identity/generation:
  `UPSTREAM_CNA_BLOCKED`.
- Physical microphone capture and authored XACT playback remain respectively
  hardware/asset pending.
- Repeated Game frame hosting still needs an upstream core callback-context
  rebinding route.

Canonical CNA HEAD is
`1bb2145d99ed572dd4eb15009c34e2e5f410fcf0`. Its unmodified C API build remains
blocked at `CnaCApiCoreExt.cpp:250` (`49 == 50`). Qualified runtime evidence
uses ABI 0.7 artifact SHA-256
`6dcefcadb7aa0233da98682bdbc343581a9f1e754a09c641078d1bef97afd7ca`.

## Post-zero state

There are no remaining selected-profile API families. Work now moves to
maintenance, runtime/platform qualification, upstream CNA blocker
reconciliation, packaging/release qualification, and real-game testing. Do
not begin a future profile merely to extend this milestone.
