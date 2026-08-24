#!/usr/bin/env python3
"""Validate the machine capability inventory and generate its Markdown view."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
DEFAULT_SOURCE = ROOT / "tools/runtime-capabilities/capabilities.json"
DEFAULT_OUTPUT = ROOT / "docs/runtime-capabilities.md"
VALID_STATUSES = {
    "VERIFIED_MANAGED",
    "VERIFIED_NATIVE",
    "UPSTREAM_CNA_BLOCKED",
    "BACKEND_BLOCKED",
    "ASSET_PENDING",
    "HARDWARE_PENDING",
    "PLATFORM_PENDING",
    "LANGUAGE_MAPPING_LIMITATION",
}


def load(path: Path) -> dict:
    data = json.loads(path.read_text(encoding="utf-8"))
    if data.get("schemaVersion") != 1:
        raise SystemExit("unsupported runtime-capability schema")
    rows = data.get("rows")
    if not isinstance(rows, list) or not rows:
        raise SystemExit("runtime-capability inventory has no rows")
    names: set[str] = set()
    for row in rows:
        name = row.get("capability")
        if not isinstance(name, str) or not name or name in names:
            raise SystemExit(f"invalid or duplicate capability: {name!r}")
        names.add(name)
        statuses = row.get("statuses")
        if not isinstance(statuses, list) or not statuses:
            raise SystemExit(f"capability has no status: {name}")
        unknown = set(statuses) - VALID_STATUSES
        if unknown:
            raise SystemExit(f"unknown statuses for {name}: {sorted(unknown)}")
        if row.get("strictComplete") is not True:
            raise SystemExit(f"runtime strict completion is false for {name}")
    return data


def render(data: dict) -> str:
    lines = [
        "# Runtime capabilities",
        "",
        "Generated from `tools/runtime-capabilities/capabilities.json`; do not edit by hand.",
        "",
        f"Scope: {data['scope']}",
        "",
        f"Qualified CNA ABI: `{data['abiVersion']}`",
        f"Qualified artifact SHA-256: `{data['qualifiedArtifactSha256']}`",
        "",
        "| Capability | Strict | Runtime status | Evidence |",
        "|---|---:|---|---|",
    ]
    for row in data["rows"]:
        statuses = ", ".join(f"`{value}`" for value in row["statuses"])
        evidence = row["evidence"].replace("|", "\\|")
        lines.append(f"| {row['capability']} | complete | {statuses} | {evidence} |")
    lines.extend(
        [
            "",
            "`strictComplete` records API/ownership implementation completeness; it does not upgrade pending or blocked runtime semantics.",
            "",
        ]
    )
    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--source", type=Path, default=DEFAULT_SOURCE)
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    rendered = render(load(args.source))
    if args.check:
        if not args.output.exists() or args.output.read_text(encoding="utf-8") != rendered:
            raise SystemExit("runtime capability Markdown is stale")
    else:
        args.output.write_text(rendered, encoding="utf-8")
    print(f"RUNTIME_CAPABILITY_ROWS={len(load(args.source)['rows'])}")
    print("RUNTIME_CAPABILITY_STATUS=PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
