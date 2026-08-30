# CNA ABI 0.7 -> 0.20 migration evidence

Status date: 2026-08-30

Authority: the canonical headers, ABI contract and built library of the live
CNA development tree. Microsoft XNA metadata remains the authority for the
public Rust contract; nothing in this migration changes that.

## Repository state

| Repository | Commit |
|---|---|
| `cnanext` | `72262a33ed5ae7657024c7f1251338748a3feee5` (branch `next`) |
| `sharp-runtimenext` | `eebebd862121953538e3b84d43384d70a8a1728d` (branch `next`) |

## The qualification build

Built out of tree from the unmodified `cnanext` checkout:

```sh
cmake -S <cnanext> -B build -G Ninja \
  -DCMAKE_BUILD_TYPE=Debug \
  -DCNA_BUILD_C_API=ON \
  -DCNA_C_API_BUILD_STATIC=OFF \
  -DCNA_SHARP_RUNTIME_ROOT=<sharp-runtimenext> \
  -DCNA_PLATFORM=HEADLESS \
  -DCNA_GRAPHICS_RENDERER=HEADLESS \
  -DCNA_AUDIO_PLATFORM=SDL3 \
  -DCNA_CNAEXT=ON -DCNA_DEVICES=ON -DCNA_ENABLE_NET=ON \
  -DCNA_BUILD_TESTS=ON -DCNA_BUILD_EXAMPLES=OFF
cmake --build build --target cna_c_api --parallel 4
```

| Fact | Value |
|---|---|
| Compiler | GCC 14.2.0 (Debian 14.2.0-19), CMake 3.31.6, Ninja 1.12.1, ccache 4.11.2 |
| Platform / renderer / audio | `HEADLESS` / `HEADLESS` / `SDL3` with `SDL_AUDIODRIVER=dummy` |
| Feature switches | `CNA_CNAEXT=ON`, `CNA_DEVICES=ON`, `CNA_ENABLE_NET=ON` |
| Reported ABI version | `0x00001400` = 0.20.0 |
| Exported `cna_*` symbols | 4,051 |
| `libcna_c_api.so` SHA-256 | `195924825a12290cdd2244fc845e119295de515cf27d1f6b31e1ecc84e93f05d` |

The audio platform is deliberately `SDL3` with SDL's dummy driver rather than
`NULL`. `SoundEffect::Duration` is compiled only under `SOUND_ENABLED`, which
`CNA_AUDIO_PLATFORM=SDL3` defines; a `NULL` audio build answers
`TimeSpan.Zero` for every sound effect and would silently retire the audio
behaviour the corpus measures. That build was run once to confirm the
difference is the backend rather than a semantic change, and the deterministic
qualification build keeps the backend the previous milestone measured.

**The historical upstream build blocker is closed.** The previous milestone
recorded canonical CNA HEAD `1bb2145d99ed572dd4eb15009c34e2e5f410fcf0` failing
its unmodified C API build at `CnaCApiCoreExt.cpp:250`, where a renderer
identity assertion reduced to `49 == 50`. ABI 0.20.0 removed eleven renderer
identities and moved `CNA_GRAPHICS_RENDERER_MAXIMUM` from 50 to 49, which is
exactly that assertion. The unmodified live checkout builds.

## What the migration actually was

The reviewed slice did **not** decay across thirteen minor versions:

| Measurement | Result |
|---|---|
| Reviewed symbols removed from the canonical headers | 0 |
| Reviewed symbols whose arity changed | 0 |
| Prototype mismatches over 731 functions / 2,496 type positions | 0 |
| C-versus-Rust ABI probe mismatches over 1,028 measurements | 0 |
| Structure layouts probed | 62 |
| Callback signatures probed | 7 |
| Constants probed | 262 |
| Reviewed symbols missing from the built library | 0 |
| Library ABI version against the manifest | `0x1400` == `0x1400` |

The migration was therefore not a re-derivation of the slice but a correction
of three things the old baseline had wrong or unstated.

### 1. The version gate encoded a number, not a policy

`native/api.rs` compared the library's reported version with a single constant.
`crates/cna/src/native/abi.rs` now encodes the rule
`docs/c-api/ABI_VERSIONING.md` actually states, and the difference matters:

- A different major is always rejected; no compatibility is defined across one.
- While the canonical major is `0` the ABI is experimental, and CNA ships an
  incompatible change **as a minor increment** — 0.20.0's renderer removal is
  the current example — while also moving the minor for every purely additive
  generation since 0.4.0. A minor therefore does not distinguish the two cases,
  so a `0.x` consumer admits the reviewed minor exactly.
- From ABI 1.0 onward only additive change is permitted within a major, so the
  documented "may require a minimum minor" reading applies and a higher minor
  is admitted.
- A patch is additive within its minor in both regimes, so a higher patch is
  admitted.

The gate is not weakened to admit the new library: `0.19.0` and `0.21.0` are
both rejected, with the reason naming the canonical rule.

### 2. One declared function type was never audited

`cna_vertex_declaration_create_fn` was declared in `cna-sys` but absent from
`tools/native-abi/bindings.json`, so no prototype, arity or export check ever
covered it. It is a real canonical route and is now audited; the reviewed
declaration count moves from 730 to 731 and the prototype type positions from
2,492 to 2,496.

### 3. The canonical surface outside the slice was unaccounted for

731 of the canonical API's 4,051 routes are declared in `cna-sys`. The other
3,320 were previously invisible. `tools/c-api-inventory` now classifies every
one of them; see [docs/c-api-classification.md](c-api-classification.md).

## Selected-profile regression

The Microsoft XNA 4.0 Windows runtime profile returns to strict zero over the
new runtime boundary, unchanged:

```text
REFERENCE_TYPES=257
REFERENCE_MEMBERS=2964
EXPECTED_RUST_TYPES=259
ACTUAL_RUST_TYPES=259
TOTAL_DIAGNOSTICS=0
ALLOWLIST=0
UNMEASURED_CATEGORIES=0
```

The complete workspace test suite passes against the live library, including
the native, audio, media and ownership stress suites.
