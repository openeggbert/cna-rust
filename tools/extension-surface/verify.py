#!/usr/bin/env python3
"""Gates CNA's own public surface, which the strict XNA verifier deliberately cannot.

`tools/api-compat/verify.py` answers one question: does
`cna::Microsoft::Xna::Framework` contain exactly what Microsoft XNA declares?
Its answer is now zero, and it stays zero by *removing* things -- so on its own
it cannot tell a CNA member that moved behind an extension trait from one that
was deleted. This answers the other half.

Three gates, over compiler-produced rustdoc JSON rather than source regex:

* **The migration manifest.** Every member `RUST-SURFACE-001` moved is named
  with the trait it moved to. Each must still be declared by a publicly
  reachable trait, with the same receiver shape and arity, implemented for the
  strict XNA type it came from -- and must *not* be a public inherent member of
  that type any more, which is what would put the strict verifier back into
  diagnostics.

* **Public reachability.** Computed by walking public modules and public `use`
  items from the crate root, not by trusting rustdoc's `paths`: a re-export
  through a private module -- which is how every `cna::extensions` type is
  published -- has no canonical path there at all.

* **Nameable public signatures.** A public item whose signature mentions a
  crate type that no public path reaches is unusable: a caller can invoke it
  and cannot name what comes back, or cannot build what it takes. That is how
  `PresentationMode` shipped -- `pub` in a private module, re-exported nowhere,
  answered by a public method -- and the strict verifier does not see it,
  because `INTERNAL_TYPE_LEAK` looks for `cna_sys` and `CNA_` identities rather
  than for absence.
"""

from __future__ import annotations

import argparse
import collections
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[2]
MANIFEST = ROOT / "tools/extension-surface/migrated.json"


def arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--rustdoc")
    parser.add_argument("--manifest", default=MANIFEST)
    parser.add_argument("--output")
    parser.add_argument(
        "--write-manifest",
        action="store_true",
        help="record the current surface as the expected one instead of checking it",
    )
    return parser.parse_args()


def generate_rustdoc(temporary: Path) -> Path:
    environment = dict(os.environ)
    environment["RUSTC_BOOTSTRAP"] = "1"
    subprocess.run(
        ["cargo", "rustdoc", "-p", "cna-rust", "--lib", "--",
         "-Z", "unstable-options", "--output-format", "json"],
        cwd=ROOT, env=environment, check=True,
    )
    destination = temporary / "cna.json"
    shutil.copyfile(ROOT / "target/doc/cna.json", destination)
    return destination


def inner_kind(item: dict) -> str:
    kind = next(iter(item.get("inner", {})), "unknown")
    # Rust 1.85 renamed the rustdoc JSON item tag from `import` to `use`.
    return "import" if kind == "use" else kind


def use_body(item: dict) -> dict:
    inner = item["inner"]
    return inner.get("use") or inner.get("import") or {}


def public_paths(index: dict, root: str) -> dict[str, set[str]]:
    """Every crate item a consumer can name, and the paths that name it.

    Walks public modules and public `use` items outward from the crate root.
    A private module is not a dead end: `mod game;` is private and
    `cna::extensions::graphics_device_ext` re-exports out of it, which is
    exactly the shape rustdoc's own `paths` does not record.
    """
    reached: dict[str, set[str]] = collections.defaultdict(set)
    pending = [(root, ("cna",))]
    seen: set[tuple[str, tuple[str, ...]]] = set()
    while pending:
        identifier, path = pending.pop()
        if (identifier, path) in seen:
            continue
        seen.add((identifier, path))
        item = index.get(identifier)
        if item is None or inner_kind(item) != "module":
            continue
        for child in item["inner"]["module"]["items"]:
            child_item = index.get(str(child))
            if child_item is None or child_item.get("visibility") != "public":
                continue
            kind = inner_kind(child_item)
            if kind == "import":
                body = use_body(child_item)
                target = str(body["id"]) if body.get("id") is not None else None
                if target is None:
                    continue
                if body.get("is_glob"):
                    if target in index and inner_kind(index[target]) == "module":
                        pending.append((target, path))
                    continue
                if target in index and inner_kind(index[target]) == "module":
                    pending.append((target, path + (body["name"],)))
                else:
                    reached[target].add("::".join(path + (body["name"],)))
            elif child_item.get("name"):
                name = child_item["name"]
                if kind == "module":
                    pending.append((str(child), path + (name,)))
                else:
                    reached[str(child)].add("::".join(path + (name,)))
    return reached


