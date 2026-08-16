# CNA Rust

This workspace exposes [CNA](https://github.com/openeggbert/cna) through raw
`cna-sys` declarations and a safe `cna` crate with an XNA-compatible module
tree.

```text
Rust game
   ↓
cna::Microsoft::Xna::Framework::{Graphics, Input, Content}
   ↓
cna-sys
   ↓
CNA stable C ABI
   ↓
CNA C++ Microsoft::Xna::Framework implementation
```

## Status

**Early scaffold.** The compatibility modules and first safe value types exist.
`cna-sys` contains no guessed declarations while the canonical C ABI is absent.

```rust
use cna::Microsoft::Xna::Framework::{Color, Game, GameTime, Vector2};
```

Binding-specific `Result`, error, and runner utilities live at
the `cna` crate root. There is deliberately no `cna::CNA::Framework` module.
Future `CNA` modules must mirror concrete native `CNA::...` extensions.

See [architecture](docs/architecture.md) and [plan](plan.md).

## License

CNA Rust is licensed under the [Microsoft Public License](LICENSE), matching CNA.
