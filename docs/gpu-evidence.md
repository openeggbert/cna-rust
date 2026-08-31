# GPU-backed qualification

The engine-layer scope decision ([engine-layer-scope.md](engine-layer-scope.md))
named one trigger for binding the remaining engine families: **a GPU-backed
qualified artifact**. This file records the measurement that produced one.

Everything below was run through the Rust binding, not through CNA's own tools.

## The host

```text
GPU                     AMD Radeon 780M (radeonsi/phoenix, RADV PHOENIX)
render node             /dev/dri/renderD128, readable by this user
desktop GL              4.6 core (Mesa 25.0.7)
GL ES                   3.2
Vulkan                  1.4.309 loader; RADV reports the 780M
software fallback       llvmpipe, GL 4.5 core, via Xvfb
```

## How a windowless run still reaches the GPU

`SDL_VIDEODRIVER=offscreen` is the qualification environment. It is neither a
headless *renderer* nor a software one: SDL creates a surfaceless EGL context,
the process opens `/dev/dri/renderD128`, and CNA's renderer reports the real
device. The evidence that it is the hardware and not llvmpipe is that the two
answer differently, and the difference was measured rather than assumed:

| | `SDL_VIDEODRIVER=offscreen` | Xvfb `:247` + `SDL_VIDEODRIVER=x11` |
|---|---|---|
| GL version reported by CNA | 4.6 core | 4.5 core |
| EasyGL maximum MSAA | 8x | 4x |
| Vulkan device named by CNA | `AMD Radeon 780M (RADV PHOENIX)` | `llvmpipe (LLVM 19.1.7, 256 bits)` |
| open file descriptors | three on `/dev/dri/renderD128` | none |

Both are useful. The offscreen path is the hardware one and needs no display
server; the Xvfb path is a real GL/Vulkan stack on a CPU rasteriser, which is
what a machine without this GPU would have.

`DISPLAY=:0` with `SDL_VIDEODRIVER=x11` is **not** usable from this session,
and trying it is worse than useless: the process blocks in `poll` before CNA
prints anything, never creates a window, and **holds the X connection open**.
One such run was left behind for about three hours and the machine's keyboard
stopped responding until it was killed -- a hung SDL client can keep an input
grab. The host session is Wayland with Xwayland, and `SDL_VIDEODRIVER=wayland`
against the same compositor does work and does report the hardware, but it draws
on the user's real desktop.

**So nothing in this project runs against the live session.** Every windowed run
goes to an Xvfb display with `SDL_VIDEODRIVER=x11`, and every one of them is
wrapped in a `timeout` so a hang cannot outlive its command. `offscreen` reaches
the hardware with no window and no grab and is the exception that stays
available for hardware evidence.

## Renderers qualified through the Rust binding

Run with `cna-rust-template`, which is an ordinary external consumer of the
public API.

| Renderer | Device the run reported | 60 frames | 600 frames |
|---|---|---|---|
| `OPENGL33` | AMD Radeon 780M (GL 4.6 core) | PASS | PASS |
| `OPENGL33` | llvmpipe (GL 4.5 core, Xvfb) | PASS | PASS |
| `VULKAN` | AMD Radeon 780M (RADV PHOENIX) | PASS | PASS |
| `VULKAN` | llvmpipe (Xvfb) | PASS | — |
| `SDL_RENDERER` | 2D only: `3d=false`, `depth_stencil=false` | PASS | — |
| `OPENGLES3` | AMD Radeon 780M | see below | see below |

`SDL_RENDERER` is listed because it answers a capability question rather than
because it is a candidate for engine work: it reports no 3D and no depth or
stencil buffer, so the engine layer has nothing to run on it.

## The artifacts, and why this session built its own

