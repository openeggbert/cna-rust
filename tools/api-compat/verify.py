#!/usr/bin/env python3
"""Strict compiler-metadata baseline for the XNA-to-Rust projection."""

from __future__ import annotations

import argparse
import collections
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[2]
PROFILE = ROOT / "tools/api-compat/profiles/xna40-windows-runtime.json"
RULES = ROOT / "tools/api-compat/mapping-rules.json"
EXTRACTOR = ROOT / "tools/api-compat/extractor/XnaContractExtractor.cs"


def arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--reference-dir", default=os.environ.get("XNA_REFERENCE_PATH") or os.environ.get("XNA_REFERENCE_DIR"))
    parser.add_argument("--rustdoc")
    parser.add_argument("--output")
    parser.add_argument("--report-only", action="store_true")
    parser.add_argument("--leak-only", action="store_true")
    return parser.parse_args()


def run(command: list[str], **kwargs: object) -> None:
    subprocess.run(command, check=True, **kwargs)


def validate_references(directory: Path, profile: dict) -> None:
    for name in profile["referenceAssemblies"]:
        path = directory / name
        if not path.is_file():
            raise FileNotFoundError(f"XNA reference assembly is missing: {path}")
        actual = hashlib.sha256(path.read_bytes()).hexdigest()
        expected = profile["referenceSha256"][name]
        if actual != expected:
            raise ValueError(f"XNA reference SHA-256 mismatch for {name}: expected {expected}, got {actual}")


def extract_reference(directory: Path, profile: dict, temporary: Path) -> dict:
    executable = temporary / "xna-contract.exe"
    output = temporary / "xna-contract.json"
    run(["mcs", "-r:System.Web.Extensions", f"-out:{executable}", str(EXTRACTOR)])
    run(["mono", str(executable), str(directory), str(output), *profile["referenceAssemblies"]])
    return json.loads(output.read_text(encoding="utf-8"))


def generate_rustdoc(temporary: Path) -> Path:
    environment = dict(os.environ)
    environment["RUSTC_BOOTSTRAP"] = "1"
    run([
        "cargo", "rustdoc", "-p", "cna-rust", "--lib", "--",
        "-Z", "unstable-options", "--output-format", "json"
    ], cwd=ROOT, env=environment)
    source = ROOT / "target/doc/cna.json"
    destination = temporary / "cna.json"
    shutil.copyfile(source, destination)
    return destination


def inner_kind(item: dict) -> str:
    return next(iter(item.get("inner", {})), "unknown")


def rust_kind(item: dict) -> str:
    kind = inner_kind(item)
    return {"struct": "struct", "enum": "enum", "trait": "trait"}.get(kind, kind)


def framework_module(index: dict[str, dict]) -> str:
    for identifier, item in index.items():
        if item.get("name") == "Framework" and inner_kind(item) == "module":
            span = item.get("span") or {}
            if span.get("filename") == "crates/cna/src/lib.rs":
                return identifier
    raise ValueError("rustdoc JSON has no cna::Microsoft::Xna::Framework module")


def actual_contract(rustdoc: dict) -> dict[str, dict]:
    index = rustdoc["index"]
    found: dict[str, dict] = {}

    def walk(module_id: str, path: str) -> None:
        module = index[module_id]["inner"]["module"]
        for child_id in module["items"]:
            child = index[child_id]
            kind = inner_kind(child)
            if kind == "module":
                name = child["name"]
                walk(child_id, f"{path}::{name}")
            elif kind == "import":
                imported = child["inner"]["import"]
                target_id = imported.get("id")
                if target_id and target_id in index:
                    target = index[target_id]
                    target_kind = rust_kind(target)
                    if target_kind in {"struct", "enum", "trait"}:
                        name = imported["name"]
                        found[f"{path}::{name}"] = read_rust_type(target_id, target, index, target_kind)
            elif rust_kind(child) in {"struct", "enum", "trait"}:
                found[f"{path}::{child['name']}"] = read_rust_type(child_id, child, index, rust_kind(child))

    walk(framework_module(index), "cna::Microsoft::Xna::Framework")
    return found


