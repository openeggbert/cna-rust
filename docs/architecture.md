# Architecture

```text
cna::Microsoft::Xna::Framework compatibility modules
                              ↓
cna crate-private bridge and crate-root binding utilities
                              ↓
cna-sys raw declarations
                              ↓
CNA stable C ABI
                              ↓
CNA C++: Microsoft::Xna::Framework
```

`cna-sys` will contain exact raw declarations derived from canonical CNA C
headers. The safe crate owns Rust errors, RAII, borrowing, callback context,
thread-safety decisions, and shutdown.

There is no public `CNA::Framework` module because no corresponding namespace
exists in CNA C++. A public `CNA` module is valid only for specific extensions
that mirror real native `CNA::...` declarations.
