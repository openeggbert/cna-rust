#!/usr/bin/env python3
"""Builds and runs an out-of-tree consumer that links CNA at build time.

Dynamic mode is the default and is exercised by the whole test suite. Direct
mode is not, because it changes what the linker must find, so it needs a
consumer of its own -- one built the way a real project would build it, outside
the workspace, with nothing but the crate and the library.

The check is deliberately not "it compiled". A directly linked build must
resolve CNA when the consumer is linked, must call a real runtime route, and
must not reach the dynamic loader at all: an executable that quietly kept
`dlopen` would pass a compile check and fail on the platform this mode exists
for.
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

CONSUMER_MANIFEST = """[workspace]

[package]
name = "cna-direct-link-consumer"
version = "0.0.0"
edition = "2021"

[dependencies]
cna = {{ package = "cna-rust", path = "{crate}", default-features = false, features = ["direct-link"] }}
"""

CONSUMER_MAIN = '''//! Proves a directly linked build reaches real CNA routes.
use cna::extensions::runtime::{current_renderer, platform, platform_name};
use cna::Microsoft::Xna::Framework::Graphics::{
    GraphicsDevice, GraphicsProfile, PresentationParameters,
};
use cna::Microsoft::Xna::Framework::{GraphicsDeviceInformation, Vector3};

fn main() {
    // A managed value, which needs no library at all.
    assert_eq!(
        Vector3::Forward.Z,
        -1.0,
        "the strict XNA projection is unchanged by linkage mode"
    );

    // Process-global native routes, reached through link-resolved symbols.
    let renderer = current_renderer().expect("the linked CNA answers its renderer");
    let host = platform().expect("the linked CNA answers its platform");
    let name = platform_name().expect("the linked CNA answers its platform name");

    // A real object with a real lifecycle, not just an identity query: this
    // creates a device, uses it and releases it, entirely over linked symbols.
    let parameters = PresentationParameters::new();
    parameters.SetBackBufferWidth(32);
    parameters.SetBackBufferHeight(32);
    let mut device = GraphicsDevice::new(
        &GraphicsDeviceInformation::new().Adapter(),
        GraphicsProfile::Reach,
        &parameters,
    )
    .expect("a directly linked build constructs a GraphicsDevice");
    assert_eq!(
        device
            .PresentationParameters()
            .expect("presentation parameters")
            .BackBufferWidth(),
        32
    );
    device
        .DisposeWithNoArguments()
        .expect("a directly linked build disposes it again");

    println!(
        "cna-direct-link-consumer: renderer={renderer:?} platform={host:?} name={name} device=ok"
    );
}
'''


def arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--build-dir", type=Path, default=DEFAULT_BUILD)
    parser.add_argument("--jobs", default="4", help="Cargo parallelism for the consumer build")
    parser.add_argument("--library", default=os.environ.get("CNA_NATIVE_LIBRARY"))
    return parser.parse_args()


def dynamic_dependencies(binary: Path) -> list[str]:
    """The shared objects the executable itself records a need for."""
    completed = subprocess.run(
        ["readelf", "--dynamic", str(binary)],
        check=True, text=True, capture_output=True,
    )
    needed = []
    for line in completed.stdout.splitlines():
        if "(NEEDED)" in line and "[" in line:
            needed.append(line.split("[", 1)[1].split("]", 1)[0])
    return needed


def main() -> int:
    args = arguments()
    if not args.library:
        raise ValueError("CNA_NATIVE_LIBRARY/--library is required")
    library = Path(args.library).resolve()
    if not library.is_file():
        raise ValueError(f"no CNA library at {library}")

    build = args.build_dir
    consumer = build / "direct-link-consumer"
    if consumer.exists():
        shutil.rmtree(consumer)
    (consumer / "src").mkdir(parents=True)
    (consumer / "Cargo.toml").write_text(
        CONSUMER_MANIFEST.format(crate=(ROOT / "crates/cna").as_posix()), encoding="utf-8"
    )
    (consumer / "src/main.rs").write_text(CONSUMER_MAIN, encoding="utf-8")

    environment = dict(os.environ)
    environment["CNA_NATIVE_LIBRARY"] = str(library)
    environment["CARGO_TARGET_DIR"] = str(build / "direct-link-target")

    subprocess.run(
        ["cargo", "build", "--jobs", str(args.jobs), "--manifest-path",
         str(consumer / "Cargo.toml")],
        check=True, env=environment,
    )

    binary = Path(environment["CARGO_TARGET_DIR"]) / "debug/cna-direct-link-consumer"
    needed = dynamic_dependencies(binary)
    linked_cna = [name for name in needed if "cna_c_api" in name]
    if not linked_cna:
        raise ValueError(
            f"the consumer does not record a link-time need for CNA; NEEDED={needed}"
        )

    # `-ldl` is still listed by the C runtime on glibc, so the meaningful check
    # is that this crate no longer *calls* the loader, not that libdl is absent.
    completed = subprocess.run(
        ["nm", "--undefined-only", "--dynamic", str(binary)],
        check=True, text=True, capture_output=True,
    )
    undefined = {line.split()[-1] for line in completed.stdout.splitlines() if line.strip()}
    loader_calls = sorted(undefined & {"dlopen", "dlsym", "dlclose"})

    environment["LD_LIBRARY_PATH"] = os.pathsep.join(
        [str(library.parent), environment.get("LD_LIBRARY_PATH", "")]
    ).strip(os.pathsep)
    run = subprocess.run([str(binary)], check=True, text=True, capture_output=True,
                         env=environment)

    print(f"DIRECT_LINK_CONSUMER_BUILD=PASS")
    print(f"DIRECT_LINK_NEEDED={','.join(linked_cna)}")
    print(f"DIRECT_LINK_LOADER_CALLS={','.join(loader_calls) if loader_calls else 'none'}")
    print(f"DIRECT_LINK_RUN={run.stdout.strip().splitlines()[-1] if run.stdout.strip() else 'no output'}")
    if loader_calls:
        raise ValueError(
            f"a directly linked consumer still imports the dynamic loader: {loader_calls}"
        )
    print("DIRECT_LINK_CONSUMER=PASS")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (ValueError, OSError, subprocess.CalledProcessError) as error:
        print(f"direct-link consumer: {error}", file=sys.stderr)
        raise SystemExit(2)
