# CNA native ABI verification

`bindings.json` inventories the reviewed declarations used by the implemented
Rust projection in `cna-rust-sys`, including the qualified Framework device
management, Touch, and Storage routes. The verifier uses canonical CNA headers
plus Clang's AST as the C prototype authority and compares every reviewed
function with its Rust function-pointer alias. It checks return and parameter
types, count, scalar width/signedness, pointer depth/constness where
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

Current reviewed ABI-0.7 evidence is:

```text
reviewed functions                   431
prototype functions checked          431
prototype type measurements         1509
layout types                          56
callback signatures                    5
constants                            243
all C/Rust measurements              936
mismatches                             0
```

The 431-function slice is intentionally smaller than CNA's 2,861 exported C
functions. It is a reviewed foundation, not a completeness claim. Every new
safe native facade route must add its raw declaration and enter this manifest.
The checked library must report exactly ABI `0x00000700`; ABI 0.8 is not
accepted implicitly. Current ELF/runtime evidence is Linux x86-64 only.
