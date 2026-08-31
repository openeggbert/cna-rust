# `cna::extensions`

CNA is not only an XNA 4.0 implementation. It has its own renderer registry,
content container, engine layer, device layer and modern input, none of which
XNA 4.0 declares. Those live under `cna::extensions` and never inside
`cna::Microsoft::Xna::Framework`.

That separation is measured, not merely intended: the strict API verifier
compares the XNA hierarchy against Microsoft metadata and reports any member it
does not find there as `UNEXPECTED_MEMBER`. A CNA concept placed in the strict
hierarchy fails the build.

## What belongs here

A route belongs under `cna::extensions` when the canonical CNA C API declares
it and XNA 4.0 has no counterpart. The full accounting of which canonical
routes those are is
[docs/c-api-classification.md](c-api-classification.md); the
`CNA_EXTENSION_BACKING` category is this module's backlog.

Three rules hold throughout:

- **Safe.** No raw `cna_sys` identity, native handle or `unsafe` function
  reaches this API. `PUBLIC_UNSAFE_API` and `RAW_HANDLE_LEAK` stay zero.
- **Truthful.** A route that CNA refuses is surfaced as the error CNA gave.
  Nothing here substitutes a plausible answer for one the runtime declined.
- **Idiomatic.** These are Rust APIs, not transliterated C. The strict XNA
  hierarchy keeps XNA's PascalCase identifiers because recognizability is its
  whole point; `cna::extensions` uses ordinary Rust naming, except where an
  existing member mirrors an XNA-shaped call.

## Modules

### `runtime` — process identity and renderer selection

CNA has 49 renderer identities and chooses between them; XNA had one and no
notion of choosing. The choice is process-global upstream and deliberately so,
because it must be made before the first graphics device exists, which is
before a `Game` has anywhere natural to keep it. Nothing in this module takes a
`Game`.

```rust,ignore
use cna::extensions::runtime::{available_renderers, set_preferred_renderer, RendererType};

if RendererType::VULKAN.is_available()? {
    set_preferred_renderer(RendererType::VULKAN)?;
}
for renderer in available_renderers()? {
    println!("{:?} {:?}", renderer.category()?, renderer.maturity()?);
}
```

`RendererType` is a value with associated constants rather than a Rust `enum`.
CNA's identity set is versioned — ABI 0.20.0 retired eleven identities and moved
the ceiling from 50 to 49, and retired numbers are never reused — so an
identity from a newer CNA stays representable and inspectable instead of
becoming a panic or a lossy `Unknown`. Names come from CNA rather than from a
table in this crate, so a spelling cannot drift out of step: `name()` answers
for the running renderer and reports `UnsupportedRuntime` for any other
identity, because that is the only name CNA publishes.

The selection **latches** when CNA creates its renderer. Before that,
`active_renderer()` fails rather than guessing, and `renderer_selection_is_latched()`
is how a caller tells the two states apart. After it, `set_preferred_renderer`
fails, and the binding surfaces that refusal instead of pretending the change
took effect. `renderer_fallbacks()` is CNA's own account of what it tried and
passed over, empty on a build whose first choice worked.

### `input` — raw joystick devices

XNA had `GamePad` and nothing else: a device that is not an Xbox-shaped
controller could not be read at all. CNA exposes the raw device -- its axes,
buttons, hats and trackballs -- so that is a CNA concept and lives here rather
than beside `GamePad`.

Two canonical rules are preserved rather than smoothed over:

- Capturing an identifier that names no connected device **succeeds** with
  every array empty. That is what the canonical query does, so the projection
  does not turn it into a failure; `capabilities(..).is_connected` is what
  distinguishes an absent device from an idle one.
- Trackball values are relative motion since the previous read, so capturing
  consumes them. Two captures in a row report the movement once.

An index into the enumeration is not the identifier the other routes take;
`JoystickInfo::id` is, and it survives another device disconnecting.

### `logging` — the process log, its level filter, and a Rust sink

XNA has no logging surface, so the whole family is a CNA concept. The
destination is a correctness matter rather than a preference: CNA's default
sink writes to stderr and never stdout, because a terminal-hosted game draws
its frame on stdout and a log line there would corrupt it.

```rust,ignore
use cna::extensions::logging::{set_sink, set_minimum_level, LogLevel};

set_minimum_level(LogLevel::Debug)?;
set_sink(Box::new(|level, category, message: &str| {
    eprintln!("[{level:?}][{category:?}] {message}");
}))?;
```