def signature(item: dict) -> dict:
    body = item["inner"]["function"]
    return body.get("sig") or body.get("decl") or {}


def shape(item: dict) -> dict:
    """The part of a signature a rename or a reorder would change."""
    sig = signature(item)
    inputs = [name for name, _ in sig.get("inputs", [])]
    return {
        "receiver": "self" if inputs[:1] == ["self"] else "associated",
        "parameters": inputs,
        "returns": render(sig.get("output")),
    }


def render(value: dict | None) -> str:
    """A stable spelling of a type, enough to notice one changing."""
    if value is None:
        return "()"
    if "primitive" in value:
        return value["primitive"]
    if "generic" in value:
        return value["generic"]
    if "resolved_path" in value:
        path = value["resolved_path"]
        name = (path.get("name") or "?").split("::")[-1]
        arguments = (path.get("args") or {}).get("angle_bracketed", {}).get("args", [])
        rendered = [render(argument["type"]) for argument in arguments if "type" in argument]
        return name + ("<" + ",".join(rendered) + ">" if rendered else "")
    if "borrowed_ref" in value:
        reference = value["borrowed_ref"]
        mutable = reference.get("mutable", reference.get("is_mutable", False))
        return ("&mut " if mutable else "&") + render(reference["type"])
    if "slice" in value:
        return "[" + render(value["slice"]) + "]"
    if "array" in value:
        return "[" + render(value["array"]["type"]) + ";" + str(value["array"]["len"]) + "]"
    if "tuple" in value:
        parts = [render(part) for part in value["tuple"]]
        return "(" + ",".join(parts) + ")"
    if "impl_trait" in value:
        names = []
        for bound in value["impl_trait"]:
            if "trait_bound" in bound:
                names.append((bound["trait_bound"]["trait"].get("name") or "?").split("::")[-1])
            elif "outlives" in bound:
                names.append(bound["outlives"])
        return "impl " + "+".join(names)
    if "raw_pointer" in value:
        pointer = value["raw_pointer"]
        mutable = pointer.get("mutable", pointer.get("is_mutable", False))
        return ("*mut " if mutable else "*const ") + render(pointer["type"])
    if "qualified_path" in value:
        return render(value.get("self_type")) + "::" + value["qualified_path"].get("name", "?")
    return "?" + next(iter(value), "unknown")


def collect_type_ids(value, out: set[str]) -> None:
    if isinstance(value, list):
        for entry in value:
            collect_type_ids(entry, out)
        return
    if not isinstance(value, dict):
        return
    if "resolved_path" in value and value["resolved_path"].get("id") is not None:
        out.add(str(value["resolved_path"]["id"]))
    for entry in value.values():
        collect_type_ids(entry, out)


def struct_fields(index: dict, item: dict) -> list[str]:
    kind = item["inner"]["struct"].get("kind")
    if not isinstance(kind, dict):
        return []
    if "plain" in kind:
        return [str(value) for value in kind["plain"].get("fields", [])]
    if "tuple" in kind:
        return [str(value) for value in kind["tuple"] or [] if value is not None]
    return []


