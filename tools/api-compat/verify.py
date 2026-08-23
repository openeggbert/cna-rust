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


FRAMEWORK_PREFIX = "cna::Microsoft::Xna::Framework::"


def relative_rust_path(path: str) -> str:
    return path[len(FRAMEWORK_PREFIX):] if path.startswith(FRAMEWORK_PREFIX) else path


def rust_generic_arguments(value: dict, owner: str, paths: dict[str, str], index: dict[str, dict]) -> list[str]:
    arguments = value.get("args") or {}
    angle = arguments.get("angle_bracketed") if isinstance(arguments, dict) else None
    result = []
    for argument in (angle or {}).get("args", []):
        if "type" in argument:
            result.append(rust_type_name(argument["type"], owner, paths, index))
        elif "const" in argument:
            result.append(str(argument["const"]))
    return result


def rust_resolved_path(value: dict, owner: str, paths: dict[str, str], index: dict[str, dict]) -> str:
    identifier = value.get("id")
    if identifier == owner:
        name = "Self"
    elif identifier in paths:
        name = relative_rust_path(paths[identifier])
    else:
        name = value.get("name") or "?"
    arguments = rust_generic_arguments(value, owner, paths, index)
    return name + ("<" + ",".join(arguments) + ">" if arguments else "")


def rust_type_name(value: dict | None, owner: str, paths: dict[str, str], index: dict[str, dict]) -> str:
    if value is None:
        return "()"
    if "primitive" in value:
        return value["primitive"]
    if "generic" in value:
        return value["generic"]
    if "resolved_path" in value:
        return rust_resolved_path(value["resolved_path"], owner, paths, index)
    if "borrowed_ref" in value:
        reference = value["borrowed_ref"]
        prefix = "&mut " if reference.get("mutable") else "&"
        return prefix + rust_type_name(reference["type"], owner, paths, index)
    if "raw_pointer" in value:
        pointer = value["raw_pointer"]
        prefix = "*mut " if pointer.get("mutable") else "*const "
        return prefix + rust_type_name(pointer["type"], owner, paths, index)
    if "slice" in value:
        return "[" + rust_type_name(value["slice"], owner, paths, index) + "]"
    if "array" in value:
        array = value["array"]
        return "[" + rust_type_name(array["type"], owner, paths, index) + ";" + str(array["len"]) + "]"
    if "tuple" in value:
        values = [rust_type_name(item, owner, paths, index) for item in value["tuple"]]
        return "()" if not values else "(" + ",".join(values) + ("," if len(values) == 1 else "") + ")"
    if "impl_trait" in value:
        return "impl " + "+".join(rust_bound_name(bound, owner, paths, index) for bound in value["impl_trait"])
    if "dyn_trait" in value:
        traits = value["dyn_trait"].get("traits", [])
        return "dyn " + "+".join(
            rust_resolved_path(bound["trait"], owner, paths, index) for bound in traits
        )
    if "qualified_path" in value:
        qualified = value["qualified_path"]
        return rust_type_name(qualified.get("self_type"), owner, paths, index) + "::" + qualified.get("name", "?")
    return "?" + next(iter(value), "unknown")


def rust_bound_name(value: dict, owner: str, paths: dict[str, str], index: dict[str, dict]) -> str:
    if "trait_bound" in value:
        return rust_resolved_path(value["trait_bound"]["trait"], owner, paths, index)
    if "outlives" in value:
        return value["outlives"]
    return "?"


def rust_generics(value: dict, owner: str, paths: dict[str, str], index: dict[str, dict]) -> list[dict]:
    result = []
    for parameter in value.get("params", []):
        kind = parameter.get("kind", {})
        if "type" not in kind:
            continue
        bounds = sorted(
            rust_bound_name(bound, owner, paths, index)
            for bound in kind["type"].get("bounds", [])
        )
        result.append({"name": parameter["name"], "bounds": bounds})
    return result


def rust_member(identifier: str, item: dict, owner: str, paths: dict[str, str], index: dict[str, dict]) -> dict:
    kind = inner_kind(item)
    descriptor = {"name": item.get("name"), "kind": kind}
    if kind == "function":
        function = item["inner"]["function"]
        descriptor.update({
            "parameters": [
                {"name": name, "type": rust_type_name(type_value, owner, paths, index)}
                for name, type_value in function["decl"].get("inputs", [])
            ],
            "returnType": rust_type_name(function["decl"].get("output"), owner, paths, index),
            "generics": rust_generics(function.get("generics", {}), owner, paths, index),
            "unsafe": bool(function["header"].get("unsafe")),
        })
    elif kind == "assoc_const":
        constant = item["inner"]["assoc_const"]
        descriptor.update({
            "type": rust_type_name(constant.get("type"), owner, paths, index),
            "value": constant.get("default"),
        })
    elif kind == "assoc_type":
        descriptor["type"] = "associated"
    elif kind == "struct_field":
        descriptor.update({"kind": "field", "type": rust_type_name(item["inner"][kind], owner, paths, index)})
    elif kind == "variant":
        variant = item["inner"]["variant"]
        descriptor.update({
            "kind": "variant",
            "value": (variant.get("discriminant") or {}).get("value"),
        })
    return descriptor


def trait_name(value: dict, owner: str, paths: dict[str, str], index: dict[str, dict]) -> str:
    return rust_resolved_path(value, owner, paths, index)


def actual_contract(rustdoc: dict) -> dict[str, dict]:
    index = rustdoc["index"]
    located: dict[str, tuple[str, dict, str]] = {}

    def walk(module_id: str, path: str) -> None:
        module = index[module_id]["inner"]["module"]
        for child_id in module["items"]:
            child = index[child_id]
            kind = inner_kind(child)
            if kind == "module":
                walk(child_id, f"{path}::{child['name']}")
            elif kind == "import":
                imported = child["inner"]["import"]
                target_id = imported.get("id")
                if target_id and target_id in index:
                    target = index[target_id]
                    target_kind = rust_kind(target)
                    if target_kind in {"struct", "enum", "trait"}:
                        located[f"{path}::{imported['name']}"] = (target_id, target, target_kind)
            elif rust_kind(child) in {"struct", "enum", "trait"}:
                located[f"{path}::{child['name']}"] = (child_id, child, rust_kind(child))

    walk(framework_module(index), "cna::Microsoft::Xna::Framework")
    paths = {identifier: path for path, (identifier, _, _) in located.items()}
    return {
        path: read_rust_type(identifier, item, index, kind, paths)
        for path, (identifier, item, kind) in located.items()
    }


