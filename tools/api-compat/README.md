# XNA-to-Rust contract verifier

The verifier checks the pinned seven-assembly XNA 4.0 Windows runtime profile. It validates every
reference assembly hash, extracts neutral CLR metadata with Mono, and inspects CNA-Rust through
compiler-produced rustdoc JSON. Source regex is not used as the public-API parser.

Rustdoc JSON is still unstable on Rust 1.74. The script isolates `RUSTC_BOOTSTRAP=1` to the verifier
subprocess; the library and its ordinary quality gates remain stable-only. Supply an existing JSON
file with `--rustdoc` to avoid that step.

```bash
XNA_REFERENCE_PATH=/legal/path/to/xna4/windows \
  python3 tools/api-compat/verify.py --report-only --output target/xna-api-report.json
```

Schema 2 measures type identity/kind, transformed member contracts, bases/traits/interfaces,
parameters and returns, generic arity/bounds, ref/out, enum and flags identity/value coverage,
delegates/events, disposal, constructors, overloads, and properties. The same run emits a
deterministic `typeScoreboard` work queue. `unmeasuredCategories` is authoritative: a category is
never printed as zero unless the implementation actually checked it.

Normal mode exits nonzero for every measured difference; `--report-only` records the incomplete
baseline. `--leak-only` is the zero-tolerance unexpected-type/member, unsafe, internal-type, and
raw-handle surface gate that does not require the reference assemblies. Mapping transformations
belong in `mapping-rules.json`; the allowlist remains empty.
