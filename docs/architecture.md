# Architecture

```text
cna::Microsoft::Xna::Framework::{Graphics, Input, Content}
                              ↓
cna::CNA::Framework::{Graphics, Input, Content}
                              ↓
cna::CNA::Interop
                              ↓
cna-sys
                              ↓
CNA stable C ABI → CNA C++ core
```

`cna-sys` contains only exact raw declarations derived from canonical CNA C
headers. The safe `cna` crate owns all Rust safety policy and exposes the
capitalized compatibility module trees.

Native resource wrappers will use RAII and `Drop`; borrowed objects carry
lifetimes; errors become `Result`; `Send` and `Sync` are granted only where the
ABI permits them. C++ exceptions and Sharp Runtime ownership/layout details
never cross the C ABI.
