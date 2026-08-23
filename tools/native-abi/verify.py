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
import tempfile

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


def rust_function_declarations() -> dict[str, dict]:
    """Extract reviewed function-pointer aliases from cna-sys's sole declaration source."""
    source = (ROOT / "crates/cna-sys/src/lib.rs").read_text(encoding="utf-8")
    pattern = re.compile(
        r'pub type (cna_[A-Za-z0-9_]+)_fn\s*=\s*unsafe extern "C" fn\s*'
        r'\((.*?)\)\s*(?:->\s*([^;]+))?;',
        re.S,
    )
    result = {}
    for match in pattern.finditer(source):
        parameters = match.group(2).strip()
        result[match.group(1)] = {
            "return": (match.group(3) or "()").strip(),
            "parameters": [] if not parameters else [
                value.strip() for value in split_parameters(parameters) if value.strip()
            ],
        }
    return result


def clang_function_declaration(cna_root: Path, symbol: str, source: Path) -> dict | None:
    """Ask Clang for the exact canonical-header declaration of one function."""
    completed = subprocess.run(
        [
            os.environ.get("CLANG", "clang"),
            "-std=c11",
            "-I", str(cna_root / "modules/c-api/include"),
            "-Xclang", "-ast-dump=json",
            "-Xclang", f"-ast-dump-filter={symbol}",
            "-fsyntax-only", str(source),
        ],
        check=True,
        text=True,
        capture_output=True,
    )
    decoder = json.JSONDecoder()
    text = completed.stdout
    position = 0
    while position < len(text):
        while position < len(text) and text[position].isspace():
            position += 1
        if position == len(text):
            break
        value, position = decoder.raw_decode(text, position)
        if value.get("kind") == "FunctionDecl" and value.get("name") == symbol:
            return {
                "return": value["type"]["qualType"].split(" (", 1)[0],
                "parameters": [
                    child["type"]["qualType"]
                    for child in value.get("inner", [])
                    if child.get("kind") == "ParmVarDecl"
                ],
            }
    return None


def canonical_c_type(value: str) -> dict:
    value = " ".join(value.replace("*", " * ").split())
    pointer_depth = value.count("*")
    pointee_const = pointer_depth > 0 and value.startswith("const ")
    base = value.replace("*", "").replace("const", "").strip()
    base = {
        "uint8_t": "u8", "int8_t": "i8", "uint16_t": "u16", "int16_t": "i16",
        "uint32_t": "u32", "int32_t": "i32", "uint64_t": "u64", "int64_t": "i64",
        "float": "f32", "double": "f64", "char": "c_char", "void": "c_void",
    }.get(base, base)
    return {"base": base, "pointerDepth": pointer_depth, "pointeeConst": pointee_const}


def canonical_rust_type(value: str) -> dict:
    value = " ".join(value.split())
    pointer_depth = 0
    pointee_const = False
    while value.startswith("*const ") or value.startswith("*mut "):
        is_const = value.startswith("*const ")
        if pointer_depth == 0:
            pointee_const = is_const
        pointer_depth += 1
        value = value.split(" ", 1)[1].strip()
    base = {
        "()": "void", "core::ffi::c_void": "c_void", "c_void": "c_void",
        "core::ffi::c_char": "c_char", "c_char": "c_char",
    }.get(value, value)
    return {"base": base, "pointerDepth": pointer_depth, "pointeeConst": pointee_const}


def prototype_probes(cna_root: Path, symbols: dict[str, int]) -> tuple[int, int, list[dict]]:
    """Compare every reviewed header prototype with its cna-sys function alias."""
    rust = rust_function_declarations()
    measurements = 0
    findings = []
    with tempfile.TemporaryDirectory(prefix="cna-rust-prototypes-") as name:
        source = Path(name) / "probe.c"
        source.write_text('#include "CNA/C/cna.h"\n', encoding="utf-8")
        for symbol in symbols:
            c_value = clang_function_declaration(cna_root, symbol, source)
            rust_value = rust.get(symbol)
            if c_value is None or rust_value is None:
                findings.append({
                    "code": "MISSING_PROTOTYPE_DECLARATION",
                    "symbol": symbol,
                    "c": c_value,
                    "rust": rust_value,
                })
                continue
            c_types = [c_value["return"], *c_value["parameters"]]
            rust_types = [rust_value["return"], *rust_value["parameters"]]
            measurements += max(len(c_types), len(rust_types))
            c_canonical = [canonical_c_type(value) for value in c_types]
            rust_canonical = [canonical_rust_type(value) for value in rust_types]
            if c_canonical != rust_canonical:
                findings.append({
                    "code": "C_RUST_PROTOTYPE_MISMATCH",
                    "symbol": symbol,
                    "c": c_value,
                    "rust": rust_value,
                    "cCanonical": c_canonical,
                    "rustCanonical": rust_canonical,
                })
    return len(symbols), measurements, findings