def read_rust_type(identifier: str, item: dict, index: dict[str, dict], kind: str) -> dict:
    names: set[str] = set()
    unsafe_members: list[str] = []
    public_items: list[dict] = [item]
    body = item["inner"][kind]
    if kind == "struct":
        structure_kind = body["kind"]
        plain = structure_kind.get("plain") if isinstance(structure_kind, dict) else None
        if plain:
            for field_id in plain["fields"]:
                field = index.get(field_id)
                if field and field.get("name"):
                    names.add(field["name"])
                    public_items.append(field)
    elif kind == "enum":
        for variant_id in body["variants"]:
            names.add(index[variant_id]["name"])
    elif kind == "trait":
        for member_id in body["items"]:
            member = index[member_id]
            if member.get("name"):
                names.add(member["name"])
                public_items.append(member)
            if inner_kind(member) == "function" and member["inner"]["function"]["header"]["unsafe"]:
                unsafe_members.append(member.get("name") or member_id)

    for implementation_id in body.get("impls", []):
        implementation = index.get(implementation_id)
        if not implementation or inner_kind(implementation) != "impl":
            continue
        implementation_body = implementation["inner"]["impl"]
        if implementation_body.get("trait") is not None:
            continue
        for member_id in implementation_body["items"]:
            member = index.get(member_id)
            if not member or not member.get("name"):
                continue
            member_kind = inner_kind(member)
            if member_kind in {"function", "assoc_const", "assoc_type"}:
                names.add(member["name"])
                public_items.append(member)
            if member_kind == "function" and member["inner"]["function"]["header"]["unsafe"]:
                unsafe_members.append(member["name"])

    serialized = json.dumps(public_items, sort_keys=True)
    return {
        "id": identifier,
        "kind": kind,
        "members": names,
        "unsafeMembers": unsafe_members,
        "internalLeak": "cna_sys" in serialized or "CNA_" in serialized,
        "rawHandleLeak": "CNA_Handle" in serialized or '"raw_pointer"' in serialized,
    }


def pascal_identifier(value: str) -> str:
    """Uppercase an identifier without destroying its authoritative camel case."""
    return "".join(part[:1].upper() + part[1:] for part in value.split("_") if part)


def snake_identifier(value: str) -> str:
    words = re.sub(r"([a-z0-9])([A-Z])", r"\1_\2", value).replace("-", "_")
    return words.lower()


def type_token(value: str) -> str:
    simple = value.replace("&", "ByRef").replace("[]", "Array")
    simple = simple.rsplit(".", 1)[-1].replace("`1", "").replace("`2", "")
    return "".join(character for character in simple if character.isalnum())


def mapped_member_names(contract_type: dict) -> set[str]:
    result: set[str] = set()
    constructors = sorted(
        (m for m in contract_type["members"] if m["kind"] == "constructor"),
        key=lambda m: (len(m["parameters"]), json.dumps(m, sort_keys=True)),
    )
    constructor_names: list[tuple[str, dict]] = []
    for position, constructor in enumerate(constructors):
        if position == 0:
            constructor_names.append(("new", constructor))
        else:
            parameters = "_and_".join(snake_identifier(p["name"] or "value") for p in constructor["parameters"])
            constructor_names.append(("from_" + parameters, constructor))
    constructor_counts = collections.Counter(name for name, _ in constructor_names)
    for name, constructor in constructor_names:
        if constructor_counts[name] > 1:
            types = "_and_".join(snake_identifier(type_token(p["type"])) for p in constructor["parameters"])
            name += "_as_" + types
        result.add(name)

    methods = collections.defaultdict(list)
    for member in contract_type["members"]:
        kind = member["kind"]
        if kind == "field":
            result.add(member["name"])
        elif kind == "property":
            name = member["name"]
            if member.get("get"):
                result.add(name)
            if member.get("set"):
                result.add("Set" + name)
        elif kind == "event":
            result.add("Add" + member["name"] + "Handler")
            result.add("Remove" + member["name"] + "Handler")
        elif kind == "method":
            # CLR operator methods are represented by Rust operator traits and
            # are intentionally not projected as extra inherent methods.
            if not member["name"].startswith("op_"):
                methods[member["name"]].append(member)

    for name, overloads in methods.items():
        result.add(name)
        alternatives: list[tuple[str, dict]] = []
        for overload in overloads[1:]:
            suffix = "And".join(pascal_identifier(p["name"] or "Value") for p in overload["parameters"])
            alternatives.append((name + "With" + (suffix or "NoArguments"), overload))
        alternative_counts = collections.Counter(value for value, _ in alternatives)
        for projected, overload in alternatives:
            if alternative_counts[projected] > 1:
                types = "And".join(type_token(p["type"]) for p in overload["parameters"])
                projected += "As" + types
            result.add(projected)
    if any("System.IDisposable" in value for value in contract_type.get("interfaces", [])):
        result.add("Dispose")
    return result


