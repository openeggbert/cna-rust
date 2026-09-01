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

It also answers the question `RUST-SURFACE-001` created. CNA's own members no
longer sit on strict XNA types; they are extension-trait methods, so the call
that used to compile against a bare `use cna::...::Song` must now fail and the
same call with `use cna::extensions::media::SongExt` must succeed. Both are
built here, from the packaged sources, outside the workspace -- the positive one
because a trait a consumer cannot import is not published, and the negative one
because a gate that only ever compiles proves nothing.
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



# The call shapes a consumer writes against the migrated surface. Nothing here
# runs a native route: the point is that the names resolve.
EXTENSION_CALLS = """\
#[allow(dead_code, unused_variables)]
fn extension_call_shapes(
    game: &GameContext<'_>,
    device: &GraphicsDevice,
    manager: &GraphicsDeviceManager,
    song: &Song,
) {
    // An associated function on a trait, resolved through the strict type.
    let _ = Song::FromFile(game, "theme", "theme.ogg");
    // Ordinary receiver methods.
    let _ = song.HandleText();
    let _ = device.set_string_marker("frame");
    let _ = manager.HasNativeGraphicsDevice();
}
"""

CONSUMER_PREAMBLE = """\
//! Compiles the packaged binding source and touches both halves of it.
use cna::extensions::graphics::NativeEnumValue;
use cna::Microsoft::Xna::Framework::Graphics::{Blend, GraphicsDevice};
use cna::Microsoft::Xna::Framework::Media::Song;
use cna::Microsoft::Xna::Framework::{GameContext, GraphicsDeviceManager, TimeSpan, Vector3};
"""

CONSUMER_IMPORTS = """\
use cna::extensions::graphics_device_ext::{DeviceStateExt, GraphicsDeviceManagerExt};
use cna::extensions::media::SongExt;
"""

CONSUMER_BODY = """\
fn main() {
    // Strict XNA: exact managed value behaviour, no native library needed.
    assert_eq!(Vector3::Forward.Z, -1.0);
    assert_eq!(TimeSpan::FromSeconds(1.5).Ticks(), 15_000_000);
    // A CNA extension that is pure arithmetic, so it runs without a library.
    assert_eq!(Blend::from_native_value(1), Some(Blend::Zero));
    assert_eq!(Blend::from_native_value(9_999), None);
    println!("cna-packaged-source-consumer: packaged sources build and run");
}
"""

CONSUMER_MAIN = CONSUMER_PREAMBLE + CONSUMER_IMPORTS + "\n" + CONSUMER_BODY + "\n" + EXTENSION_CALLS


def extension_calls_resolve(build: Path, staged: Path, jobs: str) -> dict[str, bool]:
    """Builds the same extension calls with the traits imported, and without.

    Without them the call must not resolve. A consumer that compiles either way
    would mean the members are still inherent on the strict XNA types, which is
    the state `RUST-SURFACE-001` existed to end.
    """
    outcomes = {}
    for label, imports in (("with", CONSUMER_IMPORTS), ("without", "")):
        project = build / f"extension-{label}"
        if project.exists():
            shutil.rmtree(project)
        (project / "src").mkdir(parents=True)
        (project / "Cargo.toml").write_text(
            "[workspace]\n\n"
            "[package]\n"
            f'name = "cna-extension-{label}-import"\n'
            'version = "0.0.0"\n'
            'edition = "2021"\n\n'
            "[dependencies]\n"
            f'cna = {{ package = "cna-rust", path = "{staged / "cna"}" }}\n',
            encoding="utf-8",
        )
        (project / "src/main.rs").write_text(
            CONSUMER_PREAMBLE + imports + "\nfn main() {}\n\n" + EXTENSION_CALLS,
            encoding="utf-8",
        )
        environment = dict(os.environ)
        environment["CARGO_TARGET_DIR"] = str(build / "target")
        completed = subprocess.run(
            ["cargo", "build", "--quiet", "-j", jobs],
            cwd=project, env=environment, text=True, capture_output=True,
        )
        if label == "with":
            if completed.returncode != 0:
                print(completed.stderr, file=sys.stderr)
            outcomes["with"] = completed.returncode == 0
        else:
            # E0599 is "no method named ..."; anything else means the build
            # broke for an unrelated reason and proves nothing.
            outcomes["without"] = completed.returncode != 0 and "E0599" in completed.stderr
            if not outcomes["without"]:
                print(completed.stderr, file=sys.stderr)
    return outcomes


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
    (consumer / "src/main.rs").write_text(CONSUMER_MAIN, encoding="utf-8")

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
    resolves = extension_calls_resolve(build, staged, args.jobs)
    print(f"PACKAGE_CONSUMER_EXTENSION_WITH_IMPORT={'PASS' if resolves['with'] else 'FAIL'}")
    print(
        "PACKAGE_CONSUMER_EXTENSION_WITHOUT_IMPORT="
        + ("REFUSED" if resolves["without"] else "COMPILED")
    )
    print(f"PACKAGE_CONSUMER_SYS_FILES={sys_count}")
    print(f"PACKAGE_CONSUMER_CNA_FILES={cna_count}")
    print(f"PACKAGE_CONSUMER_WORKSPACE_PATH_LEAKS={len(leaks)}")
    for leak in leaks:
        print(f"  leak: {leak}")
    failed = bool(leaks) or not resolves["with"] or not resolves["without"]
    print("PACKAGE_CONSUMER_STATUS=" + ("FAIL" if failed else "PASS"))
    return 1 if failed else 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, subprocess.CalledProcessError) as error:
        print(f"package consumer: {error}", file=sys.stderr)
        raise SystemExit(2)