Other sessions' renderer build directories under `cnanext` are usable for
*renderer* qualification and were consumed read-only for the table above. They
cannot qualify the engine layer, because they are all configured with
`CNA_CNAEXT=OFF`, and upstream compiles the engine layer out under that switch
(`modules/c-api/src/CnaCApiEngineLayer.cpp`: "The engine layer is compiled out
by default"). Against those artifacts `cna_engine_layer_get_version` answers
`0`, and every engine route answers `NOT_SUPPORTED` — which the Rust test
reports as "engine layer absent from this artifact" rather than treating as a
failure.

This session therefore configured its own, out of tree and gitignored, matching
the previously qualified HEADLESS recipe except for the renderer:

```sh
cmake -S <cnanext> -B cmake-build-opengles3 \
  -DCMAKE_BUILD_TYPE=Debug \
  -DCNA_BUILD_C_API=ON -DCNA_C_API_BUILD_STATIC=OFF \
  -DCNA_SHARP_RUNTIME_ROOT=<sharp-runtimenext> \
  -DCNA_PLATFORM=SDL3 -DCNA_GRAPHICS_RENDERER=OPENGLES3 \
  -DCNA_AUDIO_PLATFORM=SDL3 \
  -DCNA_CNAEXT=ON -DCNA_DEVICES=ON -DCNA_ENABLE_NET=ON \
  -DCNA_BUILD_TESTS=OFF -DCNA_BUILD_EXAMPLES=OFF \
  -DCMAKE_CXX_COMPILER_LAUNCHER=ccache -DCMAKE_C_COMPILER_LAUNCHER=ccache
cmake --build cmake-build-opengles3 --target cna_c_api --parallel 4
```

`CNA_BUILD_TESTS` is `OFF` here where the HEADLESS recipe has it `ON`; the
difference is CNA's own test executables, which the exported library does not
contain either way.

### A build raced an upstream commit

The first OPENGLES3 build linked a library with an undefined symbol,
`GraphicsDevice::AcquireRendererThreadContextLeaseForFrame()`. The cause was not
the configuration: `cnanext` moved from `599d14e5` to `0fd4d4e3` while the build
was running, and `GraphicsDevice.cpp` was compiled before the change while
`GraphicsDeviceManager.cpp` was compiled after it. The public C headers did not
change across those commits, so the ABI measurement was unaffected; only the
binary was inconsistent, and rebuilding against one source state fixed it.

The lesson is worth keeping: `cnanext` is a live dependency with other sessions
committing to it, so every artifact in this document names the commit it was
built from, and the ABI verifier is re-run against the artifact that the
qualification actually used.

## Renderer invariance

The engine suite was run against the same OPENGLES3 artifact on the hardware
(`offscreen`, AMD Radeon 780M) and on the virtual display (Xvfb `:247`, GL ES
3.2 on llvmpipe). Every measured value agrees except the two that are honestly
renderer-dependent:

| | hardware | llvmpipe on Xvfb |
|---|---|---|
| scene target, GPU memory estimate | 320x180, 230,400 bytes | same |
| scene-target readback | every pixel the cleared colour | same |
| CPU/GPU particle paths | worst difference 0 over 32 slots | same |
| compute dispatch over 16 elements | exact | same |
| shadow map / cube / cascade presets | 1024, 512, 4x1024 | same |
| GPU timer samples over 30 frames | 1 | 29 |
| image binding for compute | unsupported | unsupported |

The timer difference is the interesting one: a query resolves a frame or two
after the work it timed, and the software rasteriser finishes sooner, so it
collects nearly every frame where the hardware collects one. Both are the same
contract -- poll before opening the next range -- measured on two very different
schedules.

## What this unblocks

`RUST-EXT-010b` was `PRODUCT_DECISION_REQUIRED` on the stated ground that a
headless device renders nothing, so an engine family's semantics could not be
asserted beyond "it returned success". That ground is gone. The trigger the
scope decision named has arrived, and the engine layer is bound one slice at a
time from here, on the criterion that decision already set.

## What the GPU artifact re-qualified

Four measurements had never run on a renderer that draws. They are guarded by
`independent_device_or_skip`, which needs a windowless `GraphicsDevice`, and
every GL-family renderer refuses one -- so on the artifact that can actually
shade, they skipped. Inside a `Game` the device is real, so each now has a
device-backed twin in `crates/cna/tests/extensions_engine.rs`:

| What | Was | Now |
| --- | --- | --- |
| `PbrEffect`'s thirteen scalars, colours, flags and alpha modes | skipped on every GL renderer | round-trips on the OPENGLES3 device |
| `PbrMaterialFull` with a per-slot transform and coordinate set on all seven slots | skipped | round-trips, and an out-of-range coordinate set is refused by message |
| A registered Rust `.cnb` loader decoding a game's own asset type | skipped | runs once, builds its Rust value, and withdraws on drop |
| A failing loader and a panicking one | skipped | both contained, both leave the process usable |

The independent-device constructor itself is unchanged and correct: it succeeds
on `HEADLESS` and is refused on GL with `EasyGLRenderer::CreateContext failed:
surface has no platform window id`. That is a renderer capability, so the guard
stays -- but it no longer means the feature is unqualified anywhere, because the
device-backed twins cover the same contracts where the guard cannot reach.