The sink lives in this crate rather than behind the caller-owned context
pointer CNA offers. Passing null instead means there is no Rust address handed
to C that could dangle across a replacement, and nothing for the trampoline to
validate. A sink that panics is contained at the FFI boundary and uninstalled,
so one bad line does not repeat for the life of the process;
`sink_panicked()` reports that it happened. A sink that logs would re-enter the
sink lock, which CNA's contract forbids; rather than deadlock on a contract
violation the line is dropped.

### `graphics` — CNA graphics facts and construction routes

Renderer diagnostics for a strict `GraphicsDevice`, CNA's reflection-capable
empty `Effect`, and the integer indexers CLR collections have but Rust's
non-overloadable method surface cannot spell inside the strict hierarchy.

It also carries CNA's extended effects. A CRT or depth effect **is** an XNA
`Effect` upstream -- CNA hands back the same handle kind -- so the projection
returns a strict `Effect` and puts the extra knobs on extension traits, rather
than a parallel type that could not be used where an `Effect` is expected. The
ASCII post-processing effect has its own handle kind and so is its own owned
type, released by `Drop`.

The layer is a build option and a renderer may refuse an effect it cannot run.
`is_available()` separates those, and the test admits a refusal only from the
two categories that can honestly produce one. On this host's HEADLESS build all
three effects are created and every knob round-trips through CNA.

It also carries CNA's renderer capability reporting. XNA answered capability
questions through `GraphicsProfile` alone; CNA supports far more backends than
two profiles can describe and publishes per-feature, per-limit and per-format
answers instead.

```rust,ignore
use cna::extensions::graphics::{RendererCapabilityExt, RendererFeature, RendererLimit};

if device.feature_support(RendererFeature::COMPUTE_SHADERS)? == FeatureSupport::Supported {
    let groups = device.limit(RendererLimit::MAX_COMPUTE_WORK_GROUP_COUNT_X)?;
}
```

Three distinctions are kept rather than flattened, because each one is a
different answer:

- `FeatureSupport::Unknown` is a real answer. A renderer that cannot say is not
  the same as one that says no.
- A limit is an `Option`. A renderer that does not publish one reports nothing
  rather than a fabricated zero.
- `FormatSupport` carries two masks. `known` is what the renderer has an answer
  for; `supported` is what it can do. A usage outside `known` was not asked,
  which is not a refusal.

### `media` — deterministic Media hooks and video frame identity

Owner-thread `MediaPlayer` event helpers, and the frame generation and
presentation time CNA publishes alongside a `VideoPlayer` frame. XNA detects a
new frame by object identity because it alternates two textures; CNA decodes
into one texture in place and publishes a monotonic counter instead.

### `events` — the CLR event vocabulary

`EventArgs` and the `EventHandler` trait the strict XNA events are expressed
in. These are language projections rather than CNA concepts, and they live here
so the strict hierarchy contains only members Microsoft declared.

### `content` — the `.cnb` container

XNA has one content format, `.xnb`, and `ContentManager` reads it. CNA has its
own, and it lives here: the strict
`Microsoft::Xna::Framework::Content::ContentManager` is never taught to
reinterpret a non-XNA format, because a game that asks XNA for an asset must
get XNA's answer.

The implemented slice is one complete vertical: build texture data, encode it
as a `.cnb` document, parse a document back, read its metadata, and decode a
texture out of it.

```rust,ignore
use cna::extensions::content::{CnbDocument, CnbTextureData, ReadLimits};

let bytes = CnbTextureData::from_rgba8(width, height, &rgba)?.encode_texture2d("hero")?;
let document = CnbDocument::parse(&bytes, "hero.cnb", ReadLimits::default())?;
let pixels = document.decode_texture2d()?.level_bytes(0, 0)?;
```

Both handle kinds are owned and released by `Drop`, and nothing borrows a
native buffer past a call.

`ReadLimits` is not decoration. A `.cnb` file is untrusted input, so the parser
is bounded rather than trusting the file's own counts, and the test proves a
bound below a real document's size refuses it.

`AssetTypeId::custom` mints a game-defined identity by hashing the name into
CNA's custom range. It is deliberately **not** the inverse of
`AssetTypeId::name`: minting from a built-in type's name yields a custom
identity, not that built-in. Naming the method `from_name` implied an inverse
that does not exist, which the round-trip test caught.

