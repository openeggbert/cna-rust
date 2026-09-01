# Media/Video evidence

Status date: 2026-08-23

Status: selected-profile strict complete. Runtime, platform, backend, and asset
qualification remain separate in
[runtime-capabilities.md](runtime-capabilities.md).

## Strict contract

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

Normal strict and leak-only verification both exit zero. Constructor,
overload, property, event, base, trait, interface, parameter, return, generic,
generic-bound, ref/out, enum, flags, delegate, disposal, type-kind, unexpected
surface, internal-type leak, raw-handle leak, and public-unsafe categories are
all zero.

All 24 selected Media types have zero local diagnostics:

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

The mapping review added only genuine language transformations: explicit
`GameContext` for native generation/thread selection, read-only Media
collection iteration, `Uri -> &str`, `DateTime -> SystemTime`, stream inputs
and outputs, fixed visualization slices, nullable references, and fallible
native operations. No implementation helper became a strict XNA type.

## Ownership inventory

| Public type/value | Classification | Native/lifetime rule |
|---|---|---|
| Song | OWNED | one CNA Song; Game generation; related objects cached |
| Artist | OWNED | one CNA Artist; child collections cached |
| Album | OWNED | one CNA Album; Artist/Genre/Songs cached |
| Genre | OWNED | one CNA Genre; child collections cached |
| Playlist | OWNED | one CNA Playlist; Songs cached |
| Picture | OWNED | one CNA Picture; optional album cached |
| PictureAlbum | OWNED | one CNA PictureAlbum; parent/albums/pictures cached |
| seven Media collections | OWNED | native read-only view; parent generation retained |
| MediaLibrary | OWNED | one CNA library; owns cached collection/source views |
| MediaSource | OWNED | one enumerated or library-associated source facade |
| MediaPlayer | PROCESS_GLOBAL | constructorless facade; active native Game is generation-bound |
| MediaQueue | PROCESS_GLOBAL | stable non-player-owning view for one active generation |
| Video | OWNED | one CNA Video metadata object; Game generation |
| VideoPlayer | OWNED | one CNA player; strongly retains selected Video |
| GetTexture native frame | PARENT_OWNED | player-owned; wrapped as a call-scoped borrow, never destroyed by Rust |
| VisualizationData | MANAGED_VALUE | two fixed 256-element arrays |
| three enums | MANAGED_VALUE | exact XNA integer identities |

Every native owner has one destroy path. Explicit Dispose and Drop are
idempotent; a wrong-thread destroy failure leaves the handle recoverable for
an owner-thread retry. Game shutdown invalidates children before CNA destroys
their parent. The process runtime stores synchronization and generation state,
never a public raw handle or unsynchronized `static mut`.

## Object graph and collections

Album, Artist, Genre, Song, Playlist, Picture, and PictureAlbum use canonical
CNA property and relationship routes. Repeated relationship access returns the
same cached `Arc` facade. Optional native relationships remain `None`; the
binding does not invent Artist, Album, Genre, art, thumbnail, or picture data.

AlbumCollection, ArtistCollection, GenreCollection, SongCollection,
PlaylistCollection, PictureCollection, and PictureAlbumCollection are
read-only wrappers. They preserve Count, checked index access, native order,
stable per-index identity, empty behavior, snapshot enumeration, repeated
property identity, and parent invalidation. No mutable `Vec` is exposed and no
fake entries are inserted.

The qualified provider produced empty music collections and 21 real Picture
entries. Picture name, dimensions, count, bounds, ordering, and identity were
exercised. Deterministic image/thumbnail/token data, SavePicture, nested
picture albums, and populated cross-object catalog metadata remain
`PLATFORM_PENDING`; those gaps are not encoded as XNA golden behavior.

## MediaLibrary, MediaSource, and Song

MediaLibrary construction, source identity, all seven collection properties,
explicit and double Dispose, Drop, retained-child invalidation, wrong-thread
refusal/owner-thread retry, Game shutdown, and 20 lifecycle cycles use CNA
routes. No host directory is substituted for a platform library.

MediaSource enumeration calls CNA directly and copies only the native Name and
MediaSourceType values. Local/removable/network population depends on the
provider, so broader source qualification remains `PLATFORM_PENDING`.

`Song.FromUri` creates a CNA Media Song from a deterministic project-authored
PCM WAV. Twenty cycles verify Name, zero/default catalog metadata, relationship
absence, disposal, double disposal, playback compatibility, and stale/dead
generation rejection. A disposed Song and a Song retained from Game #1 cannot
be played in Game #2. Malformed or failed native creation propagates the CNA
error; Song is never routed through SoundEffect.

