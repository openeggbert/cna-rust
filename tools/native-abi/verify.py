#!/usr/bin/env python3
"""Checks the reviewed cna-sys slice against CNA headers and an optional ELF library."""

from __future__ import annotations

import argparse
import ctypes
import json
import os
from pathlib import Path
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
MANIFEST = ROOT / "tools/native-abi/bindings.json"


def arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--cna-root", default=os.environ.get("CNA_ROOT"))
    parser.add_argument("--library", default=os.environ.get("CNA_NATIVE_LIBRARY"))
    parser.add_argument("--report-only", action="store_true")
    parser.add_argument("--output")
    return parser.parse_args()


def split_parameters(text: str) -> list[str]:
    result, start, depth = [], 0, 0
    for index, value in enumerate(text):
        if value == "(":
            depth += 1
        elif value == ")":
            depth -= 1
        elif value == "," and depth == 0:
            result.append(text[start:index])
            start = index + 1
    result.append(text[start:])
    return result


def declarations(header_root: Path) -> dict[str, int]:
    combined = "\n".join(path.read_text(encoding="utf-8") for path in sorted(header_root.glob("*.h")))
    result: dict[str, int] = {}
    pattern = re.compile(r"\bCNA_C_API\s+[^;{}]*?\b(cna_[A-Za-z0-9_]+)\s*\((.*?)\)\s*;", re.S)
    for match in pattern.finditer(combined):
        parameters = match.group(2).strip()
        result[match.group(1)] = 0 if parameters in {"", "void"} else len(split_parameters(parameters))
    return result


def elf_exports(library: Path) -> set[str]:
    output = subprocess.run(
        ["nm", "-D", "--defined-only", str(library)], check=True, text=True, capture_output=True
    ).stdout
    return {
        fields[-1].split("@", 1)[0]
        for line in output.splitlines()
        if (fields := line.split()) and fields[-1].split("@", 1)[0].startswith("cna_")
    }


def main() -> int:
    args = arguments()
    if not args.cna_root:
        raise ValueError("CNA_ROOT/--cna-root is required")
    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    expected = manifest["symbols"]
    headers = declarations(Path(args.cna_root) / "modules/c-api/include/CNA/C")
    findings = []
    for name, arity in expected.items():
        if name not in headers:
            findings.append({"code": "MISSING_HEADER_SYMBOL", "symbol": name})
        elif headers[name] != arity:
            findings.append({"code": "HEADER_ARITY_MISMATCH", "symbol": name, "expected": arity, "actual": headers[name]})

    exports: set[str] | None = None
    actual_version: int | None = None
    if args.library:
        library = Path(args.library)
        exports = elf_exports(library)
        for name in expected:
            if name not in exports:
                findings.append({"code": "MISSING_LIBRARY_SYMBOL", "symbol": name})
        loaded = ctypes.CDLL(str(library))
        loaded.cna_get_abi_version.restype = ctypes.c_uint32
        actual_version = int(loaded.cna_get_abi_version())
        if actual_version != manifest["abiVersion"]:
            findings.append({"code": "ABI_VERSION_MISMATCH", "expected": manifest["abiVersion"], "actual": actual_version})

    report = {
        "schemaVersion": 1,
        "cnaSysDeclarationCount": len(expected),
        "headerExportCount": len(headers),
        "nativeLibraryExportCount": None if exports is None else len(exports),
        "expectedAbiVersion": manifest["abiVersion"],
        "nativeAbiVersion": actual_version,
        "missingHeaderSymbols": sum(x["code"] == "MISSING_HEADER_SYMBOL" for x in findings),
        "arityMismatches": sum(x["code"] == "HEADER_ARITY_MISMATCH" for x in findings),
        "missingLibrarySymbols": sum(x["code"] == "MISSING_LIBRARY_SYMBOL" for x in findings),
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
    except (ValueError, OSError, subprocess.CalledProcessError) as error:
        print(f"ABI verifier: {error}", file=sys.stderr)
        raise SystemExit(2)