A `.cnb` pixel format that has no XNA `SurfaceFormat` counterpart reports
`UnsupportedRuntime` rather than being forced onto the nearest XNA format.

### `devices` — power, system facts, locale, display and clipboard

CNA's device layer, none of which XNA 4.0 has. Every route takes the same
callback-scoped `GameContext` the strict context-injected members take, because
CNA reaches the host platform through the game's platform binding.

The layer is a build option, and that distinction is preserved rather than
hidden: every route is exported in both states, and the ones the layer
implements refuse with `NOT_SUPPORTED` when it is compiled out.
`is_available()` is how a caller tells "this build has no device layer" from
"this host has no such device", and the test holds the answers to whichever
standard CNA reports.

Two answers are `Option` because CNA's canonical answer for "unknown" would
otherwise read as a real value: a battery percentage and a remaining time, and
the display content scale, whose canonical zero means "no native window" rather
than a scale of zero.

`set_clipboard_text` succeeds when the request was made, which is not the same
as the clipboard changing -- a headless session, or a browser that requires a
user gesture, may ignore it. The projection does not pretend otherwise.

### `net` — the packet buffers XNA keeps internal

XNA's `NetworkSession` reaches a `PacketWriter`'s bytes through an `internal`
member and fills a `PacketReader` the same way, so a game never touches either.
CNA has no session yet, and without these two routes the packet types would be
a write-only sink and a permanently empty source.

### `window` — opaque native window identity

`WindowHandle` cannot be dereferenced or forged through the safe API.

## Experimental upstream API

Where upstream CNA marks an API experimental, this crate says so rather than
presenting it as stable. The engine layer is the large case and is not yet
bound; when it is, it will be reachable under a name that says what it is.

## What is actually verified

`docs/runtime-capabilities.md` carries a row per extension family with what was
measured and what was not. It uses a different status vocabulary from the XNA
capabilities above it, because an extension is measured on a different axis: a
route existing is not the same as it working here, so a family says whether it
is `API_PRESENT`, `VERIFIED_HEADLESS`, `VERIFIED_REAL_RENDERER`,
`NOT_SUPPORTED_BY_BACKEND`, `HARDWARE_PENDING` or `UPSTREAM_BLOCKED`.

## Packaging

Both crates ship the Ms-PL text and the notice file. `tools/package-consumer`
builds an outside project against **only** the files each crate would ship, so
a source file the crate needs but does not package fails there rather than on a
user's machine; it also fails if any staged file still names the development
workspace.

`cargo package -p cna-rust` itself cannot run before `cna-rust-sys` is
published, because Cargo resolves the path dependency's version through the
registry. That is a publish-order fact, not a defect, and it is why the
packaged-source consumer stages the file list directly.


## `extensions::gamer_services` and `extensions::net`

Two families that exist for a reason worth stating plainly: XNA has no way for
a game to *be* the platform, and on a host with no gamer service and no peer,
the strict projection must therefore report nothing. These modules are how a
host supplies what the platform would.

`SignedInGamerPublisher` publishes a roster through
`cna_signed_in_gamer_create_ext` and `cna_gamer_set_signed_in_gamers_ext`. XNA
declares no public `SignedInGamer` constructor and this projection declares
none either; a game that calls the publisher is acting as its own platform, and
after it does, every strict `Gamer.SignedInGamers` read reports CNA's real
roster. The publisher owns each gamer it creates and clears the roster before
releasing them, because CNA refuses to destroy a gamer its collection still
names.

`PendingGuideRequest` publishes CNA's Guide state and the routes that resolve
it. XNA's `BeginShowMessageBox` is answered by a person; CNA leaves the request
pending rather than inventing a choice, so `EndShowMessageBox` reports CNA's
state error until something answers it. This is what answers it. It also
publishes CNA's own visibility setter, which the canonical layer accepts and
ignores because visibility is derived from whether a request is pending -- the
route is exposed with that stated rather than described as if it worked.

`RemoteGamerInjection` and `NetworkEventInjection` do the same job for the
network: they admit a remote gamer to a session and deliver a state change, a
gamer join or leave, or a packet. A single process has no peer, and the strict
`NetworkSession.RemoteGamers` stays empty without them. With them, one process
can prove that a peer lands in the remote roster and not the local one, that a
subscribed handler fires exactly once with the reason it was given, that a
delivered packet arrives byte for byte and decodes through `PacketReader`, and
that a removed handler stops receiving.

