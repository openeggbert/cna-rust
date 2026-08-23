# Runtime capabilities

Generated from `tools/runtime-capabilities/capabilities.json`; do not edit by hand.

Scope: Microsoft XNA 4.0 Windows Audio/XACT projection

Qualified CNA ABI: `0.7`
Qualified artifact SHA-256: `6dcefcadb7aa0233da98682bdbc343581a9f1e754a09c641078d1bef97afd7ca`

| Capability | Strict | Runtime status | Evidence |
|---|---:|---|---|
| SoundEffect | complete | `VERIFIED_MANAGED`, `VERIFIED_NATIVE` | PCM16 construction, deterministic PCM WAV FromStream, duration/name caching, Play/CreateInstance, explicit Dispose/Drop, and 25+ ownership cycles. |
| SoundEffect process-global state | complete | `VERIFIED_MANAGED`, `VERIFIED_NATIVE`, `UPSTREAM_CNA_BLOCKED` | Set/get routes are native-verified. XNA process-static values persist, while the qualified CNA artifact resets them when Game is recreated. |
| SoundEffectInstance | complete | `VERIFIED_MANAGED`, `VERIFIED_NATIVE` | Cached properties, playback state routes, loop/pan/3D ordering, disposal, parent-child lifetime, and 25+ cycles are verified. |
| Apply3D single listener | complete | `VERIFIED_MANAGED`, `VERIFIED_NATIVE` | Exact listener/emitter value copy and canonical cna_sound_effect_instance_apply_3d route verified. |
| Apply3D multiple listeners | complete | `VERIFIED_MANAGED`, `UPSTREAM_CNA_BLOCKED` | The overload is present, but CNA ABI 0.7's canonical implementation refuses every listener count except one; Rust returns UnsupportedRuntime without silently reducing the input. |
| DynamicSoundEffectInstance | complete | `VERIFIED_MANAGED`, `VERIFIED_NATIVE` | PCM16 queue copying, ranges, pending count, transport, disposal, and 25+ dynamic cycles verified. |
| Dynamic BufferNeeded delivery | complete | `VERIFIED_MANAGED`, `VERIFIED_NATIVE` | Native trampoline only enqueues work; Game/FrameworkDispatcher owner-thread delivery, 50+ handlers, duplicate order, self-removal, reentrant submit, panic containment, unsubscribe, shutdown, and recreation verified. |
| Microphone enumeration | complete | `VERIFIED_MANAGED`, `VERIFIED_NATIVE` | Read-only All shape, stable facade identity, Default reconciliation, no-device shape, callback registration lifetime, and 20+ cycles verified against the qualified backend. |
| Microphone capture and BufferReady delivery | complete | `VERIFIED_MANAGED`, `HARDWARE_PENDING` | Exact validation and native routes are implemented; real physical-device sample capture and event timing were not qualified. No synthetic device or samples are used. |
| AudioEngine | complete | `VERIFIED_MANAGED`, `VERIFIED_NATIVE` | XGS signature validation, native construction/update/renderers, category graph, disposal, wrong-thread retry, and 20+ cycles verified with a deterministic parser fixture. |
| AudioEngine renderer and look-ahead semantics | complete | `VERIFIED_MANAGED`, `UPSTREAM_CNA_BLOCKED` | Constructor shape and XNA validation are preserved, but CNA ABI 0.7 ignores rendererId and lookAheadTime. |
| AudioCategory | complete | `VERIFIED_MANAGED`, `VERIFIED_NATIVE` | Stable same-engine identity, equality/hash, volume, pause/resume/stop, parent invalidation, and native category ownership verified. |
| RendererDetail | complete | `VERIFIED_MANAGED`, `VERIFIED_NATIVE` | XNA value equality/hash/string behavior and real renderer collection enumeration are verified; an empty backend collection is valid. |
| WaveBank | complete | `VERIFIED_MANAGED`, `VERIFIED_NATIVE`, `UPSTREAM_CNA_BLOCKED`, `ASSET_PENDING` | Signature checks, native handles, disposal, parent lifetime, and malformed parser paths are exercised. CNA logs malformed XWB parse failure but still publishes a handle; no legal XNA-authored bank is shipped. |
| SoundBank | complete | `VERIFIED_MANAGED`, `VERIFIED_NATIVE`, `UPSTREAM_CNA_BLOCKED`, `ASSET_PENDING` | Signature checks, native handles, failed cue lookup, disposal, and parent lifetime are exercised. CNA logs malformed XSB parse failure but still publishes a handle; no legal XNA-authored bank is shipped. |
| Cue | complete | `VERIFIED_MANAGED`, `ASSET_PENDING` | Complete state/transport/variable/Apply3D/disposal projection and failed native lookup are verified; successful Cue handle behavior requires an authored bank. |
| Authored XACT playback | complete | `ASSET_PENDING` | No Microsoft-authored or otherwise legally redistributable XGS/XSB/XWB success fixture is committed. Parser/error/ownership paths do not constitute authored playback qualification. |
| Host audio backend | complete | `VERIFIED_NATIVE`, `BACKEND_BLOCKED` | The qualified artifact is stable with SDL's dummy driver. This sandbox's default PulseAudio route cannot wake its mainloop and is not used as playback evidence. |
| Native ABI platform coverage | complete | `VERIFIED_NATIVE`, `PLATFORM_PENDING` | ABI 0.7 compiler/export probes pass on Linux x86-64; other operating-system/architecture binaries were not measured in this run. |
| CLR Audio exception delivery | complete | `VERIFIED_MANAGED`, `LANGUAGE_MAPPING_LIMITATION` | The three public exception identities are projected as marker/error values; ordinary Rust operations use Result<T, CnaError> per the established mapping instead of CLR throw semantics. |

`strictComplete` records API/ownership implementation completeness; it does not upgrade pending or blocked runtime semantics.
