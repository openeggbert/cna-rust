# CNA native ABI verification

`bindings.json` inventories the reviewed declarations used by the implemented
Rust projection in `cna-rust-sys`, including the qualified Framework device
management, Touch, Storage, Audio/XACT, and Media/Video routes. The verifier uses canonical
CNA headers plus Clang's AST as the C prototype authority and compares every
reviewed function with its Rust function-pointer alias. It checks return and
parameter types, count, scalar width/signedness, pointer depth/constness where
representable, callback and structure pointer types, and boolean/enum/flag
representation.

It also checks canonical header declarations, optional Linux ELF exports, and
the exact native ABI version. Independently compiled C and Rust executables
probe `sizeof`, alignment, field offsets, scalar/boolean representation,
constants, and callback signatures; the C compiler is the layout authority.

```bash
python3 tools/native-abi/verify.py \
  --cna-root /path/to/cna \
  --library /path/to/libcna_c_api.so
```

Current reviewed ABI-0.21 evidence is:

```text
reviewed functions                  1326
prototype functions checked         1326
prototype type measurements         4574
layout types                          98
callback signatures                   19
constants                            665
all C/Rust measurements             1845
symbol acquisitions                 1119
symbol type mismatches                 0
mismatches                             0
```

The 1,326-function slice is intentionally smaller than CNA's 4,054 exported C
functions. It is a reviewed foundation, not a completeness claim; every
canonical route outside it carries an explicit classification in
`tools/c-api-inventory/classification.json`. Every new safe native facade route
must add its raw declaration and enter this manifest. The version gate follows
CNA's own `0.x` policy from `docs/c-api/ABI_VERSIONING.md`: the checked library
must report exactly minor 21, because a `0.x` minor bump is a breaking change
even when a particular bump -- as `0.21.0` was -- happens to be purely additive.
Current ELF/runtime evidence is Linux x86-64 only.

`symbol acquisitions` is a separate gate from `prototype functions checked`. It
derives the expected function alias from each acquired symbol's own name and
proves the table field it fills is typed with *that* route's prototype, so a
field wired to a neighbouring route's alias fails even though both aliases exist
and both prototypes are individually correct.