`AchievementCollectionExt::item_at` and `AchievementExt::equals` are the third
kind of entry here and the least interesting: XNA overloads `this[...]` by
integer and by string, and Rust cannot give two methods one name, so the strict
type keeps the metadata-selected string form and the integer operation arrives
through a trait. Same collection, same handle, same identity rule.

## `.cnb` Model (RUST-EXT-013, 2026-08-31)

A Rust consumer can now author a compiled model, encode it as a `.cnb`
document, parse that document back and read the whole graph out of it: bone
names, parentage and per-bone transforms; meshes with their parent bone and
their parts in draw order; each part's stride, counts, index element size,
topology, primitive count and effect kind; each part's exact vertex and index
bytes; and each part's material with all eleven factors and its eight texture
slots by name.

### Why it is not XNA's `Model`

`cnb.h`'s model carries `baseColorFactor`, `metallicFactor`, `roughnessFactor`,
`ior`, `KHR_materials_specular` state, `KHR_materials_unlit`, `alphaCutoff`,
morph targets, punctual lights, and a flag saying whether the content was
authored under glTF's lighting conventions.
`Microsoft.Xna.Framework.Graphics.Model` exposes none of that. Projecting it
onto XNA's object model would mean declaring members Microsoft never did, so
the compiled model lives in `cna::extensions::content` and the strict XNA
surface is untouched.

Two facts are authored rather than inferred, and the projection keeps them
that way: `applies_gltf_lighting_policy` decides whether a model expects the
importer's default-light fallback or XNA's unlit `BasicEffect` start, and
`has_bone_hierarchy` selects between attaching meshes to their named bone and
giving every mesh its own child of the root. Both survive the round trip as
written; neither is recomputed from the bone count.

`CnbMaterialTexture` is an enum rather than an index because upstream warns
that the eight texture **names** are not the same eight slots as the seven-long
per-slot state arrays. A typed slot cannot be passed where a plain index
belongs.

### Three defects this slice found

Writing a test that asserts exact structure -- rather than that decoding
succeeded -- turned up three real problems, two of them in code that already
shipped:

1. **`CNA_CnbReadLimits` was missing `max_chunk_alignment`.** The Rust
   structure declared six bounds where C declares seven. Padding hid it
   exactly: both are 48 bytes and every declared field sat at the right offset,
   so `sizeof`, alignment and offset checks all passed.

2. **The layout gate could not have caught that.** It probes the fields the
   manifest names, so a field absent from *both* the Rust struct and the
   manifest is invisible whenever padding absorbs it. The verifier now asks
   Clang for each structure's real field list and compares it with the
   manifest, reporting `LAYOUT_FIELD_SET_MISMATCH`. It checks 103 structures
   and found exactly the one above. Four mutation tests cover it, including one
   for the prefix-matching trap that makes `CNA_Point` also print
   `CNA_PointLightEXT`.

3. **Tightening one read bound silently zeroed the others.** `None` was sent as
   `0`, and CNA reads `0` as a literal limit, not "use the default" -- its
   contract is "initialize with `cna_cnb_read_limits_init`, then lower whatever
   a caller wants tighter". Setting only `max_file_size` therefore set
   `max_chunk_count` to zero and refused every document. The projection now
   seeds from CNA's own defaults and applies only the overrides the caller
   gave, and a test asserts that tightening one bound leaves the rest alone.

CNA validates model geometry properly: an index that addresses a vertex the
part does not have is refused at decode time with the chunk, offset, part and
index named. The fixture builds real indices because of it.

## `.cnb` loader registry (RUST-EXT-013b, 2026-08-31)

A Rust game can now define its own asset type, author a `.cnb` file of that
type, install a Rust loader for it, and get its own value back out -- with no
`Game` in the process. That last part is what `RUST-ABI-008` unlocked:
`cna_cnb_loader_invoke` requires a native content manager, a native content
manager requires a graphics device, and until this milestone the only graphics
device was a running game's.

```text
GraphicsDevice::new  ->  NativeContentManager::new
CnbLoaderRegistry::register("Game.MyAsset", loader)
CnbWriter::new(AssetTypeId::custom("Game.MyAsset"), 1) -> bytes
CnbDocument::parse(bytes) -> resolve_for_document -> invoke -> Arc<dyn Any>
```