def elf_exports(library: Path) -> set[str]:
    output = subprocess.run(
        ["nm", "-D", "--defined-only", str(library)], check=True, text=True, capture_output=True
    ).stdout
    return {
        fields[-1].split("@", 1)[0]
        for line in output.splitlines()
        if (fields := line.split()) and fields[-1].split("@", 1)[0].startswith("cna_")
    }


def parse_probe_output(text: str) -> dict[str, int]:
    return {
        key: int(value)
        for line in text.splitlines()
        if "=" in line
        for key, value in [line.rsplit("=", 1)]
    }


def abi_probes(cna_root: Path, manifest: dict) -> tuple[dict[str, int], dict[str, int]]:
    """Compile independent C and Rust probes and return their measured facts."""
    with tempfile.TemporaryDirectory(prefix="cna-rust-abi-") as name:
        temporary = Path(name)
        c_lines = [
            "#include <stddef.h>",
            "#include <stdint.h>",
            "#include <stdio.h>",
            '#include "CNA/C/cna.h"',
        ]
        for callback, value in manifest.get("callbackSignatures", {}).items():
            expected = f"Expected_{callback}"
            c_lines.append(
                f"typedef {value['cReturn']} (*{expected})({', '.join(value['cParameters'])});"
            )
            c_lines.append(
                f'_Static_assert(__builtin_types_compatible_p({callback}, {expected}), "{callback}");'
            )
        c_lines.append("int main(void) {")
        for type_name, fields in manifest.get("layouts", {}).items():
            c_lines.append(f'  printf("layout.{type_name}.size=%zu\\n", sizeof({type_name}));')
            c_lines.append(f'  printf("layout.{type_name}.align=%zu\\n", _Alignof({type_name}));')
            for field in fields:
                c_lines.append(
                    f'  printf("layout.{type_name}.{field}=%zu\\n", offsetof({type_name}, {field}));'
                )
        for type_name in manifest.get("scalarTypes", []):
            c_lines.append(f'  printf("scalar.{type_name}.size=%zu\\n", sizeof({type_name}));')
            c_lines.append(f'  printf("scalar.{type_name}.align=%zu\\n", _Alignof({type_name}));')
        for constant in manifest.get("constants", []):
            c_lines.append(
                f'  printf("constant.{constant}=%llu\\n", (unsigned long long)({constant}));'
            )
        for callback in manifest.get("callbackSignatures", {}):
            c_lines.append(f'  printf("callback.{callback}=1\\n");')
        c_lines.extend(["  return 0;", "}"])
        c_source = temporary / "probe.c"
        c_source.write_text("\n".join(c_lines) + "\n", encoding="utf-8")
        c_binary = temporary / "c-probe"
        subprocess.run(
            [
                os.environ.get("CC", "cc"),
                "-std=c11",
                "-I",
                str(cna_root / "modules/c-api/include"),
                str(c_source),
                "-o",
                str(c_binary),
            ],
            check=True,
            text=True,
            capture_output=True,
        )
        c_values = parse_probe_output(
            subprocess.run([str(c_binary)], check=True, text=True, capture_output=True).stdout
        )

        rust_project = temporary / "rust-probe"
        (rust_project / "src").mkdir(parents=True)
        (rust_project / "Cargo.toml").write_text(
            "[package]\nname = \"cna-rust-abi-probe\"\nversion = \"0.0.0\"\nedition = \"2021\"\n"
            f"[dependencies]\ncna_sys = {{ package = \"cna-rust-sys\", path = {json.dumps(str(ROOT / 'crates/cna-sys'))} }}\n",
            encoding="utf-8",
        )
        rust_lines = [
            "use cna_sys::*;",
            "use core::mem::{align_of, size_of, MaybeUninit};",
            "macro_rules! offset_of { ($ty:ty, $field:ident) => {{",
            "  let value = MaybeUninit::<$ty>::uninit();",
            "  let base = value.as_ptr();",
            "  unsafe { (core::ptr::addr_of!((*base).$field) as usize) - (base as usize) }",
            "}}; }",
        ]
        for index, (callback, value) in enumerate(manifest.get("callbackSignatures", {}).items()):
            parameters = ", ".join(
                f"_p{position}: {parameter}"
                for position, parameter in enumerate(value["rustParameters"])
            )
            rust_lines.append(
                f"unsafe extern \"C\" fn callback_{index}({parameters}) -> {value['rustReturn']} {{ CNA_RESULT_SUCCESS }}"
            )
        rust_lines.append("fn main() {")
        for type_name, fields in manifest.get("layouts", {}).items():
            rust_lines.append(
                f'  println!("layout.{type_name}.size={{}}", size_of::<{type_name}>());'
            )
            rust_lines.append(
                f'  println!("layout.{type_name}.align={{}}", align_of::<{type_name}>());'
            )
            for field in fields:
                rust_lines.append(
                    f'  println!("layout.{type_name}.{field}={{}}", offset_of!({type_name}, {field}));'
                )
        for type_name in manifest.get("scalarTypes", []):
            rust_lines.append(
                f'  println!("scalar.{type_name}.size={{}}", size_of::<{type_name}>());'
            )
            rust_lines.append(
                f'  println!("scalar.{type_name}.align={{}}", align_of::<{type_name}>());'
            )
        for constant in manifest.get("constants", []):
            rust_lines.append(f'  println!("constant.{constant}={{}}", {constant} as u128);')
        for index, callback in enumerate(manifest.get("callbackSignatures", {})):
            rust_lines.append(f"  let _: {callback} = Some(callback_{index});")
            rust_lines.append(f'  println!("callback.{callback}=1");')
        rust_lines.append("}")
        (rust_project / "src/main.rs").write_text(
            "\n".join(rust_lines) + "\n", encoding="utf-8"
        )
        completed = subprocess.run(
            ["cargo", "run", "--quiet", "--manifest-path", str(rust_project / "Cargo.toml")],
            cwd=ROOT,
            check=True,
            text=True,
            capture_output=True,
        )
        return c_values, parse_probe_output(completed.stdout)


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

    c_probe, rust_probe = abi_probes(Path(args.cna_root), manifest)
    for key in sorted(c_probe.keys() | rust_probe.keys()):
        if key not in c_probe or key not in rust_probe or c_probe[key] != rust_probe[key]:
            findings.append({
                "code": "C_RUST_ABI_PROBE_MISMATCH",
                "subject": key,
                "c": c_probe.get(key),
                "rust": rust_probe.get(key),
            })

    prototype_functions, prototype_measurements, prototype_findings = prototype_probes(
        Path(args.cna_root), expected
    )
    findings.extend(prototype_findings)

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
        "schemaVersion": 2,
        "cnaSysDeclarationCount": len(expected),
        "headerExportCount": len(headers),
        "nativeLibraryExportCount": None if exports is None else len(exports),
        "expectedAbiVersion": manifest["abiVersion"],
        "nativeAbiVersion": actual_version,
        "missingHeaderSymbols": sum(x["code"] == "MISSING_HEADER_SYMBOL" for x in findings),
        "arityMismatches": sum(x["code"] == "HEADER_ARITY_MISMATCH" for x in findings),
        "cRustProbeMeasurements": len(c_probe.keys() | rust_probe.keys()),
        "cRustProbeMismatches": sum(x["code"] == "C_RUST_ABI_PROBE_MISMATCH" for x in findings),
        "prototypeFunctionsChecked": prototype_functions,
        "prototypeTypeMeasurements": prototype_measurements,
        "prototypeMismatches": sum(
            x["code"] in {"MISSING_PROTOTYPE_DECLARATION", "C_RUST_PROTOTYPE_MISMATCH"}
            for x in findings
        ),
        "layoutTypesProbed": len(manifest.get("layouts", {})),
        "callbackSignaturesProbed": len(manifest.get("callbackSignatures", {})),
        "constantsProbed": len(manifest.get("constants", [])),
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
