# CNA Rust

This workspace exposes [CNA](https://github.com/openeggbert/cna) through a raw
`cna-sys` crate and a safe `cna` crate whose public module tree mirrors CNA and
XNA 4.0 namespaces.

```text
Rust game
   ↓
cna::Microsoft::Xna::Framework compatibility modules
   ↓
cna::CNA::Framework modules
   ↓
cna-sys → stable CNA C ABI → CNA C++
```

## Status

**Early scaffold.** The corrected module hierarchy and initial safe values are
present. `cna-sys` intentionally contains no guessed ABI declarations while the
canonical CNA C API does not exist.

```rust
use cna::Microsoft::Xna::Framework::{Color, Game, GameTime, Vector2};
```

The capitalized modules are intentional compatibility modules. Private
lowercase implementation modules may follow Rust conventions internally, but
the public hierarchy preserves the CNA/XNA namespace identity.

See [architecture](docs/architecture.md) and [plan](plan.md).

## License

CNA Rust is licensed under the [Microsoft Public License](LICENSE), matching CNA.
