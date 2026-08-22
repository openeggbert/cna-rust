# Session evidence and next work

## 2026-08-22 — truth, formal mapping, first native slice

Initial evidence:

- Rust source: 321 lines; template Rust source: 255 lines.
- Initial Cargo check: failed because dependency `cna-sys` did not name package
  `cna-rust-sys`.
- Raw ABI: one false availability constant; native runner always unavailable.
- Placeholders: no-op clear/SpriteBatch, disconnected keyboard, empty content,
  and identity-returning matrix operations.
- Template: fake texture/capabilities/renderer, aspirational 3D, three Rust-only
  frames, and unsupported platform claims.

Completed:

- Fixed package/lib identities and verified on official Rust 1.74.
- Wrote the normative mapping and executable naming rules. Strict XNA casing is
  authoritative; the verifier now reports zero unexpected existing names.
- Built neutral metadata extraction and compiler-backed Rust inspection.
- Measured 257 XNA types / 2,964 members -> 259 expected Rust types -> 26
  actual; 233 types and 833 members missing; 1,066 total diagnostics; allowlist
  zero. Nine deeper categories remain explicitly unmeasured.
- Audited a 26-symbol ABI 0.7 runtime/2D slice against 2,861 CNA header exports
  and 2,861 ELF exports, with zero missing names or arity mismatches and 14
  Rust layout tests.
- Added exact ABI rejection, library discovery, safe lifecycle callbacks,
  TimeSpan/GameTime, native clear/viewport/texture/SpriteBatch/keyboard,
  renderer extension facts, ownership and idempotent cleanup.
- Replaced obvious math placeholders with a tested initial pure Rust value
  implementation.
- Rewrote the template around actual CNA execution and `Content/logo.png`.
- Completed 60-frame smoke and 600-frame stability runs on CNA HEADLESS with
  real decode/draw/update and clean shutdown.

Native qualification:

- Unmodified CNA ABI 0.7 C API build failed its correct renderer-identity gates
  because C++ contains `NanoVg` while ABI 0.7 stops at `PixiJs`.
- A test-only library in `/tmp` used two narrow compile corrections, leaving the
  CNA repository unchanged. This is experimental integration evidence, not a
  release artifact.
- The first native run exposed ABI 0.7 child teardown ordering. CNA-Rust now
  releases Rust-owned children before destroy and tolerates the native second
  unload notification idempotently.

Next coherent work:

1. Upstream/follow the CNA NanoVG ABI correction and rerun with an unmodified
   canonical library.
2. Extend the verifier to signatures, enum values, base/interface relations,
   events, ref/out, and disposal before growing broad API surface.
3. Complete the pure value group and differential corpus.
4. Complete Game/window/device manager and lifecycle ownership as one group.
5. Add non-headless resize/keyboard transition evidence.
6. Finish a parameterized fresh-project generation and packaging audit.

Commands and exact final outcomes are recorded in the run's engineering report;
`target/xna-api-report.json` and `target/native-abi-report.json` are generated
evidence and intentionally not source authorities.
