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

## Two axes

"Why does this route exist?" and "why is it not bound?" are different
questions, and answering only the first is how routes go missing. The inventory
therefore carries two independent closed sets.

`category` says which part of the projection owns a route. `binding` says
whether Rust binds it, and when it does not, why not -- with evidence, and with
a durable task when the reason is a block or a deferral.

Until this split existed, `RUST_SYS_BOUND` sat inside the purpose axis. That had
two consequences worth remembering, because both were real: binding a route
*erased* its purpose, so whole headers ended up with no purpose rule at all and
rules were deleted as "unused" when their last route was bound; and no unbound
route carried a binding decision, so "this backs XNA" was allowed to stand as
the answer to "why can't Rust reach it?" -- which it is not.

A purpose is never an answer to the binding question.

## Categories -- what a route is for

| Category | Meaning |
|---|---|
| `STRICT_XNA_BACKING` | Backs a Microsoft XNA 4.0 member. Binding it is a runtime-fidelity improvement, in the selected profile or a wider one. |
| `CNA_EXTENSION_BACKING` | A CNA concept with no XNA 4.0 counterpart. Belongs under `cna::extensions`, never inside `cna::Microsoft::Xna::Framework`. |
| `MANAGED_BY_DESIGN` | The Rust projection reproduces the semantics exactly in Rust. XNA math, value construction, equality, hashing and `ToString` are deterministic managed IL, and the behaviour corpus measures them. |
| `INTERNAL_RUNTIME_ONLY` | Internal to the CNA runtime rather than a consumer route. |
| `TOOLING_ONLY` | CNA's own test seams. A binding that called one would fake runtime state. |
| `PLATFORM_ONLY` | Raw operating-system handles; exposing one would be a public raw-handle leak. |
| `DEFERRED_RUNTIME` | A route this binding should adopt, with the work not yet scheduled. |
| `UPSTREAM_NOT_USEFUL_TO_RUST` | No useful Rust projection exists. CLR `Type.FullName` reflection is the whole of this category today. |
| `UNMAPPED_REQUIRES_REVIEW` | A hole. Always zero. |

## Binding statuses -- whether Rust binds it, and why not

`BOUND` is measured from the reviewed `cna-sys` slice and may never be
declared. Every other status must be stated in `classification.json` with a
`reason` a reader can check; `BLOCKED_UPSTREAM` and `DEFERRED_TRACKED`
additionally require a `task` that owns the route, so nothing sits in limbo
under a status that sounds like progress.

| Status | Meaning |
|---|---|
| `BOUND` | In the reviewed `cna-sys` slice. Measured. |
| `DELIBERATE_NON_BINDING` | Rust reaches the same capability another way, or the route should not exist in a safe Rust API. The reason must name the Rust surface that replaces it. |
| `BLOCKED_UPSTREAM` | A CNA defect stops it. Needs a finding id. |
| `BLOCKED_RENDERER` | No renderer available here can run it. |
| `BLOCKED_PLATFORM` | This platform cannot reach it. |
| `BLOCKED_HARDWARE` | No such device is attached **and** CNA ships no test backend for it. A missing device alone is not enough. |
| `BLOCKED_ASSET` | It needs an asset this project may not carry. |
| `DEFERRED_TRACKED` | Real work, deliberately not now. Needs a backlog id. |
| `ACTIONABLE_LOCAL` | Real work, reachable today, nobody has done it. **The gate fails on these.** |
| `UNREVIEWED` | Nobody has decided. **The gate fails on these too.** |

The gate is what makes the census worth reading: it fails while any route is
`UNREVIEWED` or `ACTIONABLE_LOCAL`, so "the census passes" and "every route has
a decision somebody stands behind" are the same statement.

## Bound is not the same as reachable

`BOUND` says a route is declared in `cna-sys` and resolved when the library
loads. It does not say a consumer can call it. `RUST-CENSUS-002` asks the other
question, and `tools/c-api-inventory/reachability.py` answers it by walking the
crate:

```text
a file outside native/  ->  a native/ wrapper  ->  ...  ->  self.table.field
```

Three measured facts make that exact rather than approximate. Every route is
acquired at exactly one place, `field: symbol!(cna_x, ...)`, which ties the C
route to the Rust field holding its pointer. All 3,244 field names are unique
across the crate, so an occurrence of `.category_pause` names one route and no
other. And the definition is not a use: the `symbol!` initialiser and the
struct declaration are blanked before any name is collected, or the loader --
which must name every field -- would make every route look reachable.

This replaced a check that compared a route's **name** to Rust identifiers.
That could not work here, because the field is not named after the route:
`cna_audio_category_pause` lives in `AudioApi::category_pause`. It reported 894
dead routes, then 1,077, then 303. The walk measures 138, and the difference is
not a threshold that moved -- it is 165 routes that always had a safe API in
front of them.

The walk has no hop limit. Measured on this crate, 2,400 routes are named by the
safe layer directly, 696 sit behind one wrapper and 4 behind two; a hop-limited
check set at two would still miss `cna_error_get_last_info`, which every
fallible call reaches through `Native::check` and `Native::last_error_category`.

A bound route with no safe call site is **not** automatically a defect. The
gate therefore does not fail on the count; it fails on an unexplained one. Each
must appear in `classification.json`'s `bindingUnreachable` with a `reason` and
one `outcome` from a closed set:

| Outcome | Meaning |
|---|---|
| `ATOMIC_TABLE_MEMBER` | The family is acquired as one table so a library missing any of it fails at load rather than at first use, and this member is deliberately absent from the safe API. |
| `IMPLEMENTED_IN_SAFE_RUST` | Rust does the same work itself, and more faithfully. Must name the Rust in `rustEvidence`, which the census greps for. |
| `OUTSIDE_XNA_SURFACE` | The route backs something XNA's public API does not have: an explicit interface implementation, a mutator XNA throws from, or a CNA-internal seam. |
| `BLOCKED_UPSTREAM` | A CNA defect stops the safe layer calling it. Names a finding. |
| `BLOCKED_ENVIRONMENT` | Nothing available here -- renderer, platform, device or asset -- can drive it. |

A justification is checked, not trusted. The gate fails when one names a route
that has since acquired a safe caller, stopped being bound, or left the
headers, and when its `rustEvidence` names Rust that is no longer there.
Measured, 2026-09-01, one mutation at a time against a passing gate:

| Planted defect | Gate | What it reported |
|---|---|---|
| a justification deleted | fails | `boundWithoutSafeCallSiteUnjustified: [cna_packet_writer_write_matrix]` |
| a justification's `rustEvidence` renamed | fails | `justificationsNamingAbsentRustCode: [cna_packet_writer_create: pub struct PacketScribbler]` |
| a justification for a route that *is* reachable | fails | `staleUnreachableJustifications: [cna_audio_category_pause]` |
| the only safe caller of a route deleted | fails | `boundWithoutSafeCallSiteUnjustified: [cna_net_get_last_join_error]` |

The fourth is the one worth reading twice. Deleting `AudioCategory::Pause`
alone does **not** fail the gate, and should not: `Resume`, `SetVolume` and
`Stop` reach the same wrapper, and the wrapper still reads
`category_pause`. The route is dead only when nothing reaches the field, which
is what the walk measures and what a name-matching check could not.

## Measurement

```sh
CNA_ROOT=<cnanext> python3 tools/c-api-inventory/inventory.py
```

The gate fails on any unmapped route, any `cna-sys` declaration absent from the
canonical headers, any rule that no longer matches anything, any override
naming a route that no longer exists, and any bound route with no safe call
site and no justification. The middle three keep the rule file from drifting
into fiction as the ABI grows.

```sh
python3 -m unittest discover -s tools/c-api-inventory/tests
```

The reachability walk's own mutation tests: a route three wrappers deep is
reachable, a route whose last safe caller is deleted stops being, and a field
named only in a comment, a string, or its own acquisition is not a call site.

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
