//! Link configuration for the `direct-link` feature.
//!
//! Dynamic mode needs nothing from this file: it resolves CNA at run time and
//! deliberately carries no link-time dependency, so a consumer can build with
//! no library present and point at one later.
//!
//! `direct-link` is the opposite by design. CNA's symbols must resolve when the
//! consumer is linked, and this is what tells the linker where to find them.
//! The same environment variables the run-time loader honours are used here, so
//! a project does not learn a second way to say where CNA is.

use std::env;
use std::path::{Path, PathBuf};

/// The base name the linker asks for, without the platform's prefix or suffix.
const LIBRARY: &str = "cna_c_api";

fn main() {
    println!("cargo:rerun-if-env-changed=CNA_NATIVE_LIBRARY");
    println!("cargo:rerun-if-env-changed=CNA_NATIVE_DIR");
    println!("cargo:rerun-if-env-changed=CNA_ROOT");
    println!("cargo:rerun-if-changed=build.rs");

    if env::var_os("CARGO_FEATURE_DIRECT_LINK").is_none() {
        return;
    }

    match search_directory() {
        Some(directory) => {
            println!("cargo:rustc-link-search=native={}", directory.display());
            println!("cargo:rustc-link-lib=dylib={LIBRARY}");
        }
        None => {
            // Failing here, with the same names the loader documents, is much
            // better than a page of undefined-reference lines naming symbols
            // the reader has never heard of.
            panic!(
                "the direct-link feature needs CNA at link time, but no library directory was \
                 found. Set CNA_NATIVE_LIBRARY to the library file, CNA_NATIVE_DIR to the \
                 directory holding it, or CNA_ROOT to a CNA build or install root."
            );
        }
    }
}

/// The directory holding the CNA library, from the loader's own variables.
fn search_directory() -> Option<PathBuf> {
    if let Some(value) = env::var_os("CNA_NATIVE_LIBRARY") {
        let path = PathBuf::from(value);
        if let Some(parent) = path.parent() {
            if parent.as_os_str().is_empty() {
                return Some(PathBuf::from("."));
            }
            return Some(parent.to_path_buf());
        }
    }
    if let Some(value) = env::var_os("CNA_NATIVE_DIR") {
        return Some(PathBuf::from(value));
    }
    if let Some(value) = env::var_os("CNA_ROOT") {
        let root = PathBuf::from(value);
        // The layouts a CNA build or install actually produces, in the order
        // the run-time loader searches them.
        for relative in [
            "modules/c-api",
            "build/modules/c-api",
            "lib",
            "bin",
            "",
        ] {
            let candidate = root.join(relative);
            if contains_library(&candidate) {
                return Some(candidate);
            }
        }
    }
    None
}

fn contains_library(directory: &Path) -> bool {
    ["so", "dylib", "dll", "a"]
        .iter()
        .any(|extension| directory.join(format!("lib{LIBRARY}.{extension}")).is_file())
        || directory.join(format!("{LIBRARY}.dll")).is_file()
}
