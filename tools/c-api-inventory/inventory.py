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

sys.path.insert(0, str(Path(__file__).resolve().parent))
import reachability  # noqa: E402  (path is set immediately above)

ROOT = Path(__file__).resolve().parents[2]
CLASSIFICATION = ROOT / "tools/c-api-inventory/classification.json"
CNA_SYS = ROOT / "crates/cna-sys/src/lib.rs"

# Two axes, two closed sets.
#
# CATEGORIES answers "why does this route exist, and which part of the
# projection owns it".  BINDING_STATUSES answers "does Rust bind it, and if
# not, why not".  Keeping them apart is the point: a purpose is not a binding
# decision, and the old single axis let "RUST_SYS_BOUND" erase a route's
# purpose while leaving every unbound route with no binding decision at all.
#
# Adding to either set is a deliberate project decision recorded in
# docs/c-api-classification.md.
CATEGORIES = (
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

BINDING_STATUSES = (
    # Measured, never declared: the route is in the reviewed cna-sys slice.
    "BOUND",
    # Rust reaches the same capability another way, or the route should not
    # exist in a safe Rust API.  Needs a reason naming the Rust surface.
    "DELIBERATE_NON_BINDING",
    # A CNA defect stops it.  Needs a finding id.
    "BLOCKED_UPSTREAM",
    # No renderer available here can run it.
    "BLOCKED_RENDERER",
    # This platform cannot reach it.
    "BLOCKED_PLATFORM",
    # No such device is attached, and CNA ships no test backend for it.
    "BLOCKED_HARDWARE",
    # It needs an asset this project may not carry.
    "BLOCKED_ASSET",
    # Real work, deliberately not now.  Needs a backlog id.
    "DEFERRED_TRACKED",
    # Real work, reachable today, nobody has done it.  The gate fails on these.
    "ACTIONABLE_LOCAL",
    # Nobody has decided.  The gate fails on these too.
    "UNREVIEWED",
)

# A status that stops the gate is not allowed to be a bare assertion.
NEEDS_TASK = ("BLOCKED_UPSTREAM", "DEFERRED_TRACKED")

# Why a bound route legitimately has no safe caller.  A closed set, so
# "unused" and "review later" are not available as answers.
UNREACHABLE_OUTCOMES = (
    # The family is acquired as one table so a library missing any of it fails
    # at load rather than at first use, and this member is deliberately absent
    # from the safe API.
    "ATOMIC_TABLE_MEMBER",
    # Safe Rust implements the behaviour itself, more faithfully than the C
    # route does.  Must name the Rust that does it in ``rustEvidence``.
    "IMPLEMENTED_IN_SAFE_RUST",
    # The route mutates state XNA's object model freezes, so exposing it would
    # contradict the semantics the projection exists to reproduce.
    "MUTATOR_XNA_DOES_NOT_HAVE",
    # A CNA defect stops the safe layer from calling it.  Names a finding.
    "BLOCKED_UPSTREAM",
    # No renderer, platform, device or asset available here can drive it.
    "BLOCKED_ENVIRONMENT",
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


def backlog_text():
    """The backlog, for checking that a named task is actually written down."""
    path = Path(__file__).resolve().parents[2] / "docs" / "backlog.md"
    try:
        return path.read_text(encoding="utf-8")
    except OSError:
        return ""


def rust_source() -> str:
    """The whole safe Rust layer as one string, for evidence checks."""
    return "\n".join(
        path.read_text(encoding="utf-8")
        for path in sorted((ROOT / "crates/cna/src").rglob("*.rs"))
    )


def missing_rust_evidence(rules: dict, source: str) -> list[str]:
    """Rule evidence that names Rust code which is not there.

    A deliberate non-binding whose reason is "Rust already does this" is only
    as good as the claim, and prose cannot be checked.  A rule may therefore
    carry ``rustEvidence``: exact Rust symbols the claim rests on.  This finds
    the ones that have since been renamed or removed, so a rule cannot go on
    pointing at code nobody has.
    """
    missing = []
    for rule in rules.get("binding", {}).get("rules", []):
        for symbol in rule.get("rustEvidence", []):
            if symbol not in source:
                missing.append(f"{rule.get('id', '?')}: {symbol}")
    return missing


def safe_layer_reachability() -> dict:
    """Which acquired routes a safe caller can reach, from the call graph.

    ``BOUND`` is measured from ``cna-sys``; this answers the different question
    of whether anything outside ``native/`` can arrive at the route.  The work
    is in ``reachability.py``, which walks the crate rather than pattern-
    matching names -- the first version of this check compared a route's name
    to Rust identifiers and could not have worked, because the field holding a
    route's pointer is not named after it.
    """
    return reachability.analyse(ROOT / "crates/cna/src")


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
    """Assigns exactly one purpose to every canonical route.

    Purpose is independent of whether Rust binds the route: a bound route still
    has to say which part of the projection owns it, and an unbound one still
    has a purpose.  ``classify_binding`` answers the other question.
    """
    result = {}
    for name, info in sorted(functions.items()):
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


def classify_binding(functions: dict[str, dict], bound: set[str], rules: dict) -> dict:
    """Answers, for every canonical route, whether Rust binds it and why not.

    ``BOUND`` is measured.  Everything else has to be stated in
    classification.json with a reason, and a task when the reason is a block or
    a deferral, so no route can quietly sit in limbo.

    A declared *block* outranks the measurement, because the two answer
    different questions.  A blocked family is often declared in ``cna-sys``
    anyway -- so its reproducer can drive it, and so the ABI gate keeps checking
    its prototypes -- and counting those declarations as BOUND would put routes
    nobody can use on the same line of the scoreboard as routes that work.  The
    declaration is still recorded, in ``declaredInCnaSys``, so a reader can see
    that the symbols are there.
    """
    binding = rules.get("binding", {"rules": [], "overrides": {}})
    outranks_measurement = {
        "BLOCKED_UPSTREAM", "BLOCKED_RENDERER", "BLOCKED_PLATFORM",
        "BLOCKED_HARDWARE", "BLOCKED_ASSET",
    }

    def stated(name: str, header: str) -> dict | None:
        override = binding.get("overrides", {}).get(name)
        if override is not None:
            return dict(override, rule="override")
        for index, rule in enumerate(binding.get("rules", [])):
            if matches(rule, name, header):
                return dict(rule, rule=rule.get("id", f"binding[{index}]"))
        return None

    # First match wins, so the order in classification.json is load-bearing:
    # blocks first, then rules naming exact routes, then rules matching a
    # *shape* of route across every family. A family that is blocked upstream is
    # blocked as a unit -- singling out, say, its struct initialiser as
    # "deliberately not bound" would be a distinction without a difference,
    # because nobody can use it either way.

    result = {}
    for name, info in sorted(functions.items()):
        header = info["header"]
        if name in bound:
            blocked = stated(name, header)
            if blocked is not None and blocked["status"] in outranks_measurement:
                result[name] = {
                    "status": blocked["status"],
                    "reason": blocked["reason"],
                    "evidence": blocked.get("evidence", ""),
                    "task": blocked.get("task"),
                    "rule": blocked["rule"],
                    "declaredInCnaSys": True,
                    "header": header,
                }
                continue
            result[name] = {
                "status": "BOUND",
                "reason": "declared in the reviewed cna-sys slice",
                "evidence": "tools/native-abi/bindings.json",
                "rule": "measured",
                "header": header,
            }
            continue
        override = binding.get("overrides", {}).get(name)
        if override is not None:
            result[name] = {
                "status": override["status"],
                "reason": override["reason"],
                "evidence": override.get("evidence", ""),
                "task": override.get("task"),
                "rule": "override",
                "header": header,
            }
            continue
        for index, rule in enumerate(binding.get("rules", [])):
            if matches(rule, name, header):
                result[name] = {
                    "status": rule["status"],
                    "reason": rule["reason"],
                    "evidence": rule.get("evidence", ""),
                    "task": rule.get("task"),
                    "rule": rule.get("id", f"binding[{index}]"),
                    "header": header,
                }
                break
        else:
            result[name] = {
                "status": "UNREVIEWED",
                "reason": "no binding rule states why Rust does not bind this route",
                "evidence": "",
                "task": None,
                "rule": "none",
                "header": header,
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
    rust_layer = rust_source()
    for rule in rules["rules"] + list(rules["overrides"].values()):
        if rule["category"] not in CATEGORIES:
            raise ValueError(f"unknown classification category: {rule['category']}")

    for rule in rules.get("binding", {}).get("rules", []) + list(
        rules.get("binding", {}).get("overrides", {}).values()
    ):
        if rule["status"] not in BINDING_STATUSES:
            raise ValueError(f"unknown binding status: {rule['status']}")
        if rule["status"] == "BOUND":
            raise ValueError("BOUND is measured from cna-sys, never declared")
        if not rule.get("reason"):
            raise ValueError(f"binding rule {rule.get('id', '?')} states no reason")
        if rule["status"] in NEEDS_TASK and not rule.get("task"):
            raise ValueError(
                f"binding rule {rule.get('id', '?')} is {rule['status']} with no owning task"
            )

    routes = classify(headers["functions"], rust["functions"], rules)
    binding = classify_binding(headers["functions"], rust["functions"], rules)

    # Bound but with no safe call site.  Each one needs a stated reason, or the
    # binding is dead weight nobody can use.
    graph = safe_layer_reachability()
    justified = rules.get("bindingUnreachable", {})
    unreachable = sorted(
        name
        for name in headers["functions"]
        if binding[name]["status"] == "BOUND" and name not in graph["reachable"]
    )
    unjustified = [name for name in unreachable if name not in justified]
    # A justification is only worth having while it is still true.  One that
    # names a route the safe layer has since started calling, a route that has
    # stopped being bound, or a route the headers no longer declare is a claim
    # about code that is not there any more, and saying so is the whole point
    # of writing the reason down.
    stale_justifications = sorted(
        name
        for name in justified
        if name not in headers["functions"]
        or binding.get(name, {}).get("status") != "BOUND"
        or name in graph["reachable"]
    )
    justifications_naming_absent_rust = []
    for name, entry in sorted(justified.items()):
        for symbol in entry.get("rustEvidence", []):
            if symbol not in rust_layer:
                justifications_naming_absent_rust.append(f"{name}: {symbol}")
    unjustified_outcomes = sorted(
        {
            entry.get("outcome", "")
            for entry in justified.values()
            if entry.get("outcome") not in UNREACHABLE_OUTCOMES
        }
    )
    if args.list:
        wanted = args.list
        for name, value in routes.items():
            if value["category"] == wanted or binding[name]["status"] == wanted:
                print(f"{value['header']:28} {name}")
        return 0

    counts = {category: 0 for category in CATEGORIES}
    for value in routes.values():
        counts[value["category"]] += 1
    binding_counts = {status: 0 for status in BINDING_STATUSES}
    for value in binding.values():
        binding_counts[value["status"]] += 1

    declared_not_in_headers = sorted(rust["functions"] - set(headers["functions"]))
    used = {v["rule"] for v in routes.values()} | {v["rule"] for v in binding.values()}
    unused_rules = sorted(
        rule.get("id", "")
        for rule in rules["rules"] + rules.get("binding", {}).get("rules", [])
        if rule.get("id") and rule["id"] not in used
    )
    # An override is fiction once its route stops existing, and dead weight once
    # the route is bound: a bound route is measured, never classified by rule.
    stale_overrides = sorted(
        name
        for name in rules["overrides"]
        if name not in headers["functions"] or name in rust["functions"]
    )

    # A deferral names a task so the route has an owner, and the census already
    # refuses a rule that names none. That is half the promise: a task nobody
    # wrote down owns nothing. RUST-EXT-016 was in exactly that state -- named
    # by a rule, absent from the backlog -- so the check is now both halves.
    undocumented_tasks = sorted(
        {
            value["task"]
            for value in binding.values()
            if value.get("task") and value["task"] not in backlog_text()
        }
    )

    absent_rust_evidence = missing_rust_evidence(rules, rust_layer)

    # A defect that does not change a route's status still has to name real
    # routes, or a finding can quietly stop pointing at anything.
    known_defects = []
    stale_defect_names = []
    for defect in rules.get("knownDefects", []):
        named = defect.get("names", [])
        missing = [name for name in named if name not in headers["functions"]]
        stale_defect_names.extend(missing)
        known_defects.append({
            "id": defect["id"],
            "routes": len(named),
            "summary": defect["summary"],
            "evidence": defect.get("evidence", ""),
            "statuses": sorted({
                binding[name]["status"] for name in named if name in binding
            }),
        })

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
        "bindingStatuses": binding_counts,
        "actionableLocal": sorted(
            name for name, value in binding.items() if value["status"] == "ACTIONABLE_LOCAL"
        ),
        "unreviewedBinding": sorted(
            name for name, value in binding.items() if value["status"] == "UNREVIEWED"
        ),
        "byHeader": {
            header: {
                "total": sum(1 for v in routes.values() if v["header"] == header),
                "bound": sum(
                    1
                    for name, v in routes.items()
                    if v["header"] == header and binding[name]["status"] == "BOUND"
                ),
                "actionableLocal": sum(
                    1
                    for name, v in routes.items()
                    if v["header"] == header
                    and binding[name]["status"] == "ACTIONABLE_LOCAL"
                ),
                "unreviewed": sum(
                    1
                    for name, v in routes.items()
                    if v["header"] == header and binding[name]["status"] == "UNREVIEWED"
                ),
            }
            for header in sorted({v["header"] for v in routes.values()})
        },
        "declaredButNotInHeaders": declared_not_in_headers,
        "unusedRules": unused_rules,
        "staleOverrides": stale_overrides,
        "undocumentedTasks": undocumented_tasks,
        # Reported, not gated. A bound route with no safe call site is not
        # automatically wrong -- the ABI slice deliberately declares the whole
        # of a family so a library missing one fails at load rather than at
        # first use, and read-only projections legitimately leave the C
        # mutators uncalled. Turning this into a gate before those families
        # have family-level justifications would only teach people to bypass
        # it. RUST-CENSUS-002 owns working the list down.
        "knownDefects": known_defects,
        "rulesNamingAbsentRustCode": absent_rust_evidence,
        "boundWithoutSafeCallSite": len(unreachable),
        "boundWithoutSafeCallSiteJustified": len(justified),
        "boundWithoutSafeCallSiteUnjustified": unjustified,
        "boundWithoutSafeCallSiteOutcomes": {
            outcome: sum(
                1 for entry in justified.values() if entry.get("outcome") == outcome
            )
            for outcome in UNREACHABLE_OUTCOMES
        },
        "staleUnreachableJustifications": stale_justifications,
        "justificationsNamingAbsentRustCode": justifications_naming_absent_rust,
        "unknownUnreachableOutcomes": unjustified_outcomes,
        "nativeFunctionsWalked": graph["nativeFunctions"],
        "nativeFunctionsReached": graph["visitedNativeFunctions"],
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
        len(absent_rust_evidence)
        + len(stale_defect_names)
        + 
        counts["UNMAPPED_REQUIRES_REVIEW"]
        + len(declared_not_in_headers)
        + len(unused_rules)
        + len(stale_overrides)
        + len(undocumented_tasks)
        # The two that make the census mean something: a route nobody has made
        # a binding decision about, and one everybody agrees is doable and
        # nobody has done.
        + binding_counts["UNREVIEWED"]
        + binding_counts["ACTIONABLE_LOCAL"]
        # And the third, which RUST-CENSUS-002 added: a route Rust binds, keeps
        # resolving at load, and no safe caller can reach -- with nobody
        # willing to say why.  The raw count is *not* gated: an atomic table
        # and a read-only projection both leave routes legitimately uncalled.
        # What is gated is the absence of an explanation.
        + len(unjustified)
        + len(stale_justifications)
        + len(justifications_naming_absent_rust)
        + len(unjustified_outcomes)
    )
    return 0 if args.report_only or failures == 0 else 1


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (ValueError, OSError, json.JSONDecodeError) as error:
        print(f"C API inventory: {error}", file=sys.stderr)
        raise SystemExit(2)