def unnameable(index: dict, reachable: dict[str, set[str]]) -> list[dict]:
    """Public signatures that mention a crate type no public path reaches."""
    targets = set(reachable)
    for item in index.values():
        if inner_kind(item) != "impl":
            continue
        body = item["inner"]["impl"]
        owner = ((body.get("for") or {}).get("resolved_path") or {}).get("id")
        trait = (body.get("trait") or {}).get("id")
        if str(owner) not in reachable and str(trait) not in reachable:
            continue
        for member in body.get("items", []):
            entry = index.get(str(member))
            if entry is None:
                continue
            if entry.get("visibility") == "public" or trait is not None:
                targets.add(str(member))
    findings: dict[tuple[str, str], set[str]] = collections.defaultdict(set)
    for identifier in targets:
        item = index.get(identifier)
        if item is None:
            continue
        kind = inner_kind(item)
        references: set[str] = set()
        if kind == "function":
            collect_type_ids(signature(item), references)
        elif kind == "trait":
            for member in item["inner"]["trait"].get("items", []):
                entry = index.get(str(member))
                if entry is not None and inner_kind(entry) == "function":
                    collect_type_ids(signature(entry), references)
        elif kind == "struct":
            for field in struct_fields(index, item):
                entry = index.get(field)
                if entry is not None and entry.get("visibility") == "public":
                    collect_type_ids(entry["inner"], references)
        elif kind == "enum":
            for variant in item["inner"]["enum"].get("variants", []):
                entry = index.get(str(variant))
                if entry is not None:
                    collect_type_ids(entry["inner"], references)
        elif kind in ("type_alias", "constant", "static"):
            collect_type_ids(item["inner"], references)
        else:
            continue
        for reference in references:
            target = index.get(reference)
            # Absent from the index means a foreign crate's item, which this
            # crate does not publish and is not answerable for.
            if target is None or reference in reachable:
                continue
            name = target.get("name") or "?"
            where = (target.get("span") or {}).get("filename") or "?"
            findings[(name, where)].add(item.get("name") or identifier)
    return [
        {"type": name, "declaredIn": where, "usedBy": sorted(users)}
        for (name, where), users in sorted(findings.items())
    ]


def measure(document: dict, manifest: dict) -> dict:
    index = document["index"]
    root = str(document["root"])
    reachable = public_paths(index, root)
    by_path = {path: identifier for identifier, paths in reachable.items() for path in paths}

    # Every trait implementation, by the strict type it is for.
    implemented: dict[tuple[str, str], set[str]] = collections.defaultdict(set)
    inherent: dict[str, set[str]] = collections.defaultdict(set)
    for item in index.values():
        if inner_kind(item) != "impl":
            continue
        body = item["inner"]["impl"]
        owner = ((body.get("for") or {}).get("resolved_path") or {}).get("id")
        if owner is None:
            continue
        owner = str(owner)
        trait = body.get("trait")
        for member in body.get("items", []):
            entry = index.get(str(member))
            if entry is None or not entry.get("name"):
                continue
            if trait is None:
                if entry.get("visibility") == "public":
                    inherent[owner].add(entry["name"])
            else:
                implemented[(owner, entry["name"])].add(str(trait["id"]))

    findings: list[dict] = []
    for member in manifest["members"]:
        strict_path = member["strictType"]
        strict_id = by_path.get(strict_path)
        if strict_id is None:
            findings.append({"code": "STRICT_TYPE_UNREACHABLE", "type": strict_path})
            continue
        trait_path = member["trait"]
        trait_id = by_path.get(trait_path)
        if trait_id is None:
            findings.append({
                "code": "EXTENSION_TRAIT_NOT_PUBLIC",
                "type": strict_path, "member": member["member"], "trait": trait_path,
            })
            continue
        trait_item = index[trait_id]
        declarations = {
            index[str(value)]["name"]: index[str(value)]
            for value in trait_item["inner"]["trait"].get("items", [])
            if str(value) in index and index[str(value)].get("name")
        }
        declared = declarations.get(member["member"])
        if declared is None or inner_kind(declared) != "function":
            findings.append({
                "code": "MEMBER_MISSING_FROM_TRAIT",
                "type": strict_path, "member": member["member"], "trait": trait_path,
            })
        else:
            actual = shape(declared)
            if actual != member["shape"]:
                findings.append({
                    "code": "EXTENSION_SIGNATURE_MISMATCH",
                    "type": strict_path, "member": member["member"], "trait": trait_path,
                    "expected": member["shape"], "actual": actual,
                })
        if trait_id not in implemented.get((strict_id, member["member"]), set()):
            findings.append({
                "code": "EXTENSION_TRAIT_NOT_IMPLEMENTED",
                "type": strict_path, "member": member["member"], "trait": trait_path,
            })
        if member["member"] in inherent.get(strict_id, set()):
            findings.append({
                "code": "MEMBER_STILL_INHERENT",
                "type": strict_path, "member": member["member"],
            })

    for moved in manifest["types"]:
        identifier = by_path.get(moved["extensionPath"])
        if identifier is None:
            findings.append({"code": "EXTENSION_TYPE_NOT_PUBLIC", "type": moved["extensionPath"]})
            continue
        for forbidden in moved["absentFrom"]:
            if forbidden in by_path:
                findings.append({
                    "code": "EXTENSION_TYPE_IN_STRICT_NAMESPACE",
                    "type": moved["extensionPath"], "path": forbidden,
                })

    holes = unnameable(index, reachable)
    findings.extend(
        {"code": "UNNAMEABLE_PUBLIC_TYPE", "type": hole["type"],
         "declaredIn": hole["declaredIn"], "usedBy": hole["usedBy"]}
        for hole in holes
    )
    return {
        "schemaVersion": 1,
        "publiclyNameableItems": len(reachable),
        "migratedMembers": len(manifest["members"]),
        "migratedTypes": len(manifest["types"]),
        "extensionTraits": len({member["trait"] for member in manifest["members"]}),
        "totalDiagnostics": len(findings),
        "counts": dict(sorted(collections.Counter(value["code"] for value in findings).items())),
        "findings": findings,
    }


