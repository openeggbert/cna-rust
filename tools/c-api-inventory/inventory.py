#!/usr/bin/env python3
"""Inventories the complete live CNA C API and classifies every canonical route.

The native ABI verifier in ``tools/native-abi`` proves that the reviewed
``cna-sys`` slice matches the canonical headers.  This tool answers the other
half of the question: what does the canonical C API contain that ``cna-sys``
does *not* declare, and why is each of those routes absent?

Every public ``cna_*`` function must match exactly one classification rule.  A
route that matches none is reported as ``UNMAPPED_REQUIRES_REVIEW`` so a new
upstream family cannot enter the ABI without being noticed here.
"""

from __future__ import annotations

import argparse
import fnmatch
import json
import os
from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[2]
CLASSIFICATION = ROOT / "tools/c-api-inventory/classification.json"
CNA_SYS = ROOT / "crates/cna-sys/src/lib.rs"

# Every category a canonical route may carry.  The set is closed: adding one is
# a deliberate project decision recorded in docs/c-api-classification.md.
CATEGORIES = (
    "RUST_SYS_BOUND",
    "STRICT_XNA_BACKING",
    "CNA_EXTENSION_BACKING",
    "MANAGED_BY_DESIGN",
    "INTERNAL_RUNTIME_ONLY",
    "TOOLING_ONLY",
    "PLATFORM_ONLY",
    "DEFERRED_RUNTIME",
    "UPSTREAM_NOT_USEFUL_TO_RUST",
    "UNMAPPED_REQUIRES_REVIEW",
)

FUNCTION = re.compile(
    r"\bCNA_C_API\s+[^;{}]*?\b(cna_[A-Za-z0-9_]+)\s*\((.*?)\)\s*;", re.S
)
HANDLE_TYPEDEF = re.compile(r"typedef\s+CNA_Handle\s+(CNA_[A-Za-z0-9_]+)\s*;")
SCALAR_TYPEDEF = re.compile(
    r"typedef\s+(?:u?int(?:8|16|32|64)_t|float|double)\s+(CNA_[A-Za-z0-9_]+)\s*;"
)
STRUCT_TYPEDEF = re.compile(r"typedef\s+struct\s+(CNA_[A-Za-z0-9_]+)\s*\{")
CALLBACK_TYPEDEF = re.compile(
    r"typedef\s+[A-Za-z0-9_ ]+?\s*\(\s*\*\s*(CNA_[A-Za-z0-9_]+)\s*\)\s*\("
)
DEFINE = re.compile(r"^\s*#\s*define\s+(CNA_[A-Za-z0-9_]+)\b", re.M)


def arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--cna-root", default=os.environ.get("CNA_ROOT"))
    parser.add_argument("--output")
    parser.add_argument("--report-only", action="store_true")
    parser.add_argument(
        "--list",
        metavar="CATEGORY",
        help="print every route in one category instead of the summary",
    )
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


def header_inventory(header_root: Path) -> dict:
    """Collects every public identity the canonical headers declare."""
    functions: dict[str, dict] = {}
    handles: dict[str, str] = {}
    scalars: dict[str, str] = {}
    structs: dict[str, str] = {}
    callbacks: dict[str, str] = {}
    constants: dict[str, str] = {}
    for path in sorted(header_root.glob("*.h")):
        text = path.read_text(encoding="utf-8")
        for match in FUNCTION.finditer(text):
            parameters = match.group(2).strip()
            functions[match.group(1)] = {
                "header": path.name,
                "arity": 0
                if parameters in {"", "void"}
                else len(split_parameters(parameters)),
            }
        for pattern, sink in (
            (HANDLE_TYPEDEF, handles),
            (SCALAR_TYPEDEF, scalars),
            (STRUCT_TYPEDEF, structs),
            (CALLBACK_TYPEDEF, callbacks),
        ):
            for match in pattern.finditer(text):
                sink[match.group(1)] = path.name
        for match in DEFINE.finditer(text):
            constants.setdefault(match.group(1), path.name)
    return {
        "functions": functions,
        "handles": handles,
        "scalars": scalars,
        "structs": structs,
        "callbacks": callbacks,
        "constants": constants,
    }


def rust_sys_inventory() -> dict:
    """Collects every identity the reviewed cna-sys slice declares."""
    source = CNA_SYS.read_text(encoding="utf-8")
    return {
        "functions": set(re.findall(r"pub type (cna_[A-Za-z0-9_]+)_fn\s*=", source)),
        "types": set(re.findall(r"pub (?:type|struct|enum) ([A-Za-z0-9_]+)", source)),
        "constants": set(re.findall(r"pub const ([A-Za-z0-9_]+)\s*:", source)),
    }


