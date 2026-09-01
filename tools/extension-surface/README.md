# CNA extension-surface gate

The strict verifier in [`tools/api-compat`](../api-compat) answers one question:
does `cna::Microsoft::Xna::Framework` contain exactly what Microsoft XNA
declares? Since `RUST-SURFACE-001` its answer is zero, and it reaches zero by
*removing* CNA's own members from the strict hierarchy -- so on its own it
cannot tell a member that moved behind an extension trait from one that was
deleted.

This answers the other half, and keeps the two questions separate: CNA's
extensions are never forced into the XNA contract verifier.

```sh
python3 tools/extension-surface/verify.py --output target/extension-surface.json

# Against rustdoc JSON that already exists, as the strict verifier does.
python3 tools/extension-surface/verify.py --rustdoc target/doc/cna.json
```

Three gates:

- **The migration manifest.** `migrated.json` names every CNA-only member that
  is reachable on a strict XNA type and the trait that publishes it. Each must
  still be declared by a publicly reachable trait, with the same receiver shape,
  parameter names and return type; must be implemented for the strict type; and
  must **not** be a public inherent member of it. The 109 marked
  `movedBy: RUST-SURFACE-001` are the ones that milestone moved; the rest were
  already extension traits and are gated the same way.
- **Public reachability**, computed by walking public modules and public `use`
  items from the crate root rather than trusting rustdoc's `paths`. Every
  `cna::extensions` type is re-exported out of a private module, and rustdoc
  records no canonical path for one of those at all.
- **Nameable public signatures.** A public item whose signature mentions a crate
  type that no public path reaches cannot be used: a caller can invoke it and
  not name what comes back. `PresentationMode` shipped that way -- `pub` in a
  private module, re-exported nowhere, answered by a public method -- and the
  strict verifier does not see it, because `INTERNAL_TYPE_LEAK` looks for
  `cna_sys` and `CNA_` identities rather than for absence.

Re-record the manifest only when the extension surface deliberately changes:

```sh
python3 tools/extension-surface/verify.py --write-manifest
```

`--write-manifest` records what is there now, so it will happily record a
regression. Read the diff.

```sh
python3 -m unittest discover -s tools/extension-surface/tests
```

Eight mutation tests over a synthetic rustdoc document: a member returned to an
inherent impl, one dropped from its trait, a trait no longer implemented, a
trait that stops being exported, an extension type re-exported into the strict
namespace, a public signature naming an unreachable type, and the private-module
re-export the reachability walk exists for.