def read_rust_type(
    identifier: str,
    item: dict,
    index: dict[str, dict],
    kind: str,
    paths: dict[str, str],
) -> dict:
    members: dict[str, dict] = {}
    trait_members: dict[str, dict] = {}
    traits: set[str] = set()
    unsafe_members: list[str] = []
    public_items: list[dict] = [item]
    body = item["inner"][kind]
    generics = rust_generics(body.get("generics", {}), identifier, paths, index)

    if kind == "struct":
        structure_kind = body["kind"]
        plain = structure_kind.get("plain") if isinstance(structure_kind, dict) else None
        for field_id in (plain or {}).get("fields", []):
            field = index.get(field_id)
            if field and field.get("name"):
                members[field["name"]] = rust_member(field_id, field, identifier, paths, index)
                public_items.append(field)
    elif kind == "enum":
        for variant_id in body["variants"]:
            variant = index[variant_id]
            members[variant["name"]] = rust_member(variant_id, variant, identifier, paths, index)
    elif kind == "trait":
        for bound in body.get("bounds", []):
            name = rust_bound_name(bound, identifier, paths, index)
            traits.update({name, name.split("<", 1)[0]})
        for member_id in body["items"]:
            member = index[member_id]
            if member.get("name"):
                descriptor = rust_member(member_id, member, identifier, paths, index)
                members[member["name"]] = descriptor
                public_items.append(member)
                if descriptor.get("unsafe"):
                    unsafe_members.append(member["name"])

    implementation_ids = body.get("impls", [])
    for implementation_id in implementation_ids:
        implementation = index.get(implementation_id)
        if not implementation or inner_kind(implementation) != "impl":
            continue
        implementation_body = implementation["inner"]["impl"]
        implemented_trait = implementation_body.get("trait")
        if implemented_trait is not None:
            name = trait_name(implemented_trait, identifier, paths, index)
            traits.update({name, name.split("<", 1)[0]})
            for member_id in implementation_body.get("items", []):
                member = index.get(member_id)
                if member and member.get("name"):
                    descriptor = rust_member(member_id, member, identifier, paths, index)
                    trait_members[member["name"]] = descriptor
                    public_items.append(member)
                    if descriptor.get("unsafe"):
                        unsafe_members.append(member["name"])
            continue
        for member_id in implementation_body.get("items", []):
            member = index.get(member_id)
            if not member or not member.get("name"):
                continue
            member_kind = inner_kind(member)
            if member_kind in {"function", "assoc_const", "assoc_type"}:
                descriptor = rust_member(member_id, member, identifier, paths, index)
                members[member["name"]] = descriptor
                public_items.append(member)
                if descriptor.get("unsafe"):
                    unsafe_members.append(member["name"])

    serialized = json.dumps(public_items, sort_keys=True)
    reprs = []
    for attribute in item.get("attrs", []):
        match = re.fullmatch(r"#\[repr\(([^)]+)\)\]", attribute.replace(" ", ""))
        if match:
            reprs.extend(value.strip() for value in match.group(1).split(","))
    return {
        "id": identifier,
        "kind": kind,
        "members": members,
        "traitMembers": trait_members,
        "traits": traits,
        "generics": generics,
        "repr": sorted(reprs),
        "drop": "Drop" in traits,
        "unsafeMembers": sorted(set(unsafe_members)),
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


def split_generic_arguments(value: str) -> tuple[str, list[str]]:
    if "[" not in value or not value.endswith("]"):
        return value, []
    base, body = value.split("[", 1)
    body = body[:-1]
    result, start, depth = [], 0, 0
    for position, character in enumerate(body):
        if character == "[":
            depth += 1
        elif character == "]":
            depth -= 1
        elif character == "," and depth == 0:
            result.append(body[start:position])
            start = position + 1
    result.append(body[start:])
    return base, result


def mapped_type_path(value: str, rules: dict) -> str:
    base, arguments = split_generic_arguments(value)
    if base in rules["primitiveTypes"]:
        return rules["primitiveTypes"][base]
    renamed = rules["genericTypeRenames"].get(base, base).replace("+", "::")
    if renamed.startswith("Microsoft.Xna.Framework."):
        name = renamed[len("Microsoft.Xna.Framework."):].replace(".", "::")
    else:
        name = renamed.rsplit(".", 1)[-1]
    name = re.sub(r"`\d+$", "", name)
    if arguments:
        name += "<" + ",".join(mapped_type_path(argument, rules) for argument in arguments) + ">"
    return name


def generic_parameter_name(value: str, type_generics: list[dict], method_generics: list[dict]) -> str | None:
    if value.startswith("!!"):
        position = int(value[2:])
        return method_generics[position]["name"] if position < len(method_generics) else f"M{position}"
    if value.startswith("!"):
        position = int(value[1:])
        return type_generics[position]["name"] if position < len(type_generics) else f"T{position}"
    return None


def clr_value_type(
    value: str,
    rules: dict,
    current: dict,
    reference_types: dict[str, dict],
    method_generics: list[dict],
) -> str:
    parameter = generic_parameter_name(value, current.get("genericParameters", []), method_generics)
    if parameter is not None:
        return parameter
    if value.endswith("&"):
        return clr_value_type(value[:-1], rules, current, reference_types, method_generics)
    if value.endswith("[]"):
        return "Vec<" + clr_value_type(value[:-2], rules, current, reference_types, method_generics) + ">"
    base, arguments = split_generic_arguments(value)
    owned_collection = rules.get("ownedCollectionProjections", {}).get(base)
    if owned_collection and len(arguments) == 1:
        element = clr_value_type(arguments[0], rules, current, reference_types, method_generics)
        return owned_collection.replace("T", element)
    if base == "System.Nullable`1" and arguments:
        return "Option<" + clr_value_type(arguments[0], rules, current, reference_types, method_generics) + ">"
    if base in rules["primitiveTypes"]:
        return rules["primitiveTypes"][base]
    if base == "System.Object":
        return "Box<dyn Any>"
    if base == "System.IO.Stream":
        return rules["streamProjection"]["genericName"]
    if base.startswith("System.EventHandler"):
        if arguments:
            payload = clr_value_type(arguments[0], rules, current, reference_types, method_generics)
            if payload != "EventArgs":
                return "Box<dyn EventHandler<" + payload + ">>"
        return "Box<dyn EventHandler>"
    if base.startswith("System."):
        name = base.rsplit(".", 1)[-1]
        name = re.sub(r"`\d+$", "", name)
    else:
        name = mapped_type_path(base, rules)
        if base == current["name"]:
            name = "Self"
    if arguments:
        name += "<" + ",".join(
            clr_value_type(argument, rules, current, reference_types, method_generics)
            for argument in arguments
        ) + ">"
    return name


def clr_parameter_type(
    parameter: dict,
    member: dict,
    rules: dict,
    current: dict,
    reference_types: dict[str, dict],
    method_generics: list[dict],
) -> str:
    override = rules.get("parameterTypeOverrides", {}).get(
        current["name"] + "::" + member["name"] + "::" + (parameter.get("name") or "value")
    )
    if override:
        return override
    value = parameter["type"]
    by_reference = value.endswith("&")
    if by_reference:
        value = value[:-1]
    if value == "System.Object":
        return "&dyn Any"
    if value == "System.Exception":
        return "&dyn Error"
    base, arguments = split_generic_arguments(value)
    collection = rules.get("collectionProjections", {}).get(base)
    if collection and len(arguments) == 1:
        element = clr_value_type(arguments[0], rules, current, reference_types, method_generics)
        return collection.replace("T", element)
    if value == "System.String":
        mapped = "str"
        borrowed = True
    elif value == "System.IO.Stream":
        output = any(
            rule_matches(pattern, current["name"], member["name"])
            for pattern in rules.get("outputStreamMembers", [])
        )
        projection = rules["outputStreamProjection"] if output else rules["streamProjection"]
        return projection["type"]
    elif value.endswith("[]"):
        element = clr_value_type(value[:-2], rules, current, reference_types, method_generics)
        mutable = (
            parameter.get("out")
            or "destination" in parameter.get("name", "").lower()
            or member.get("name", "").startswith("Get")
            or member.get("name") == "CopyTo"
        )
        return ("&mut [" if mutable else "&[") + element + "]"
    else:
        mapped = clr_value_type(value, rules, current, reference_types, method_generics)
        contract_name = split_generic_arguments(value)[0]
        contract = reference_types.get(contract_name)
        projected_kind = rules.get("typeKindOverrides", {}).get(
            contract_name, contract.get("kind") if contract else None
        )
        if projected_kind in {"interface", "trait", "delegate"}:
            mapped = "dyn " + mapped
        borrowed = bool(contract and contract["kind"] in {"class", "interface", "delegate"}) or value == "System.Object"
    if by_reference:
        # A CLR `ref` parameter is an alias through which the callee may write;
        # only metadata `in` byrefs can be projected as a shared borrow.  XNA
        # contains observable examples (notably Matrix.CreateReflection) that
        # mutate a non-`out` ref argument while producing another result.
        return ("&" if parameter.get("in") and not parameter.get("out") else "&mut ") + mapped
    if borrowed:
        return "&" + mapped
    return mapped


def projected_generic_parameters(current: dict, member: dict, rules: dict) -> list[dict]:
    result = []
    for parameter in member.get("genericParameters", []):
        bounds = []
        for special in parameter.get("specialConstraints", []):
            bounds.extend(rules["genericConstraintProjection"].get(special, []))
        bounds.extend(type_token(value) for value in parameter.get("typeConstraints", []))
        result.append({"name": parameter["name"], "bounds": sorted(set(bounds))})
    if any(parameter["type"].rstrip("&") == "System.IO.Stream" for parameter in member.get("parameters", [])):
        output = any(
            rule_matches(pattern, current["name"], member["name"])
            for pattern in rules.get("outputStreamMembers", [])
        )
        stream = rules["outputStreamProjection"] if output else rules["streamProjection"]
        if not any(parameter["name"] == stream["genericName"] for parameter in result):
            result.append({"name": stream["genericName"], "bounds": [stream["bound"]]})
    return result


def rule_matches(pattern: str, type_name: str, member_name: str) -> bool:
    wanted_type, wanted_member = pattern.rsplit("::", 1)
    return (wanted_type == "*" or wanted_type == type_name) and (wanted_member == "*" or wanted_member == member_name)


def is_fallible(type_name: str, member_name: str, rules: dict) -> bool:
    if any(rule_matches(pattern, type_name, member_name) for pattern in rules.get("fallibleExclusions", [])):
        return False
    return any(rule_matches(pattern, type_name, member_name) for pattern in rules.get("fallibleMembers", []))


def mutable_receiver(contract_type: dict, member: dict, rules: dict) -> bool:
    if any(rule_matches(pattern, contract_type["name"], member["name"])
           for pattern in rules.get("sharedReceiverMembers", [])):
        return False
    if member.get("mapping") == "property-setter":
        return True
    if contract_type["name"] == "Microsoft.Xna.Framework.Game" and member["name"] in {
        "Initialize", "LoadContent", "Update", "Draw", "UnloadContent", "OnExiting"
    }:
        return True
    if any(rule_matches(pattern, contract_type["name"], member["name"]) for pattern in rules.get("mutableReceiverMembers", [])):
        return True
    return contract_type["kind"] == "struct" and member.get("returnType") == "System.Void"


def projected_return_type(
    value: str | None,
    contract_type: dict,
    member: dict,
    rules: dict,
    reference_types: dict[str, dict],
    method_generics: list[dict],
    property_borrow: bool = False,
    projected_name: str | None = None,
) -> str:
    projected_member_name = projected_name or member["name"]
    fallible = is_fallible(contract_type["name"], projected_member_name, rules)
    if projected_member_name != member["name"]:
        fallible = fallible or is_fallible(
            contract_type["name"], member["name"], rules
        )
    override = rules.get("returnTypeOverrides", {}).get(
        contract_type["name"] + "::" + projected_member_name
    )
    if override:
        mapped = override
        if fallible:
            mapped = "Result<" + mapped + ">"
        return mapped
    mapped = clr_value_type(value or "System.Void", rules, contract_type, reference_types, method_generics)
    if property_borrow and value:
        referenced = reference_types.get(split_generic_arguments(value)[0])
        if referenced and referenced["kind"] in {"class", "interface", "delegate"}:
            mapped = "&" + mapped
    if fallible:
        mapped = "Result<" + mapped + ">"
    return mapped


def game_lifecycle_signature(contract_type: dict, member: dict) -> dict | None:
    if contract_type["name"] != "Microsoft.Xna.Framework.Game" or member["name"] not in {
        "Initialize", "LoadContent", "Update", "Draw", "UnloadContent", "OnExiting"
    }:
        return None
    parameters = [
        {"name": "self", "type": "&mut Self"},
        {"name": "game", "type": "&mut GameContext"},
    ]
    if member["name"] in {"Update", "Draw"}:
        parameters.append({"name": "time", "type": "&GameTime"})
    return {"parameters": parameters, "returnType": "Result<()>", "generics": [], "refOut": []}


def callable_descriptor(
    contract_type: dict,
    member: dict,
    projected_name: str,
    rules: dict,
    reference_types: dict[str, dict],
    origin: str,
    overload: int = 0,
) -> dict:
    lifecycle = game_lifecycle_signature(contract_type, member)
    if lifecycle is not None:
        signature = lifecycle
    else:
        generics = projected_generic_parameters(contract_type, member, rules)
        generic_overrides = rules.get("genericBoundOverrides", {}).get(
            contract_type["name"] + "::" + projected_name, {}
        )
        for generic in generics:
            if generic["name"] in generic_overrides:
                generic["bounds"] = sorted(generic_overrides[generic["name"]])
        parameters = []
        if member["kind"] != "constructor" and not member.get("static"):
            parameters.append({
                "name": "self",
                "type": "&mut Self" if mutable_receiver(contract_type, member, rules) else "&Self",
            })
        for pattern, injected in rules.get("contextInjectedMembers", {}).items():
            if rule_matches(pattern, contract_type["name"], member["name"]):
                parameters.extend(dict(value) for value in injected)
        ref_out = []
        for parameter in member.get("parameters", []):
            position = len(parameters)
            parameters.append({
                "name": parameter.get("name") or "value",
                "type": clr_parameter_type(parameter, member, rules, contract_type, reference_types, generics),
            })
            if parameter.get("ref") or parameter["type"].endswith("&"):
                ref_out.append(position)
        projected_parameter_overrides = rules.get("projectedParameterTypeOverrides", {})
        for parameter in parameters:
            override = projected_parameter_overrides.get(
                contract_type["name"] + "::" + projected_name + "::" + parameter["name"]
            )
            if override:
                parameter["type"] = override
        if member["kind"] == "constructor":
            returned = "Self"
            if is_fallible(contract_type["name"], ".ctor", rules):
                returned = "Result<Self>"
        else:
            returned = projected_return_type(
                member.get("returnType"), contract_type, member, rules, reference_types, generics,
                projected_name=projected_name
            )
        signature = {"parameters": parameters, "returnType": returned, "generics": generics, "refOut": ref_out}
    signature.update({
        "name": projected_name,
        "kind": "function",
        "origin": origin,
        "overload": overload,
        "clrMember": member["name"],
    })
    return signature


def const_representable(value: str, reference_types: dict[str, dict], rules: dict) -> bool:
    base = split_generic_arguments(value)[0]
    return base in rules["primitiveTypes"] or bool(
        reference_types.get(base) and reference_types[base]["kind"] in {"struct", "enum"}
    )


def mapped_members(contract_type: dict, rules: dict, reference_types: dict[str, dict]) -> dict[str, dict]:
    result: dict[str, dict] = {}
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
    for position, (name, constructor) in enumerate(constructor_names):
        if constructor_counts[name] > 1:
            types = "_and_".join(snake_identifier(type_token(p["type"])) for p in constructor["parameters"])
            name += "_as_" + types
        result[name] = callable_descriptor(
            contract_type, constructor, name, rules, reference_types, "constructor", position
        )

    methods = collections.defaultdict(list)
    for member in contract_type["members"]:
        kind = member["kind"]
        if kind == "field":
            if member["name"] == "value__":
                continue
            if contract_type["kind"] == "enum":
                result[member["name"]] = {
                    "name": member["name"],
                    "kind": "assoc_const" if contract_type.get("flags") else "variant",
                    "origin": "enum-value",
                    "type": "Self", "value": member.get("value"), "overload": 0,
                }
            else:
                mapped = clr_value_type(member["type"], rules, contract_type, reference_types, [])
                static_getter = rules.get("staticFieldGetterOverrides", {}).get(
                    contract_type["name"] + "::" + member["name"]
                )
                if member.get("static") and static_getter:
                    result[member["name"]] = {
                        "name": member["name"], "kind": "function", "origin": "field",
                        "parameters": [], "returnType": static_getter,
                        "generics": [], "refOut": [], "overload": 0,
                    }
                else:
                    result[member["name"]] = {
                        "name": member["name"],
                        "kind": "assoc_const" if member.get("static") else "field",
                        "origin": "field", "type": mapped, "overload": 0,
                    }
        elif kind == "property":
            name = member["name"]
            retained_object_type = rules.get("retainedObjectProperties", {}).get(
                contract_type["name"] + "::" + name
            )
            if member.get("get"):
                dynamic_static = any(
                    rule_matches(pattern, contract_type["name"], name)
                    for pattern in rules.get("dynamicStaticProperties", [])
                )
                if (member.get("static") and not member.get("set") and not dynamic_static
                        and const_representable(member["type"], reference_types, rules)):
                    result[name] = {
                        "name": name, "kind": "assoc_const", "origin": "property-getter",
                        "type": clr_value_type(member["type"], rules, contract_type, reference_types, []),
                        "overload": 0,
                    }
                else:
                    getter = {
                        "kind": "method", "name": name, "static": member.get("static", False),
                        "returnType": member["type"], "parameters": member.get("parameters", []),
                        "genericParameters": [], "mapping": "property-getter",
                    }
                    descriptor = callable_descriptor(
                        contract_type, getter, name, rules, reference_types, "property-getter"
                    )
                    descriptor["returnType"] = projected_return_type(
                        member["type"], contract_type, getter, rules, reference_types, [],
                        property_borrow=True, projected_name=name
                    )
                    if any(rule_matches(pattern, contract_type["name"], name)
                           for pattern in rules.get("ownedClassPropertyResults", [])):
                        descriptor["returnType"] = projected_return_type(
                            member["type"], contract_type, getter, rules, reference_types, [],
                            projected_name=name
                        )
                    if any(rule_matches(pattern, contract_type["name"], name)
                           for pattern in rules.get("optionalClassPropertyResults", [])):
                        descriptor["returnType"] = "Option<" + descriptor["returnType"] + ">"
                    if retained_object_type:
                        descriptor["returnType"] = retained_object_type
                    result[name] = descriptor
            if member.get("set"):
                setter_name = "Set" + name
                setter = {
                    "kind": "method", "name": name, "static": member.get("static", False),
                    "returnType": "System.Void",
                    "parameters": [*member.get("parameters", []), {
                        "name": "value", "type": member["type"], "ref": False, "out": False,
                    }],
                    "genericParameters": [], "mapping": "property-setter",
                }
                descriptor = callable_descriptor(
                    contract_type, setter, setter_name, rules, reference_types, "property-setter"
                )
                if retained_object_type:
                    descriptor["parameters"][-1]["type"] = retained_object_type
                result[setter_name] = descriptor
        elif kind == "event":
            # Event registries use interior mutability. This keeps subscription
            # available through shared identities such as `Game.Components`
            # and matches CLR reference-object mutation without requiring an
            # exclusive borrow of the object wrapper.
            receiver = [] if member.get("static") else [{"name": "self", "type": "&Self"}]
            handler = clr_value_type(member["type"], rules, contract_type, reference_types, [])
            add_name = "Add" + member["name"] + "Handler"
            remove_name = "Remove" + member["name"] + "Handler"
            result[add_name] = {
                "name": add_name, "kind": "function", "origin": "event", "overload": 0,
                "parameters": receiver + [{"name": "handler", "type": handler}],
                "returnType": rules["eventProjection"]["addReturn"], "generics": [], "refOut": [],
            }
            result[remove_name] = {
                "name": remove_name, "kind": "function", "origin": "event", "overload": 0,
                "parameters": receiver + [{"name": "registration", "type": rules["eventProjection"]["removeParameter"]}],
                "returnType": "bool", "generics": [], "refOut": [],
            }
        elif kind == "method":
            # CLR operator methods are represented by Rust operator traits and
            # are intentionally not projected as extra inherent methods.
            if not member["name"].startswith("op_"):
                methods[member["name"]].append(member)

    for name, overloads in methods.items():
        origin = "delegate" if contract_type["kind"] == "delegate" else "method"
        result[name] = callable_descriptor(
            contract_type, overloads[0], name, rules, reference_types, origin, 0
        )
        alternatives: list[tuple[str, dict]] = []
        for overload in overloads[1:]:
            suffix = "And".join(pascal_identifier(p["name"] or "Value") for p in overload["parameters"])
            alternatives.append((name + "With" + (suffix or "NoArguments"), overload))
        alternative_counts = collections.Counter(value for value, _ in alternatives)
        for projected, overload in alternatives:
            if alternative_counts[projected] > 1:
                types = "And".join(type_token(p["type"]) for p in overload["parameters"])
                projected += "As" + types
            result[projected] = callable_descriptor(
                contract_type, overload, projected, rules, reference_types, origin, overloads.index(overload)
            )
    return result


def expected_contract(reference: dict, rules: dict) -> dict[str, dict]:
    expected: dict[str, dict] = {}
    reference_types = {value["name"]: value for value in reference["types"]}
    overrides = rules["typeKindOverrides"]
    renames = rules["genericTypeRenames"]
    for contract_type in reference["types"]:
        clr_name = renames.get(contract_type["name"], contract_type["name"]).replace("+", "::")
        path = "cna::" + clr_name.replace(".", "::").replace("`1", "").replace("`2", "")
        clr_kind = contract_type["kind"]
        kind = overrides.get(contract_type["name"])
        if kind is None:
            kind = "trait" if clr_kind in {"interface", "delegate"} else "enum" if clr_kind == "enum" and not contract_type.get("flags") else "struct"
        expected[path] = {
            "kind": kind,
            "clrKind": clr_kind,
            "clrName": contract_type["name"],
            "members": mapped_members(contract_type, rules, reference_types),
            "flags": bool(contract_type.get("flags")),
            "underlyingType": rules["primitiveTypes"].get(contract_type.get("underlyingType")),
            "baseType": contract_type.get("baseType"),
            "interfaces": contract_type.get("directInterfaces", contract_type.get("interfaces", [])),
            "allInterfaces": contract_type.get("interfaces", []),
            "genericParameters": [
                {"name": value["name"], "bounds": sorted(
                    rules.get("typeGenericBoundOverrides", {})
                    .get(contract_type["name"], {})
                    .get(value["name"], [
                        bound
                        for special in value.get("specialConstraints", [])
                        for bound in rules["genericConstraintProjection"].get(special, [])
                    ])
                )}
                for value in contract_type.get("genericParameters", [])
            ],
            "operatorTraits": operator_traits(contract_type, rules, reference_types),
        }
    for synthetic in rules["syntheticTypes"]:
        path = "cna::" + synthetic["name"].replace(".", "::")
        members = {}
        for value in synthetic.get("members", []):
            descriptor = {"name": value, "kind": "unknown", "origin": "synthetic", "overload": 0} if isinstance(value, str) else dict(value)
            descriptor.setdefault("origin", "synthetic")
            descriptor.setdefault("overload", 0)
            if descriptor.get("kind") == "method":
                descriptor["kind"] = "function"
                parameters = []
                if descriptor.get("receiver"):
                    parameters.append({"name": "self", "type": descriptor.pop("receiver")})
                parameters.extend(descriptor.get("parameters", []))
                descriptor["parameters"] = parameters
                descriptor.setdefault("generics", [])
                descriptor.setdefault("refOut", [])
            members[descriptor["name"]] = descriptor
        expected[path] = {
            "kind": synthetic["kind"], "clrKind": "synthetic", "clrName": synthetic["name"],
            "members": members, "flags": False, "underlyingType": None, "baseType": None,
            "interfaces": [], "allInterfaces": [], "genericParameters": [], "operatorTraits": set(),
        }
    return expected


def probe_flag_values(expected: dict[str, dict], actual: dict[str, dict], temporary: Path) -> None:
    probes = []
    for type_name in sorted(expected.keys() & actual.keys()):
        wanted, present = expected[type_name], actual[type_name]
        representation = wanted.get("underlyingType")
        if not wanted.get("flags") or not representation:
            continue
        for member_name, member in sorted(wanted["members"].items()):
            actual_member = present["members"].get(member_name)
            if member.get("origin") == "enum-value" and actual_member and actual_member["kind"] == "assoc_const":
                probes.append((type_name, member_name, representation))
    if not probes:
        return

    project = temporary / "flag-value-probe"
    source = project / "src"
    source.mkdir(parents=True)
    (project / "Cargo.toml").write_text(
        "[package]\nname = \"cna-rust-flag-value-probe\"\nversion = \"0.0.0\"\nedition = \"2021\"\n"
        f"[dependencies]\ncna = {{ package = \"cna-rust\", path = {json.dumps(str(ROOT / 'crates/cna'))} }}\n",
        encoding="utf-8",
    )
    lines = ["fn main() {"]
    for type_name, member_name, representation in probes:
        key = f"{type_name}::{member_name}"
        lines.append(
            f"    let value: {representation} = unsafe {{ core::mem::transmute({type_name}::{member_name}) }};"
        )
        lines.append(f"    println!({json.dumps(key + '={}')}, value as i128);")
    lines.append("}")
    (source / "main.rs").write_text("\n".join(lines) + "\n", encoding="utf-8")
    completed = subprocess.run(
        ["cargo", "run", "--quiet", "--manifest-path", str(project / "Cargo.toml")],
        cwd=ROOT, text=True, capture_output=True,
    )
    if completed.returncode != 0:
        message = completed.stderr.strip() or completed.stdout.strip() or "flag probe failed"
        for type_name, _, _ in probes:
            actual[type_name]["flagProbeError"] = message
        return
    values = dict(line.rsplit("=", 1) for line in completed.stdout.splitlines() if "=" in line)
    for type_name, member_name, _ in probes:
        key = f"{type_name}::{member_name}"
        if key in values:
            actual[type_name]["members"][member_name]["value"] = values[key]


def operator_traits(contract_type: dict, rules: dict, reference_types: dict[str, dict]) -> set[str]:
    names = {
        "op_Addition": "Add", "op_Subtraction": "Sub", "op_Multiply": "Mul",
        "op_Division": "Div", "op_UnaryNegation": "Neg",
        "op_Equality": "PartialEq", "op_Inequality": "PartialEq",
    }
    result = set()
    for member in contract_type["members"]:
        trait = names.get(member.get("name"))
        if not trait:
            continue
        if trait in {"Neg", "PartialEq"}:
            result.add(trait)
            continue
        parameters = member.get("parameters", [])
        rhs = "Self"
        if len(parameters) > 1:
            rhs = clr_value_type(parameters[1]["type"], rules, contract_type, reference_types, [])
        result.add(f"{trait}<{rhs}>")
    return result


def interface_projection(value: str, expected: dict[str, dict], rules: dict) -> str | None:
    base, arguments = split_generic_arguments(value)
    if base == "System.IDisposable" or base in rules.get("ignoredClrInterfaces", []):
        return None
    system = rules.get("systemInterfaceTraits", {}).get(base)
    if system:
        if arguments:
            system = system.replace("{T}", mapped_type_path(arguments[0], rules))
        return system
    if base.startswith("Microsoft.Xna.Framework."):
        path = "cna::" + rules["genericTypeRenames"].get(base, base).replace(".", "::").replace("`1", "").replace("`2", "")
        if path not in expected:
            return None
        return mapped_type_path(value, rules)
    return "UNMAPPED:" + value


def finding(code: str, type_name: str, member: str | None = None, **values: object) -> dict:
    subject = type_name if member is None else f"{type_name}::{member}"
    result = {"code": code, "subject": subject, "type": type_name}
    if member is not None:
        result["member"] = member
    result.update(values)
    categories = set(result.pop("categories", []))
    categories.add(code)
    result["categories"] = sorted(categories)
    return result


def mapping_categories(member: dict) -> set[str]:
    result = set()
    if member.get("origin") == "constructor":
        result.add("CONSTRUCTOR_MAPPING_MISMATCH")
    if member.get("origin", "").startswith("property"):
        result.add("PROPERTY_MAPPING_MISMATCH")
    if member.get("origin") == "event":
        result.add("EVENT_MAPPING_MISMATCH")
    if member.get("origin") == "delegate":
        result.add("DELEGATE_MAPPING_MISMATCH")
    if member.get("overload", 0) > 0:
        result.add("OVERLOAD_MAPPING_MISMATCH")
    return result


def compare_generics(
    findings: list[dict], type_name: str, member_name: str | None,
    wanted: list[dict], present: list[dict], categories: set[str],
) -> None:
    wanted_names = [value["name"] for value in wanted]
    present_names = [value["name"] for value in present]
    if wanted_names != present_names:
        findings.append(finding(
            "GENERIC_MISMATCH", type_name, member_name,
            expected=wanted_names, actual=present_names, categories=categories,
        ))
        return
    wanted_bounds = {value["name"]: value.get("bounds", []) for value in wanted}
    present_bounds = {value["name"]: value.get("bounds", []) for value in present}
    if wanted_bounds != present_bounds:
        findings.append(finding(
            "GENERIC_BOUND_MISMATCH", type_name, member_name,
            expected=wanted_bounds, actual=present_bounds, categories=categories,
        ))


def compare_member_signature(
    findings: list[dict], type_name: str, wanted: dict, present: dict,
) -> None:
    categories = mapping_categories(wanted)
    if wanted["kind"] != "unknown" and wanted["kind"] != present["kind"]:
        primary = next(iter(sorted(categories)), "PARAMETER_MISMATCH")
        findings.append(finding(
            primary, type_name, wanted["name"], expected=wanted["kind"], actual=present["kind"],
            categories=categories,
        ))
        return
    if wanted["kind"] == "function":
        compare_generics(
            findings, type_name, wanted["name"], wanted.get("generics", []),
            present.get("generics", []), categories,
        )
        expected_parameters = wanted.get("parameters", [])
        actual_parameters = present.get("parameters", [])
        ordinary, ref_out = [], []
        maximum = max(len(expected_parameters), len(actual_parameters))
        for position in range(maximum):
            expected_value = expected_parameters[position] if position < len(expected_parameters) else None
            actual_value = actual_parameters[position] if position < len(actual_parameters) else None
            if expected_value != actual_value:
                target = ref_out if position in wanted.get("refOut", []) else ordinary
                target.append({"position": position, "expected": expected_value, "actual": actual_value})
        if ordinary:
            findings.append(finding(
                "PARAMETER_MISMATCH", type_name, wanted["name"], differences=ordinary,
                categories=categories,
            ))
        if ref_out:
            findings.append(finding(
                "REF_OUT_MAPPING_MISMATCH", type_name, wanted["name"], differences=ref_out,
                categories=categories,
            ))
        if wanted.get("returnType", "()") != present.get("returnType", "()"):
            findings.append(finding(
                "RETURN_TYPE_MISMATCH", type_name, wanted["name"],
                expected=wanted.get("returnType", "()"), actual=present.get("returnType", "()"),
                categories=categories,
            ))
    elif wanted["kind"] in {"field", "assoc_const"}:
        if wanted.get("type") != present.get("type"):
            findings.append(finding(
                "RETURN_TYPE_MISMATCH", type_name, wanted["name"],
                expected=wanted.get("type"), actual=present.get("type"), categories=categories,
            ))
        if wanted.get("origin") == "enum-value" and wanted.get("value") != present.get("value"):
            findings.append(finding(
                "ENUM_VALUE_MISMATCH", type_name, wanted["name"],
                expected=wanted.get("value"), actual=present.get("value"),
                categories=categories | {"FLAGS_MAPPING_MISMATCH"},
            ))
    elif wanted["kind"] == "variant" and wanted.get("value") != present.get("value"):
        findings.append(finding(
            "ENUM_VALUE_MISMATCH", type_name, wanted["name"],
            expected=wanted.get("value"), actual=present.get("value"), categories=categories,
        ))


def compare_relations(
    findings: list[dict], type_name: str, wanted: dict, present: dict,
    expected: dict[str, dict], rules: dict,
) -> None:
    compare_generics(
        findings, type_name, None, wanted.get("genericParameters", []), present.get("generics", []), set()
    )
    traits = present["traits"]
    if wanted["clrKind"] == "struct":
        for required in {"Copy", "Clone", "PartialEq"}:
            if (
                required == "Copy"
                and wanted.get("clrName") in rules.get("nonCopyStructProjections", [])
            ):
                continue
            if required not in traits:
                findings.append(finding("TRAIT_MISMATCH", type_name, expected=required, actual=sorted(traits)))
    for required in wanted.get("operatorTraits", set()):
        if required not in traits:
            findings.append(finding("TRAIT_MISMATCH", type_name, expected=required, actual=sorted(traits)))

    base = wanted.get("baseType")
    if base and base.startswith("Microsoft.Xna.Framework."):
        required = rules.get("baseProjectionOverrides", {}).get(wanted.get("clrName", ""))
        if required is None:
            base_path = "cna::" + rules["genericTypeRenames"].get(base, base).replace(".", "::").replace("`1", "").replace("`2", "")
            required = relative_rust_path(base_path)
        if required not in traits:
            findings.append(finding(
                "BASE_PROJECTION_MISMATCH", type_name, expected=required, actual=sorted(traits)
            ))

    for interface in wanted.get("interfaces", []):
        interface_base, _ = split_generic_arguments(interface)
        required = rules.get("interfaceProjectionOverrides", {}).get(
            wanted.get("clrName", "") + "::" + interface_base
        )
        if required is None:
            required = interface_projection(interface, expected, rules)
        if required is not None and required not in traits:
            findings.append(finding(
                "INTERFACE_MISMATCH", type_name, expected=required, actual=sorted(traits)
            ))

    disposable = any(value == "System.IDisposable" for value in wanted.get("allInterfaces", []))
    if disposable:
        available = set(present["members"]) | set(present["traitMembers"])
        if "Graphics::GraphicsResource" in traits:
            available.add("Dispose")
        reasons = []
        if "Dispose" not in available:
            reasons.append("Dispose contract")
        # Value-type enumerators often implement IDisposable as a no-op. They
        # have no owned resource to release and Rust cannot combine Copy with
        # Drop, so explicit Dispose is the complete projection for CLR structs.
        if wanted["clrKind"] != "struct" and present["kind"] != "trait" and not present["drop"]:
            reasons.append("Drop")
        if reasons:
            findings.append(finding(
                "DISPOSAL_MAPPING_MISMATCH", type_name,
                expected=["Dispose contract", "Drop"], actual=reasons,
            ))

    if wanted.get("flags") and "transparent" not in present.get("repr", []):
        findings.append(finding(
            "FLAGS_MAPPING_MISMATCH", type_name, expected="repr(transparent) newtype", actual=present.get("repr", [])
        ))
    if wanted.get("flags") and present.get("flagProbeError"):
        findings.append(finding(
            "FLAGS_MAPPING_MISMATCH", type_name, expected="readable flag values",
            actual=present["flagProbeError"],
        ))


def compare(expected: dict[str, dict], actual: dict[str, dict], rules: dict) -> list[dict]:
    findings: list[dict] = []
    for name in sorted(expected.keys() - actual.keys()):
        findings.append(finding("MISSING_TYPE", name))
    for name in sorted(actual.keys() - expected.keys()):
        findings.append(finding("UNEXPECTED_TYPE", name))
    for name in sorted(expected.keys() & actual.keys()):
        wanted, present = expected[name], actual[name]
        if wanted["kind"] != present["kind"]:
            findings.append(finding(
                "TYPE_KIND_MAPPING_MISMATCH", name, expected=wanted["kind"], actual=present["kind"]
            ))
        wanted_names, present_names = set(wanted["members"]), set(present["members"])
        for member_name in sorted(wanted_names - present_names):
            descriptor = wanted["members"][member_name]
            findings.append(finding(
                "MISSING_MEMBER", name, member_name, origin=descriptor.get("origin"),
                categories=mapping_categories(descriptor),
            ))
        for member_name in sorted(present_names - wanted_names):
            findings.append(finding("UNEXPECTED_MEMBER", name, member_name))
        for member_name in sorted(wanted_names & present_names):
            compare_member_signature(
                findings, name, wanted["members"][member_name], present["members"][member_name]
            )
        compare_relations(findings, name, wanted, present, expected, rules)
        if wanted.get("underlyingType") and present["kind"] == "enum" and wanted["underlyingType"] not in present.get("repr", []):
            findings.append(finding(
                "ENUM_VALUE_MISMATCH", name, expected=f"repr({wanted['underlyingType']})", actual=present.get("repr", [])
            ))
        if present["internalLeak"]:
            findings.append(finding("INTERNAL_TYPE_LEAK", name))
        if present["rawHandleLeak"]:
            findings.append(finding("RAW_HANDLE_LEAK", name))
        for member in present["unsafeMembers"]:
            findings.append(finding("UNSAFE_PUBLIC_API", name, member))
    return findings


def scoreboard(expected: dict[str, dict], findings: list[dict]) -> list[dict]:
    by_type = collections.defaultdict(list)
    for value in findings:
        by_type[value["type"]].append(value)
    signature_codes = {
        "PARAMETER_MISMATCH", "RETURN_TYPE_MISMATCH", "GENERIC_MISMATCH",
        "GENERIC_BOUND_MISMATCH", "REF_OUT_MAPPING_MISMATCH",
    }
    result = []
    for name in sorted(expected):
        local = by_type[name]
        result.append({
            "type": name,
            "missingType": any(value["code"] == "MISSING_TYPE" for value in local),
            "missingMembers": sum(value["code"] == "MISSING_MEMBER" for value in local),
            "missingConstructors": sum(
                value["code"] == "MISSING_MEMBER" and "CONSTRUCTOR_MAPPING_MISMATCH" in value["categories"]
                for value in local
            ),
            "missingProperties": sum(
                value["code"] == "MISSING_MEMBER" and "PROPERTY_MAPPING_MISMATCH" in value["categories"]
                for value in local
            ),
            "missingOverloads": sum(
                value["code"] == "MISSING_MEMBER" and "OVERLOAD_MAPPING_MISMATCH" in value["categories"]
                for value in local
            ),
            "missingTraitsInterfaces": sum(
                value["code"] in {"BASE_PROJECTION_MISMATCH", "TRAIT_MISMATCH", "INTERFACE_MISMATCH"}
                for value in local
            ),
            "signatureMismatches": sum(value["code"] in signature_codes for value in local),
            "enumValueMismatches": sum(value["code"] in {"ENUM_VALUE_MISMATCH", "FLAGS_MAPPING_MISMATCH"} for value in local),
            "eventMappings": sum("EVENT_MAPPING_MISMATCH" in value["categories"] for value in local),
            "disposalMappings": sum(value["code"] == "DISPOSAL_MAPPING_MISMATCH" for value in local),
            "totalDiagnostics": len(local),
        })
    return result


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
                finding("INTERNAL_TYPE_LEAK", name)
                for name, value in actual.items() if value["internalLeak"]
            ]
            findings.extend(
                finding("RAW_HANDLE_LEAK", name)
                for name, value in actual.items() if value["rawHandleLeak"]
            )
            findings.extend(
                finding("UNSAFE_PUBLIC_API", name, member)
                for name, value in actual.items() for member in value["unsafeMembers"]
            )
        else:
            if not args.reference_dir:
                raise ValueError("XNA_REFERENCE_PATH/--reference-dir is required")
            reference_dir = Path(args.reference_dir)
            validate_references(reference_dir, profile)
            reference = extract_reference(reference_dir, profile, temporary)
            expected = expected_contract(reference, rules)
            probe_flag_values(expected, actual, temporary)
            findings = compare(expected, actual, rules)
        counts = collections.Counter(value["code"] for value in findings)
        category_counts = collections.Counter(
            category for value in findings for category in value.get("categories", [value["code"]])
        )
        measured_categories = [
            "BASE_PROJECTION_MISMATCH", "TRAIT_MISMATCH", "INTERFACE_MISMATCH",
            "PARAMETER_MISMATCH", "RETURN_TYPE_MISMATCH", "GENERIC_MISMATCH",
            "GENERIC_BOUND_MISMATCH", "REF_OUT_MAPPING_MISMATCH",
            "ENUM_VALUE_MISMATCH", "FLAGS_MAPPING_MISMATCH",
            "DELEGATE_MAPPING_MISMATCH", "EVENT_MAPPING_MISMATCH",
            "DISPOSAL_MAPPING_MISMATCH", "CONSTRUCTOR_MAPPING_MISMATCH",
            "OVERLOAD_MAPPING_MISMATCH", "PROPERTY_MAPPING_MISMATCH",
        ]
        report = {
            "schemaVersion": 2,
            "profile": profile["name"],
            "referenceTypes": None if args.leak_only else len(reference["types"]),
            "referenceMembers": None if args.leak_only else sum(len(t["members"]) for t in reference["types"]),
            "expectedRustTypes": len(expected),
            "actualRustTypes": len(actual),
            "totalDiagnostics": len(findings),
            "counts": dict(sorted(counts.items())),
            "categoryCounts": {
                category: category_counts.get(category, 0) for category in measured_categories
            },
            "allowlistEntries": len(rules["allowlist"]),
            "measuredCategories": measured_categories,
            "unmeasuredCategories": [],
            "typeScoreboard": [] if args.leak_only else scoreboard(expected, findings),
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