## Process-global MediaPlayer and MediaQueue

One synchronized process-global runtime owns static registrations and scalar
state while accepting exactly one active Game generation. Every operation
takes `&GameContext` only to prove owner thread and native generation. Game
teardown invalidates native queues and objects, discards queued old-generation
events, and preserves only XNA process-static subscriptions/settings. Twenty
fresh Games prove that old queue/song facades fail and fresh queues do not
alias them.

The following native routes are verified: Play(Song), Play(collection),
Play(collection,index), Pause, Resume, Stop, MoveNext, MovePrevious, Update,
State, PlayPosition, Queue, GameHasControl, Volume, mute, repeat, shuffle, and
visualization enablement. Pause/resume/stop/movement on an empty queue use the
canonical CNA behavior. CNA accepts an out-of-range collection start index and
stores it until reset; this backend observation is documented but is not made
an XNA golden fact.

MediaPlayer Volume clamps negative infinity to 0 and positive infinity to 1,
preserves NaN and signed negative zero, and persists across Stop and Game
recreation. Mute, repeat, shuffle, and visualization enablement are native
state and likewise survive the tested boundaries.

MediaQueue is one stable cached facade per live generation. Count,
ActiveSongIndex, ActiveSong, checked Item access, and identity are native.
Repeated Queue reads alias one facade and repeated active/index reads alias one
cached Song facade. Queue views never own or destroy MediaPlayer.

## Events and visualization

ActiveSongChanged and MediaStateChanged use process-static CNA registrations.
Each extern trampoline catches panic, records the active generation and
registration cutoff, and enqueues weak work. The existing
FrameworkDispatcher owner-thread pump invokes user handlers; there is no
Media-specific dispatcher. Fifty explicit deliveries cover order, cutoff,
self-removal, reentrant native Stop/Play, panic containment with later
handlers, removal, shutdown, and recreation. The scoped reentrant helpers fail
outside owner-thread Media dispatch. A throwing Game.Update skips dispatch,
and teardown discards that stale event before the next Game.

