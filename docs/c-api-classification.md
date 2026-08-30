# Canonical CNA C API classification

Authority: the canonical headers under
`<cnanext>/modules/c-api/include/CNA/C/*.h`. Measured by
`tools/c-api-inventory/inventory.py` against
`tools/c-api-inventory/classification.json`.

The native ABI verifier in `tools/native-abi` proves that everything
`cna-sys` *does* declare matches the canonical headers. This inventory answers
the other half: what the canonical C API contains that `cna-sys` does **not**
declare, and why each of those routes is absent.

Every public `cna_*` function must match exactly one classification rule. A
route matching none is reported as `UNMAPPED_REQUIRES_REVIEW` and fails the
gate, so a new upstream family cannot enter the ABI unnoticed.

## Categories

| Category | Meaning |
|---|---|
| `RUST_SYS_BOUND` | Declared in the reviewed `cna-sys` slice. Measured, never declared. |
| `STRICT_XNA_BACKING` | Backs a Microsoft XNA 4.0 member. Binding it is a runtime-fidelity improvement, in the selected profile or a wider one. |
| `CNA_EXTENSION_BACKING` | A CNA concept with no XNA 4.0 counterpart. Belongs under `cna::extensions`, never inside `cna::Microsoft::Xna::Framework`. |
| `MANAGED_BY_DESIGN` | The Rust projection reproduces the semantics exactly in Rust. XNA math, value construction, equality, hashing and `ToString` are deterministic managed IL, and the behaviour corpus measures them. |
| `INTERNAL_RUNTIME_ONLY` | Internal to the CNA runtime rather than a consumer route. |
| `TOOLING_ONLY` | CNA's own test seams. A binding that called one would fake runtime state. |
| `PLATFORM_ONLY` | Raw operating-system handles; exposing one would be a public raw-handle leak. |
| `DEFERRED_RUNTIME` | A route this binding should adopt, with the work not yet scheduled. |
| `UPSTREAM_NOT_USEFUL_TO_RUST` | No useful Rust projection exists. CLR `Type.FullName` reflection is the whole of this category today. |
| `UNMAPPED_REQUIRES_REVIEW` | A hole. Always zero. |

## Measurement

```sh
CNA_ROOT=<cnanext> python3 tools/c-api-inventory/inventory.py
```

The gate fails on any unmapped route, any `cna-sys` declaration absent from the
canonical headers, any rule that no longer matches anything, and any override
naming a route that no longer exists. The last two keep the rule file from
drifting into fiction as the ABI grows.

## Reading a classification

`MANAGED_BY_DESIGN` is the category most easily mistaken for a gap. It does not
mean "unimplemented"; it means the native route exists and the Rust projection
deliberately does not call it because the behaviour is exactly reproducible in
Rust and is measured as such. `Vector3.Normalize` is the canonical example: CNA
exports it, and `cna::Microsoft::Xna::Framework::Vector3::Normalize` computes it
in Rust with XNA's operation order.

`STRICT_XNA_BACKING` is the opposite: the route exists, XNA declares the
member, and the Rust projection currently satisfies it some other way or does
not reach it at all. Every entry there is a candidate for a future binding
milestone, and the count is the honest size of the remaining native surface.