### Ownership, stated rather than implied

- **Registration owner.** `CnbLoaderRegistration` is an RAII handle. CNA's
  registrations are process-wide and outlive any content manager, so this is
  what bounds one; dropping it withdraws the loader.
- **Produced objects.** CNA never dereferences, copies or frees a loader's
  object -- upstream says its lifetime "is the caller's own business" -- so the
  registration owns every object its loader produced and releases them all when
  it drops. A load whose object CNA hands to C++ code Rust never sees again is
  still released, at the latest with the registration.
- **Callback context.** The context handed to CNA is the asset type identifier
  by value, not a pointer. There is therefore no context lifetime to get wrong:
  a stale registration finds nothing in the table and fails the load instead of
  dereferencing freed memory.
- **Document lifetime.** The document reaching a loader is borrowed for exactly
  that call. `CnbDocument` carries a `DocumentOwner`, and the callback-scoped
  form never calls destroy -- CNA invalidates the handle when the callback
  returns and it has no destroy operation.
- **Callback thread.** Whatever thread performs the load, so `CnbLoader` is
  `Send + Sync + 'static`.
- **Panic containment.** A panic is caught at the boundary and becomes a failed
  load. No Rust unwind crosses into C.
- **Raw pointers.** None are public. The trait takes `&CnbDocument` and returns
  `Arc<dyn Any + Send + Sync>`; no `void*`, callback pointer or native vtable
  reaches a safe consumer.

### What the round trip proved

`cna_cnb_loader_registry_resolve_for_document` really does check identity, not
just the number, and CNA is stricter about it than the header's load-time
description suggests: **the writer refuses to author an ambiguous file at all.**
Building a custom-typed document whose declared name does not hash to its
identifier fails at build time as a "hash collision", and so does building a
custom-typed document with no canonical name -- with a message naming the call
to make instead. Both halves of the collision defence therefore sit where the
file is written, and this API cannot produce an ambiguous custom-typed document.
The load-time refusal upstream documents still matters for a file some other
toolchain wrote; it is not reachable from here, and the test says so rather than
claiming coverage it does not have.

`cna_cnb_loader_invoke` with no content manager answers `INVALID_HANDLE` rather
than manufacturing one, which is what upstream intends: a placeholder manager
would install the built-in loaders as a side effect of an invoke.

## `.cnb` SpriteFont and SoundEffect (RUST-EXT-013c, 2026-08-31)

Both complete the container's read/write surface for the asset types a Rust
game is most likely to ship. A font round-trips its metrics, its glyphs and its
atlas; a sound round-trips its encoding, rate, shape, loop region and every
sample byte.

Two places where the container distinguishes "absent" from "a particular
value", and where collapsing them would be a real bug, are kept apart:

- `default_character` is `Option<u16>`. XNA throws on a character a font has no
  glyph for **unless** the font declares a fallback, so "no fallback" is not the
  same as "the fallback happens to be `\0`".
- `loop_region` is `Option<(u32, u32)>`. The container writes "no loop" as a
  zero length; handing that back as a zero-length region would give a caller a
  region it could loop on forever.

CNA validates more than the shape: encoding a font whose declared fallback
glyph is not one of the font's own characters is refused, because such a font
would substitute nothing and draw a hole. A test asserts that refusal rather
than only the happy path.

## Text input, IME composition and candidates (RUST-EXT-014, 2026-08-31)

XNA had nothing like this. `Keyboard.GetState` reports which physical keys are
down, which cannot spell a character an IME composed, cannot report a draft the
user has not committed, and cannot see a candidate list at all. All three are
CNA concepts, so they live in `cna::extensions::text_input` and the strict XNA
`Keyboard` is untouched.

Three properties of this boundary are load-bearing:

- **Every string CNA passes is borrowed for the call.** Upstream says so for
  the composition draft and for each candidate, and the views are not
  NUL-terminated. Everything reaching a Rust handler is copied out before the
  call returns; no `CNA_StringView` escapes into a `TextEditing` or a
  `TextEditingCandidates`.
- **Committed text arrives as UTF-16 code units, not characters.** A code point
  above U+FFFF arrives as two calls. `Utf16Assembler` rejoins them and reports
  an unpaired surrogate as exactly that rather than substituting U+FFFD, which
  would be indistinguishable from a replacement character the user really
  typed. Its `push` returns both what the unit completed *and* any high
  surrogate the unit proved unpaired: an earlier draft returned one and
  silently dropped the other. Seven unit tests cover ASCII, accented, CJK,
  astral, orphaned-high, high-orphaned-by-high, lone-low and trailing-high.
