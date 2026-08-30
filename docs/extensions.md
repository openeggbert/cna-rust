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