VisualizationData owns two exact 256-float arrays. The measured C/Rust layout
is 2,056 bytes with alignment 4 and structure version 1. Both enabled and
disabled GetVisualizationData calls reach CNA. The spectrum's *content* is now
qualified too: `RUST-BEHAVIOR-008` lifted the audio `BACKEND_BLOCKED` when SDL
began selecting `pulseaudio` on this host, and a project-authored 1 kHz tone
peaks in bin 12 and a 4 kHz tone in bin 46 -- `round(hz * 512 / 44100)` for
CNA's fixed mixer rate -- with the captured peak equal to the authored
amplitude times MediaPlayer's volume. Nothing is synthesized: a silent authored
fixture is still required to read as an all-zero spectrum, which is what the
earlier readings were measuring. See
[docs/audio-xact-evidence.md](audio-xact-evidence.md#the-real-host-audio-backend).
Video output is a separate backend and remains blocked below.

## Video and VideoPlayer

A deterministic project-authored XNB metadata fixture exercises the normal
Content VideoReader and CNA Video creation route. Duration, 320x180 dimensions,
24 FPS, MusicAndDialog soundtrack, ownership, rollback, and generation-safe
destruction are verified. Twenty Content load/unload cycles invalidate retained
Video objects; an injected post-native-create failure destroys the unpublished
handle and a subsequent load succeeds, proving rollback/cache recovery. The
referenced media path is deliberately absent;
metadata/control qualification is real, while decoded output is
`BACKEND_BLOCKED` and `ASSET_PENDING`.

VideoPlayer uses real create, Play, Pause, Resume, Stop, state, position,
loop/mute/volume, Video identity, GetTexture, Dispose, and destroy routes.
Twenty cycles plus wrong-thread retry verify ownership. Finite Volume outside
[0,1] fails, NaN is accepted and preserved, and IsLooped, IsMuted, and Volume
remain cached after double Dispose. State and transport still fail at the
live-handle boundary after disposal.

### GetTexture boundary

ABI 0.9.0 added `cna_video_player_get_frame_ext`, which publishes the frame
identity the previous milestone was missing: a borrowed texture handle, a
monotonic `generation` that changes only when a frame is actually decoded and
is never restarted by `Stop` or by playing a different video, and a
presentation timestamp.

`GetTexture` now calls that route. A decoded frame is wrapped in a
`PARENT_OWNED` `Texture2D` that never calls native texture destruction. CNA's
frame texture is valid only until the *next call on its player*, including
another `GetTexture`, so the Rust view counts player calls and refuses a stale
frame texture in Rust one call before CNA would answer `INVALID_HANDLE`.
Validating the borrow deliberately makes no native call, because asking the
player would itself be the call that invalidates the handle. `Dispose` and
`Drop` invalidate every outstanding borrow.

XNA owns two frame textures and alternates between them; CNA decodes into one
texture in place. The projection maps both XNA slots onto that one frame and
publishes `generation` for change detection through
`cna::extensions::media::VideoFrameGeneration`, never inside the strict XNA
hierarchy. Before `Play` the route fails, and on HEADLESS no frame is decoded,
so the measured answer is the canonical `Ok(None)`. No pixels are fabricated.

## ABI, behavior, and stress

```text
reviewed functions                  528 -> 730
prototype type positions          1862 -> 2492
independent C/Rust measurements    1004 -> 1028
layouts                              61 -> 62
callbacks                              6 -> 7
constants                            253 -> 262
missing header/library symbols                0
prototype/layout/callback mismatches          0
ABI                                      0x0700
```

All 202 added functions are actually used Media/Video routes. Canonical
headers independently establish full prototypes, pointer depth/constness,
fixed-width values, CNA_Bool, enum/scalar widths, string/blob copying, handle
ownership, exports, and the MediaPlayer callback ABI. The measured callback is
`void (*)(void *user_data)`; the registration retains its context for the
process lifetime, the trampoline never unwinds, and user code runs only later
on the owner thread. No C++ ABI or ABI 0.8 route is used.

The final behavior corpus contains 215 named observations and 216 assertions,
with zero failures. Ten Media observations cover the exact enum identities and
two 256-value visualization views. Provider population, native timing,
backend spectra, and video decoding are deliberately excluded from golden XNA
behavior.

Crash-isolated stress covers at least 20 MediaLibrary, 20 Song, 20
MediaPlayer/Game-generation, 20 queue-generation, 20 Video, 20 VideoPlayer,
20 GetTexture/frame-route, and 50 callback-delivery cycles. Explicit Dispose,
Drop, double Dispose, parent shutdown, recreation, stale facades, wrong-thread
retry, handler panic/self-removal/reentrancy, failed decode/frame acquisition,
post-create rollback, and retained player/video objects are covered.

```text
NATIVE_CRASHES=0
OBSERVED_DOUBLE_FREE=0
OBSERVED_UAF=0
SANITIZER_STATUS=NOT_RUN
```

No instrumented exact ABI-0.7 artifact was used, so sanitizer status is not
upgraded from `NOT_RUN`.

## Final gates and template canary

`cargo check --workspace`, `cargo test --workspace --all-features` with the
qualified native library, and `cargo doc --workspace --no-deps` exit zero. All
28 verifier self-tests, normal strict, leak-only, the complete ABI verifier,
and the 35-row runtime-capability generator/check exit zero. Rust is
`rustc 1.85.0 (4d91de4e4 2025-02-17)`. This source-tarball installation has no
`cargo-fmt` or `cargo-clippy`, so both statuses are `NOT_AVAILABLE`, not passed.

The sibling template source is unchanged at
`86612449a2414663f0e17dac98c1bd5239712559`. Its Cargo test and native 60- and
600-frame canaries exit zero. A fresh generated consumer vendors the completed
binding, passes Cargo test, and completes 60 native frames. Neither repository
contains a symlink; the checked-in template has only its intentional
development sibling dependency, and the generated consumer contains no
developer-absolute or sibling-repository path.

## Qualification and scope

Canonical read-only CNA HEAD is
`1bb2145d99ed572dd4eb15009c34e2e5f410fcf0`; its unmodified C API build still
fails at `CnaCApiCoreExt.cpp:250` because the renderer identity assertion is
`49 == 50`. Runtime evidence uses the qualified ABI-0.7 artifact with SHA-256
`6dcefcadb7aa0233da98682bdbc343581a9f1e754a09c641078d1bef97afd7ca`.

The selected XNA 4.0 Windows runtime Rust projection is structurally complete.
Remaining work is runtime/backend/platform qualification, upstream CNA blocker
reconciliation, packaging/release qualification, and real-game testing.
Wider GamerServices/Avatar, Net, Content Pipeline, Xbox, and Windows Phone are
future profiles and were not opened by this milestone.

```text
MEDIA_MILESTONE_COMPLETE=true
STRICT_ZERO=true
TEMPLATE_SOURCE_CHANGED=NO
```