def expected_contract(reference: dict, rules: dict) -> dict[str, dict]:
    expected: dict[str, dict] = {}
    overrides = rules["typeKindOverrides"]
    renames = rules["genericTypeRenames"]
    for contract_type in reference["types"]:
        clr_name = renames.get(contract_type["name"], contract_type["name"]).replace("+", "::")
        path = "cna::" + clr_name.replace(".", "::").replace("`1", "").replace("`2", "")
        clr_kind = contract_type["kind"]
        kind = overrides.get(contract_type["name"])
        if kind is None:
            kind = "trait" if clr_kind in {"interface", "delegate"} else "enum" if clr_kind == "enum" and not contract_type.get("flags") else "struct"
        expected[path] = {"kind": kind, "members": mapped_member_names(contract_type)}
    for synthetic in rules["syntheticTypes"]:
        path = "cna::" + synthetic["name"].replace(".", "::")
        expected[path] = {"kind": synthetic["kind"], "members": set(synthetic.get("members", []))}
    return expected


def compare(expected: dict[str, dict], actual: dict[str, dict]) -> list[dict]:
    findings: list[dict] = []
    for name in sorted(expected.keys() - actual.keys()):
        findings.append({"code": "MISSING_TYPE", "subject": name})
    for name in sorted(actual.keys() - expected.keys()):
        findings.append({"code": "UNEXPECTED_TYPE", "subject": name})
    for name in sorted(expected.keys() & actual.keys()):
        wanted, present = expected[name], actual[name]
        if wanted["kind"] != present["kind"]:
            findings.append({"code": "TYPE_KIND_MAPPING_MISMATCH", "subject": name, "expected": wanted["kind"], "actual": present["kind"]})
        for member in sorted(wanted["members"] - present["members"]):
            findings.append({"code": "MISSING_MEMBER", "subject": f"{name}::{member}"})
        for member in sorted(present["members"] - wanted["members"]):
            findings.append({"code": "UNEXPECTED_MEMBER", "subject": f"{name}::{member}"})
        if present["internalLeak"]:
            findings.append({"code": "INTERNAL_TYPE_LEAK", "subject": name})
        if present["rawHandleLeak"]:
            findings.append({"code": "RAW_HANDLE_LEAK", "subject": name})
        for member in present["unsafeMembers"]:
            findings.append({"code": "UNSAFE_PUBLIC_API", "subject": f"{name}::{member}"})
    return findings


def main() -> int:
    args = arguments()
    profile = json.loads(PROFILE.read_text(encoding="utf-8"))
    rules = json.loads(RULES.read_text(encoding="utf-8"))
    if rules["allowlist"]:
        raise ValueError("mapping allowlist must remain empty")
    with tempfile.TemporaryDirectory(prefix="cna-rust-api-") as name:
        temporary = Path(name)
        rustdoc_path = Path(args.rustdoc) if args.rustdoc else generate_rustdoc(temporary)
        actual = actual_contract(json.loads(rustdoc_path.read_text(encoding="utf-8")))
        if args.leak_only:
            expected = actual
            findings = [
                {"code": "INTERNAL_TYPE_LEAK", "subject": name}
                for name, value in actual.items() if value["internalLeak"]
            ]
            findings.extend(
                {"code": "RAW_HANDLE_LEAK", "subject": name}
                for name, value in actual.items() if value["rawHandleLeak"]
            )
            findings.extend(
                {"code": "UNSAFE_PUBLIC_API", "subject": f"{name}::{member}"}
                for name, value in actual.items() for member in value["unsafeMembers"]
            )
        else:
            if not args.reference_dir:
                raise ValueError("XNA_REFERENCE_PATH/--reference-dir is required")
            reference_dir = Path(args.reference_dir)
            validate_references(reference_dir, profile)
            reference = extract_reference(reference_dir, profile, temporary)
            expected = expected_contract(reference, rules)
            findings = compare(expected, actual)
        counts = collections.Counter(value["code"] for value in findings)
        report = {
            "schemaVersion": 1,
            "profile": profile["name"],
            "referenceTypes": None if args.leak_only else len(reference["types"]),
            "referenceMembers": None if args.leak_only else sum(len(t["members"]) for t in reference["types"]),
            "expectedRustTypes": len(expected),
            "actualRustTypes": len(actual),
            "totalDiagnostics": len(findings),
            "counts": dict(sorted(counts.items())),
            "allowlistEntries": len(rules["allowlist"]),
            "unmeasuredCategories": [
                "BASE_PROJECTION_MISMATCH", "TRAIT_MISMATCH", "PARAMETER_MISMATCH",
                "RETURN_TYPE_MISMATCH", "GENERIC_MISMATCH", "REF_OUT_MAPPING_MISMATCH",
                "ENUM_VALUE_MISMATCH", "DELEGATE_EVENT_MAPPING_MISMATCH", "DISPOSAL_MAPPING_MISMATCH"
            ],
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
    except (FileNotFoundError, ValueError, subprocess.CalledProcessError) as error:
        print(f"api verifier: {error}", file=sys.stderr)
        raise SystemExit(2)
