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

This first verifier revision measures type identity/kind and projected member names, plus public
unsafe/internal-type/raw-handle leakage. Its JSON explicitly lists signature, base/trait, enum and
disposal categories that remain unmeasured. Those fields must never be interpreted as zero. Normal
mode exits nonzero for every measured difference; `--report-only` records a baseline. `--leak-only`
is the zero-tolerance safe-surface gate that does not require the reference assemblies.
