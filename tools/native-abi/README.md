# CNA native ABI verification

`bindings.json` inventories the reviewed runtime/2D declarations currently present in
`cna-rust-sys`. The verifier checks names and arities against CNA's canonical headers and, when
`CNA_NATIVE_LIBRARY` is supplied, checks ELF exports and the experimental ABI version.

```bash
CNA_ROOT=/path/to/cna CNA_NATIVE_LIBRARY=/path/to/libcna_c_api.so \
  python3 tools/native-abi/verify.py
```

The declaration count is intentionally smaller than CNA's complete header export count. This is a
measured initial slice, not a claim that `cna-rust-sys` is complete. Rust unit tests guard layouts
in the slice. ELF evidence proves Linux only.
