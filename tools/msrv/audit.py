#!/usr/bin/env python3
"""Checks the crates against the declared MSRV without that toolchain installed.

The only complete MSRV check is compiling with the declared compiler, and this
host has one toolchain and no `rustup`, so that check cannot run here. What can
run is a search for library and language items stabilized *after* the declared
version: those are the mistakes an MSRV claim actually accumulates, and one of
them is how `Option::is_none_or` -- stable in 1.82 -- reached a crate declaring
1.74.

This is deliberately a denylist rather than a proof. It says "these known
newer items are absent", never "this compiles on the declared version".
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]

# Item -> the Rust version that stabilized it. Only items plausible in this
# codebase are listed; a false negative is a gap, never a wrong pass.
STABILIZED_AFTER = {
    r"\bOption::as_slice\b": "1.75",
    r"\bptr::from_ref\b": "1.76",
    r"\bptr::from_mut\b": "1.76",
    r"\bArc::unwrap_or_clone\b": "1.76",
    r"\bRc::unwrap_or_clone\b": "1.76",
    r'\bc"': "1.77",
    r"\barray::each_ref\b": "1.77",
    r"\beach_ref\(\)": "1.77",
    r"\btake_if\(": "1.80",
    r"\bLazyLock\b": "1.80",
    r"\bLazyCell\b": "1.80",
    r"\bsplit_once\(\s*&": "1.80",
    r"\bcore::error::Error\b": "1.81",
    r"\bis_none_or\(": "1.82",
    r"&raw (const|mut)\b": "1.82",
    r"\bchar::MIN\b": "1.83",
    r"\bexit_ok\(": "unstable",
    r"\bIterator::next_chunk\b": "unstable",
}


def arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output")
    parser.add_argument("--report-only", action="store_true")
    return parser.parse_args()


def declared_msrv() -> str:
    text = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
    match = re.search(r'rust-version\s*=\s*"([^"]+)"', text)
    if match is None:
        raise ValueError("the workspace declares no rust-version")
    return match.group(1)


def newer(version: str, than: str) -> bool:
    if version == "unstable":
        return True
    left = [int(part) for part in version.split(".")]
    right = [int(part) for part in than.split(".")]
    return left > right


def scan(msrv: str) -> list[dict]:
    findings = []
    for path in sorted((ROOT / "crates").rglob("*.rs")):
        text = path.read_text(encoding="utf-8")
        for line_number, line in enumerate(text.splitlines(), start=1):
            # A whole-line comment is skipped so the scanner does not flag a
            # note explaining why an item was avoided. A trailing comment is
            # still scanned, which only makes the check stricter.
            if line.lstrip().startswith("//"):
                continue
            for pattern, since in STABILIZED_AFTER.items():
                if newer(since, msrv) and re.search(pattern, line):
                    findings.append({
                        "code": "ABOVE_MSRV",
                        "file": str(path.relative_to(ROOT)),
                        "line": line_number,
                        "pattern": pattern,
                        "stabilizedIn": since,
                        "text": line.strip()[:160],
                    })
    return findings


def edition_floor() -> str | None:
    """Rust 2021 needs at least 1.56; a lower MSRV would be incoherent."""
    text = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
    match = re.search(r'edition\s*=\s*"([^"]+)"', text)
    return match.group(1) if match else None


def main() -> int:
    args = arguments()
    msrv = declared_msrv()
    findings = scan(msrv)
    toolchain = subprocess.run(
        ["rustc", "--version"], check=False, text=True, capture_output=True
    ).stdout.strip()
    report = {
        "schemaVersion": 1,
        "declaredMsrv": msrv,
        "edition": edition_floor(),
        "hostToolchain": toolchain,
        "msrvToolchainInstalled": False,
        "checkKind": "source denylist; the declared toolchain is not installed here",
        "findings": findings,
    }
    text = json.dumps(report, indent=2, sort_keys=True) + "\n"
    if args.output:
        Path(args.output).write_text(text, encoding="utf-8")
    print(text, end="")
    return 0 if args.report_only or not findings else 1


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (ValueError, OSError) as error:
        print(f"MSRV audit: {error}", file=sys.stderr)
        raise SystemExit(2)