def matches(rule: dict, name: str, header: str) -> bool:
    if "header" in rule and rule["header"] != header:
        return False
    if "headers" in rule and header not in rule["headers"]:
        return False
    for pattern in rule.get("names", []):
        if fnmatch.fnmatchcase(name, pattern):
            return True
    return "names" not in rule


def classify(functions: dict[str, dict], bound: set[str], rules: dict) -> dict:
    """Assigns exactly one category to every canonical route."""
    result = {}
    for name, info in sorted(functions.items()):
        if name in bound:
            result[name] = {
                "category": "RUST_SYS_BOUND",
                "rationale": "declared in the reviewed cna-sys slice",
                "rule": "measured",
                "header": info["header"],
            }
            continue
        override = rules["overrides"].get(name)
        if override is not None:
            result[name] = {
                "category": override["category"],
                "rationale": override["rationale"],
                "rule": "override",
                "header": info["header"],
            }
            continue
        for index, rule in enumerate(rules["rules"]):
            if matches(rule, name, info["header"]):
                result[name] = {
                    "category": rule["category"],
                    "rationale": rule["rationale"],
                    "rule": rule.get("id", f"rule[{index}]"),
                    "header": info["header"],
                }
                break
        else:
            result[name] = {
                "category": "UNMAPPED_REQUIRES_REVIEW",
                "rationale": "no classification rule matches this canonical route",
                "rule": "none",
                "header": info["header"],
            }
    return result


def main() -> int:
    args = arguments()
    if not args.cna_root:
        raise ValueError("CNA_ROOT/--cna-root is required")
    header_root = Path(args.cna_root) / "modules/c-api/include/CNA/C"
    if not header_root.is_dir():
        raise FileNotFoundError(f"canonical header directory is missing: {header_root}")

    headers = header_inventory(header_root)
    rust = rust_sys_inventory()
    rules = json.loads(CLASSIFICATION.read_text(encoding="utf-8"))
    for rule in rules["rules"] + list(rules["overrides"].values()):
        if rule["category"] not in CATEGORIES:
            raise ValueError(f"unknown classification category: {rule['category']}")

    routes = classify(headers["functions"], rust["functions"], rules)
    if args.list:
        for name, value in routes.items():
            if value["category"] == args.list:
                print(f"{value['header']:28} {name}")
        return 0

    counts = {category: 0 for category in CATEGORIES}
    for value in routes.values():
        counts[value["category"]] += 1

    declared_not_in_headers = sorted(rust["functions"] - set(headers["functions"]))
    unused_rules = sorted(
        rule.get("id", "")
        for rule in rules["rules"]
        if rule.get("id") and not any(v["rule"] == rule.get("id") for v in routes.values())
    )
    # An override is fiction once its route stops existing, and dead weight once
    # the route is bound: a bound route is measured, never classified by rule.
    stale_overrides = sorted(
        name
        for name in rules["overrides"]
        if name not in headers["functions"] or name in rust["functions"]
    )

    report = {
        "schemaVersion": 1,
        "canonicalFunctions": len(headers["functions"]),
        "canonicalHandles": len(headers["handles"]),
        "canonicalScalars": len(headers["scalars"]),
        "canonicalStructs": len(headers["structs"]),
        "canonicalCallbacks": len(headers["callbacks"]),
        "canonicalConstants": len(headers["constants"]),
        "canonicalHeaders": len({v["header"] for v in headers["functions"].values()}),
        "rustSysFunctions": len(rust["functions"]),
        "categories": counts,
        "byHeader": {
            header: {
                "total": sum(1 for v in routes.values() if v["header"] == header),
                "bound": sum(
                    1
                    for v in routes.values()
                    if v["header"] == header and v["category"] == "RUST_SYS_BOUND"
                ),
            }
            for header in sorted({v["header"] for v in routes.values()})
        },
        "declaredButNotInHeaders": declared_not_in_headers,
        "unusedRules": unused_rules,
        "staleOverrides": stale_overrides,
        "unmapped": sorted(
            name
            for name, value in routes.items()
            if value["category"] == "UNMAPPED_REQUIRES_REVIEW"
        ),
    }
    text = json.dumps(report, indent=2, sort_keys=True) + "\n"
    if args.output:
        Path(args.output).write_text(text, encoding="utf-8")
    print(text, end="")
    failures = (
        counts["UNMAPPED_REQUIRES_REVIEW"]
        + len(declared_not_in_headers)
        + len(unused_rules)
        + len(stale_overrides)
    )
    return 0 if args.report_only or failures == 0 else 1


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (ValueError, OSError, json.JSONDecodeError) as error:
        print(f"C API inventory: {error}", file=sys.stderr)
        raise SystemExit(2)