- **`selected` is an `Option`.** The container encodes "nothing selected" as
  `-1`; keeping it signed would let a caller use it as an index.

### One native registration per event kind

CNA delivers each event once per *registration*, and this crate's trampoline
delivers to every Rust handler. A registration per subscriber therefore
delivers `registrations x handlers` times -- two subscribers each saw every
character twice, which the round-trip test caught as `aaéé中中🎮` where
`aé中🎮` was expected. The registration is now shared per kind, created with
the first handler and released with the last.

A panicking handler is contained at the boundary and does not stop delivery to
the others; the test subscribes one deliberate panicker alongside the real
handler and asserts both that the text arrived intact and that the panicker ran
once per code unit -- five times, not four, because the emoji is a pair.

### Measured platform behaviour

`cna_text_input_start_ext` succeeds on a HEADLESS host but `is_active` stays
false: there is no platform text-input service to activate, and CNA reports
that rather than claiming an activation nothing backs. Measured with
`build-probe/ext014_text.c`. Delivery is unaffected -- CNA's own raise routes
carry every event through the real path, so the projection is exercised end to
end without a keyboard or an IME.

### A route bound in one acquisition mode only

Adding these routes broke the direct-link build, because
`crates/cna-sys/src/linked.rs` had not been regenerated. The compiler caught it,
but only in the configuration that happened to be built next. The ABI verifier
now checks that the linked module declares exactly the manifest's symbols and
reports `LINKED_DECLARATION_MISSING`, so a route bound in one acquisition mode
and forgotten in the other is a finding rather than a build failure somewhere
else.

## Device enumeration, hot-plug and cursors (RUST-EXT-014b, 2026-08-31)

XNA's input is positional: `Keyboard.GetState()` is *the* keyboard and
`Mouse.GetState()` is *the* mouse, with no way to ask how many there are, which
one moved, or when one is unplugged. CNA reports the devices themselves, so
that lives in `cna::extensions::input_devices`.

### Identity is not an index

Upstream states the enumeration is a point-in-time snapshot and that an index
is valid only until the device set changes, so no index is handed out as a
durable reference. `InputDevice::id` is the stable identity, and it is what a
hot-plug event carries -- a disconnection arrives when the device is already
gone, so an identifier is the only thing that can still mean anything.

`InputDevice::same_device` asks CNA rather than comparing the Rust fields,
because CNA defines device equality -- identifier **and** name -- and a
derived `==` that happened to agree today would be a guess that could quietly
stop agreeing.

### One registration per transition

The same multiplication bug the text-input work found applies here, so each of
the four transitions holds one shared native registration, created with its
first handler and released with its last. The test raises four transitions with
four distinct identifiers and asserts that each reached exactly its own
handler, once, with its own identifier -- a mis-wired subscription would show
up as a keyboard event in the mouse list rather than as a plausible count.

A panicking handler is contained and does not stop delivery to the others.

### Cursors

XNA had no cursor object at all -- `Game.IsMouseVisible` was the whole of it --
so `MouseCursor` stays out of the XNA `Mouse` projection. A texture-backed
cursor is the interesting ownership case: CNA copies what it needs during the
call, so the test drops the texture immediately afterwards and then still uses
the cursor. Disposal is idempotent.

A headless host has no window to show a cursor on. The test records CNA's
actual answer rather than asserting one, and requires only that it be one of
CNA's real answers rather than a crash or a silent success that did nothing.

## Motion sensors (RUST-EXT-009, 2026-08-31)

XNA had these only on Windows Phone, in `Microsoft.Devices.Sensors`, which is
not one of the ten runtime assemblies this binding projects, so accelerometer,
compass and gyroscope live in `cna::extensions::sensors`.

The whole point of the module is that **absence is an answer**. A desktop has
no accelerometer, and the honest report is `SensorState::NotSupported` with no
reading at all -- not `Vector3::ZERO`, which is what a device in free fall
genuinely reports. `current_value` therefore returns `Option`, and returns
`None` whenever `is_data_valid` is false.

`SensorState` keeps CNA's six distinctions rather than collapsing to a boolean,
because a game should be able to tell a user to grant a permission, wait for
initialisation, or stop asking: `NotSupported`, `Ready`, `Initializing`,
`NoData`, `NoPermissions`, `Disabled`.

