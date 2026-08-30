#!/usr/bin/env python3
"""Builds an external consumer from exactly the files the crates would ship.

`cargo package -p cna-rust` cannot run before `cna-rust-sys` is published, so
the ordinary packaging check cannot answer the question that actually matters:
is the packaged file set self-sufficient? This tool answers it directly. It
asks Cargo which files each crate would ship, stages **only those**, rewrites
the one path dependency to point at the staged sibling, and builds a consumer
against the result from outside the workspace.

A file the crate needs but does not package fails the build here rather than on
a user's machine.
"""

from __future__ import annotations

import argparse
import os
from pathlib import Path
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
# The shared consumer build directory; see the openeggbert build rules.
DEFAULT_BUILD = ROOT / "build-consumer"

# Cargo generates these into the archive; they are not crate sources and a
# staged tree must not carry the workspace's own resolved lock file.
GENERATED = {"Cargo.toml.orig", ".cargo_vcs_info.json", "Cargo.lock"}


def arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--build-dir", type=Path, default=DEFAULT_BUILD)
    parser.add_argument("--jobs", default="4", help="Cargo parallelism for the consumer build")
    return parser.parse_args()


def packaged_files(package: str) -> list[str]:
    completed = subprocess.run(
        ["cargo", "package", "--list", "--allow-dirty", "-p", package],
        cwd=ROOT, check=True, text=True, capture_output=True,
    )
    return [
        line.strip()
        for line in completed.stdout.splitlines()
        if line.strip() and line.strip() not in GENERATED
    ]


def stage(package: str, source: Path, destination: Path) -> int:
    if destination.exists():
        shutil.rmtree(destination)
    destination.mkdir(parents=True)
    files = packaged_files(package)
    for name in files:
        target = destination / name
        target.parent.mkdir(parents=True, exist_ok=True)
        origin = source / name
        if not origin.is_file():
            # Cargo rewrites a readme that lives above the package root into the
            # archive root; stage it from where the manifest actually points.
            origin = ROOT / name
        shutil.copyfile(origin, target)
    return len(files)


def main() -> int:
    args = arguments()
    build = args.build_dir
    build.mkdir(parents=True, exist_ok=True)
    staged = build / "staged"
    sys_count = stage("cna-rust-sys", ROOT / "crates/cna-sys", staged / "cna-sys")
    cna_count = stage("cna-rust", ROOT / "crates/cna", staged / "cna")

    # The staged crates leave the workspace, so the inherited workspace keys and
    # the sibling path both have to be rewritten to what a published crate
    # carries. Everything else stays exactly as packaged.
    for name, manifest in (("cna-sys", staged / "cna-sys/Cargo.toml"), ("cna", staged / "cna/Cargo.toml")):
        _ = name
        text = manifest.read_text(encoding="utf-8")
        # `rust-version` first: it ends in `version.workspace = true` too, and
        # replacing the shorter key first would rewrite it to the package version.
        text = text.replace("rust-version.workspace = true", 'rust-version = "1.74"')
        text = text.replace("version.workspace = true", 'version = "0.0.0"')
        text = text.replace("edition.workspace = true", 'edition = "2021"')
        text = text.replace("license.workspace = true", 'license = "Ms-PL"')
        text = text.replace("repository.workspace = true", 'repository = "https://github.com/openeggbert/cna-rust"')
        text = text.replace('readme = "../../README.md"', 'readme = "README.md"')
        text = text.replace("[lints]\nworkspace = true\n", "")
        text = text.replace('path = "../cna-sys"', 'path = "../cna-sys"')
        manifest.write_text(text, encoding="utf-8")

    consumer = build / "consumer"
    if consumer.exists():
        shutil.rmtree(consumer)
    (consumer / "src").mkdir(parents=True)
    (consumer / "Cargo.toml").write_text(
        # An empty [workspace] keeps the consumer out of the binding's own
        # workspace, which is the point: it must build as an outside project.
        "[workspace]\n\n"
        "[package]\n"
        'name = "cna-packaged-source-consumer"\n'
        'version = "0.0.0"\n'
        'edition = "2021"\n\n'
        "[dependencies]\n"
        'cna = { package = "cna-rust", path = "../staged/cna" }\n',
        encoding="utf-8",
    )
    (consumer / "src/main.rs").write_text(
        "//! Compiles the packaged binding source and touches both halves of it.\n"
        "use cna::extensions::runtime::RendererType;\n"
        "use cna::Microsoft::Xna::Framework::{Vector3, TimeSpan};\n\n"
        "fn main() {\n"
        "    // Strict XNA: exact managed value behaviour, no native library needed.\n"
        "    assert_eq!(Vector3::Forward.Z, -1.0);\n"
        "    assert_eq!(TimeSpan::FromSeconds(1.5).Ticks(), 15_000_000);\n"
        "    // CNA extensions: an identity value, again without a library.\n"
        "    assert_eq!(RendererType::VULKAN.value(), 8);\n"
        "    println!(\"cna-packaged-source-consumer: packaged sources build and run\");\n"
        "}\n",
        encoding="utf-8",
    )

    environment = dict(os.environ)
    environment["CARGO_TARGET_DIR"] = str(build / "target")
    subprocess.run(
        ["cargo", "run", "--quiet", "-j", args.jobs],
        cwd=consumer, check=True, env=environment,
    )

    # Nothing staged may point back into the source workspace.
    leaks = [
        str(path.relative_to(build))
        for path in staged.rglob("*")
        if path.is_file() and str(ROOT) in path.read_text(encoding="utf-8", errors="ignore")
    ]
    print(f"PACKAGE_CONSUMER_SYS_FILES={sys_count}")
    print(f"PACKAGE_CONSUMER_CNA_FILES={cna_count}")
    print(f"PACKAGE_CONSUMER_WORKSPACE_PATH_LEAKS={len(leaks)}")
    for leak in leaks:
        print(f"  leak: {leak}")
    print("PACKAGE_CONSUMER_STATUS=" + ("FAIL" if leaks else "PASS"))
    return 1 if leaks else 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, subprocess.CalledProcessError) as error:
        print(f"package consumer: {error}", file=sys.stderr)
        raise SystemExit(2)
