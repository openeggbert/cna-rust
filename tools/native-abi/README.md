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

Current reviewed ABI-0.20 evidence is:

```text
reviewed functions                   886
prototype functions checked          886
prototype type measurements         3019
layout types                          71
callback signatures                    8
constants                            397
all C/Rust measurements             1236
mismatches                             0
```

The 886-function slice is intentionally smaller than CNA's 4,051 exported C
functions. It is a reviewed foundation, not a completeness claim; every
canonical route outside it carries an explicit classification in
`tools/c-api-inventory/classification.json`. Every new safe native facade route
must add its raw declaration and enter this manifest. The version gate follows
CNA's own `0.x` policy from `docs/c-api/ABI_VERSIONING.md`: the checked library
must report exactly minor 20, because a `0.x` minor bump is a breaking change.
Current ELF/runtime evidence is Linux x86-64 only.