def write_manifest(document: dict, path: Path) -> int:
    index = document["index"]
    reachable = public_paths(index, str(document["root"]))
    strict_prefix = "cna::Microsoft::Xna::Framework::"

    def preferred(paths: set[str]) -> str:
        # A strict type is usually re-exported at the crate root as well, and
        # the manifest is about the strict hierarchy, so name it that way.
        strict = sorted(path for path in paths if path.startswith(strict_prefix))
        return strict[0] if strict else sorted(paths)[0]

    by_id = {identifier: preferred(paths) for identifier, paths in reachable.items()}
    members = []
    for item in index.values():
        if inner_kind(item) != "impl":
            continue
        body = item["inner"]["impl"]
        trait = body.get("trait")
        owner = ((body.get("for") or {}).get("resolved_path") or {}).get("id")
        if trait is None or owner is None:
            continue
        owner_path = by_id.get(str(owner), "")
        trait_path = by_id.get(str(trait["id"]), "")
        if not owner_path.startswith(strict_prefix) or not trait_path.startswith("cna::extensions::"):
            continue
        trait_item = index[str(trait["id"])]
        for member in body.get("items", []):
            entry = index.get(str(member))
            if entry is None or inner_kind(entry) != "function":
                continue
            declared = None
            for value in trait_item["inner"]["trait"].get("items", []):
                candidate = index.get(str(value))
                if candidate is not None and candidate.get("name") == entry.get("name"):
                    declared = candidate
            if declared is None:
                continue
            members.append({
                "strictType": owner_path,
                "member": entry["name"],
                "trait": trait_path,
                "shape": shape(declared),
            })
    members.sort(key=lambda value: (value["strictType"], value["member"]))
    path.write_text(json.dumps({"schemaVersion": 1, "members": members, "types": []}, indent=2) + "\n")
    return len(members)


def main() -> int:
    args = arguments()
    with tempfile.TemporaryDirectory(prefix="cna-rust-extension-") as name:
        rustdoc = Path(args.rustdoc) if args.rustdoc else generate_rustdoc(Path(name))
        document = json.loads(rustdoc.read_text(encoding="utf-8"))
        if args.write_manifest:
            print(f"recorded {write_manifest(document, Path(args.manifest))} members")
            return 0
        manifest = json.loads(Path(args.manifest).read_text(encoding="utf-8"))
        report = measure(document, manifest)
        text = json.dumps(report, indent=2, sort_keys=True) + "\n"
        if args.output:
            Path(args.output).write_text(text, encoding="utf-8")
        print(text, end="")
        return 0 if not report["findings"] else 1


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (FileNotFoundError, ValueError, KeyError, subprocess.CalledProcessError) as error:
        print(f"extension surface: {error}", file=sys.stderr)
        raise SystemExit(2)