Units are stated rather than converted: **g** for the accelerometer, **radians
per second** for the gyroscope, **degrees** for the compass headings and
**micro-teslas** for its raw magnetometer axes. Timestamps keep both tick
counts CNA carries -- local ticks and the UTC offset -- because folding the
offset away would lose which zone the device recorded.

### What this host measured

`HARDWARE_PENDING` for every real reading: there is no accelerometer, compass
or gyroscope here. Everything else was measured:

| Behaviour | Result |
|---|---|
| enumeration and count agree | yes, for zero sensors |
| construction with no hardware | succeeds, so a game can ask *why* there are no readings |
| state | `NotSupported`, uniformly across all three families |
| `current_value` with no hardware | `None`, never a zeroed reading |
| sampling interval | a real setting; 200,000 ticks reads back exactly |
| accelerometer and gyroscope injection | **accepted** |
| compass injection | refused: "No test backend is installed and started for this sensor" |
| a reading after an accepted injection | still `None` |

That last row is the one worth keeping. CNA accepts an injected value without
claiming the device now exists, so `is_data_valid` stays false and
`current_value` stays `None`, and the projection passes that through instead of
surfacing an injected number as though hardware had reported it. `is_supported`
stays false too: injecting a reading does not conjure a sensor.

The three families genuinely disagree about injection, and the test records
that rather than smoothing it over. The routes that would install a sensor test
backend are CNA's own test seams -- classified `TOOLING_ONLY` and deliberately
unbound, because a binding that called them would fake runtime state.

## PBR materials, effects and pipeline settings (RUST-EXT-005, 2026-08-31)

None of this is XNA. `BasicEffect` has a diffuse colour and a specular power;
there is no metallic factor, no roughness, no index of refraction, no
tonemapping operator and no HDR anywhere in
`Microsoft.Xna.Framework.Graphics`. It lives in `cna::extensions::pbr`.

### Availability is queried, not assumed

These routes need CNA's engine layer, which is a build-time choice. A symbol
exists either way -- upstream keeps the exported ABI one shape regardless of
what was built -- so presence proves nothing and `engine_layer_version()` is
the query that does. Zero means absent, and this artifact answers **2**,
matching what the header declares.

The two version routes must agree, and a test asserts it: a build where the
number says "absent" while the string names a revision would send a consumer
down the wrong path. The string route is also not CNA's usual size-then-copy
pair, so its size probe answers `BUFFER_TOO_SMALL` rather than success --
treating that as a failure is the difference between reading the string and
refusing to.

### Defaults come from CNA

`PbrMaterial::canonical_defaults()` and
`RenderPipelineSettings::canonical_defaults()` ask the library rather than
restating values here, because restating them is how a binding ends up quietly
disagreeing with the renderer about what "default" means. The measured values
are asserted, so one that changes upstream fails here rather than shipping:

```text
PbrMaterial          metallic 0, roughness 0.5, normal 1, occlusion 1,
                     cutoff 0.5, albedo white, emissive black, blend off
RenderPipeline       exposure 1, gamma 2.2, bloom 1, tonemapping None,
                     quality Medium, shadows Disabled, every pass off
```

The plain `PbrMaterial` and the extended `PbrMaterialEXT` do **not** share
defaults -- the plain one starts non-metallic and half-rough where the extended
one starts fully metallic and fully rough -- and the comment says so, because
assuming they matched is the obvious mistake.

Every pass starting off matters: a game opts into HDR, bloom, SSAO and shadows
rather than discovering it is already paying for them.

### The effect round trip

`PbrEffect` is created on a device -- an independently constructed one works,
which is what makes this testable without a `Game` -- and every scalar it
carries round-trips through distinguishable values, so a property read back
from a neighbouring slot is visible rather than plausible. All three alpha
modes round-trip, not only the one the main assertion uses, and every
tonemapping, render-quality and shadow-quality identity is walked in both
directions so a mapping that collapsed two variants onto one number would fail.

`PbrMaterial` deliberately carries no textures. The canonical structure has
non-owning handle slots, and a safe Rust value holding raw handles would be a
raw-handle leak; textures belong on the effect, where the lifetime relationship
is real.

## Engine-layer render settings (RUST-EXT-010, 2026-08-31)

