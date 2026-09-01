# C reproducers for the upstream findings

Each file here is a self-contained C program against CNA's public headers, with
no Rust in the process. That is the point: a finding measured only through a
binding invites the answer "then fix your binding", and these remove it.

They live in the repository rather than in `build-probe/` -- which is
gitignored -- because `docs/upstream-findings.md` cites them, and a finding
whose reproducer is not in the tree is a finding the next person has to rebuild
from prose. Build them *into* `build-probe/`, which is where every throwaway
binary in this repository goes:

```sh
CNA=<path to cnanext>
gcc -O0 -g -rdynamic -D_GNU_SOURCE tools/reproducers/<file>.c \
  -I$CNA/modules/c-api/include \
  -L$CNA/cmake-build-headless/modules/c-api -lcna_c_api \
  -o build-probe/<file>
LD_LIBRARY_PATH=$CNA/cmake-build-headless/modules/c-api ./build-probe/<file> [args]
```

`cmake-build-headless` is the artifact these were measured against, because it
is the one whose renderer makes a `GraphicsDevice` without a window. A GL-family
renderer refuses that, and each probe says so and exits 0 rather than pretending
to have measured something.

| File | Finding | What it shows |
|---|---|---|
| `ext015g_model_ownership.c` | — | that a `models.h` view handle is independently counted: it keeps answering, name and all, after `cna_model_destroy`. This is why `ModelBoneView` and `ModelMeshView` carry no lifetime parameter. |
| `ext015g_load_model_destroy.c` | `RUST-UPSTREAM-021` | destroying a content-loaded model with a mesh part faults. Takes a content root and an asset name. |
| `ext015g_handbuilt_mesh.c` | `RUST-UPSTREAM-021` | the control: the same shape built by hand destroys cleanly, which is what makes *content-loaded* the answer. |
| `ext015g_manager_teardown.c` | `RUST-UPSTREAM-021` | that leaking the model handle does not avoid the fault; it moves it to process exit. |
