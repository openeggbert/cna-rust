#!/usr/bin/env python3
"""Splice newly reviewed symbols into `linked.rs` without regenerating it all.

`generate.py --linked --all-manifest-symbols` re-derives every declaration from
Clang, one process per symbol, and at three thousand symbols that is twenty-five
minutes. The file it produces is one `extern "C"` block of independent
declarations in manifest order, so a slice that adds forty routes can generate
just those forty -- about ten seconds -- and have them spliced in.

The result is byte-identical to a full regeneration as long as the manifest
order is preserved: every symbol already in the file keeps its declaration, and
the new ones are appended in the order the manifest lists them. That is not an
assumption -- it was measured by removing the last five declarations from a
freshly generated file, splicing them back with this tool, and diffing against
the original, which matched exactly.

Run the full generator when a *changed* declaration is expected, when a symbol's
header has moved, or as a periodic check that the two still agree. The ABI
verifier reads `linked.rs` either way, so a splice that went wrong fails there.

    python3 tools/native-abi/splice-linked.py SYMBOL [SYMBOL ...]
"""
import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
LINKED = ROOT / "crates" / "cna-sys" / "src" / "linked.rs"
MANIFEST = ROOT / "tools" / "native-abi" / "bindings.json"


def declared(text):
    """The symbols `linked.rs` already declares, in file order."""
    return re.findall(r"^    pub fn (cna_\w+)", text, re.M)


def main(argv):
    symbols = [value for value in argv if value.strip()]
    if not symbols:
        print("splice-linked: name at least one symbol", file=sys.stderr)
        return 2

    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))["symbols"]
    missing = [name for name in symbols if name not in manifest]
    if missing:
        print(f"splice-linked: not in the manifest: {missing}", file=sys.stderr)
        return 2

    text = LINKED.read_text(encoding="utf-8")
    already = set(declared(text))
    fresh = [name for name in symbols if name not in already]
    if not fresh:
        print("splice-linked: every symbol is already declared")
        return 0

    generated = subprocess.run(
        [sys.executable, str(ROOT / "tools" / "native-abi" / "generate.py"), "--linked", *fresh],
        capture_output=True,
        text=True,
        check=True,
        cwd=ROOT,
    ).stdout
    # The generator prints a whole module; take only the declarations from it.
    body = generated[generated.index("extern \"C\" {") + len("extern \"C\" {"):]
    body = body[: body.rindex("}")].strip("\n")

    closing = text.rindex("\n}")
    spliced = text[:closing] + "\n" + body + text[closing:]
    LINKED.write_text(spliced, encoding="utf-8")
    print(f"splice-linked: added {len(fresh)} declaration(s); the file now has "
          f"{len(declared(spliced))}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