The engine layer is 857 canonical routes, and binding it wholesale would be
binding for a percentage. This is one coherent vertical slice, chosen because
it is genuinely useful to a game -- it is what a graphics-settings screen is
made of -- and because it can be tested semantically on a headless host, which
most of the layer cannot.

`EngineRenderSettings` owns CNA's 50-field settings value and exposes typed
accessors rather than the structure. A `#[repr(C)]` field set is the ABI's
shape, not an API: making it public would turn every later CNA field addition
into a breaking change here.

Three operations carry the real semantics, and all three were measured:

### `normalize` -- what the engine will actually use

Upstream runs every field through its own setter and reads it back, so a caller
can see what a value will become *before* handing it to a pipeline. Thirty-one
corrections are documented, ten clamping to a two-sided range and twenty-one
flooring. Measured on this artifact:

| Field | Asked | Used |
|---|---:|---:|
| `exposure` | -5.0 | 0.0 |
| `gamma` | -1.0 | 0.01 -- a positive minimum, not zero, which a renderer would divide by |
| `bloom_intensity` | -2.0 | 0.0 |
| `ssao_radius` | -4.0 | 0.0 |
| `bloom_iterations` | -7 | **-7** |
| `ssao_sample_count` | -3 | **-3** |
| `ssr_step_count` | -11 | **-11** |

The last three are the point. The continuous fields are corrected; the integer
counts are **not**, and are not among the thirty-one. A caller that assumed
every field was corrected would hand the engine a negative bloom pyramid depth.
The test asserts the exact pass-through rather than a range, so a future
upstream change in either direction is visible.

`normalize` is also idempotent, which is what makes it safe to call on every
settings change.

### `apply_quality_preset` -- only what has been decided

Upstream derives only the fields a quality dial has been settled for -- today
bloom's pyramid level count and the FXAA edge threshold -- and deliberately
leaves the rest alone rather than guessing. The test asserts that Low and Ultra
differ and that Ultra does not use fewer bloom levels, rather than asserting
that every field follows the dial, which would be asserting a design upstream
explicitly declined to commit to.

### `apply_from_text` -- a count, not a boolean

Unrecognised fields are skipped rather than refused, and the returned count is
what makes that usable: a caller compares it with what it meant to set and can
tell a typo from a stale key. Unrecognised text applies zero and succeeds;
empty text is the degenerate case of the same rule.

`PbrEffect` from `RUST-EXT-005` is the other engine-layer object already bound,
so the layer now has two working slices rather than a survey.

## Haptics (RUST-EXT-014c, 2026-08-31)

XNA had exactly one haptic operation: `GamePad.SetVibration(index, left,
right)` -- two motor amplitudes on a controller. CNA reports the *device*: how
many axes forces can be directed along, how many effects it holds, how many can
play at once, which waveform and condition families it supports, and whether it
takes a global gain or an autocentre setting.

These are not the same thing, and this module deliberately does not compress
the second into the first. `GamePad.SetVibration` stays exactly where XNA put
it; a wheel that supports spring, damper and friction conditions is described
here, because describing it as "left motor, right motor" would discard almost
everything true about it. `HapticFeatures::LEFT_RIGHT` is one bit of
seventeen, and a test walks all of them to prove no two share a bit -- two
features collapsing onto one would make a device claim a capability it lacks.

### Two measurements that shaped the API

**`Applied` is its own type.** CNA answers "did it apply" separately from "did
it succeed", and that distinction is load-bearing. Setting a gain on a device
with no gain control is not an error, but it also did not happen. Folding the
two together would let a settings screen show a working slider that never
buzzes.

**Opening an unknown identifier is not an error.** Measured: `open(u32::MAX)`
succeeds and hands back a device object whose `is_open` is false, whose
capabilities are empty, and whose every operation reports `Applied(false)`.
That is coherent, and it is exactly the case `Applied` exists for. The test
asserts all of it rather than the refusal an earlier draft assumed.

**`-1` is not `0`.** The default capabilities report `max_effects` and
`max_effects_playing` as `-1`, meaning *not known*, where zero would mean "holds
no effects" -- a real and different answer. The projection passes the `-1`
through, and the test asserts it, because rewriting it to zero would turn an
unknown into a claim.

Real forces are `HARDWARE_PENDING`: no haptic device is attached to this host.
Everything else -- enumeration agreeing with its count, capability reads,
opening and closing, and CNA's own name-inclusive capability equality -- is
measured.
