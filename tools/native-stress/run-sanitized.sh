#!/usr/bin/env bash
set -euo pipefail

: "${CNA_NATIVE_LIBRARY:?set CNA_NATIVE_LIBRARY to an ABI-0.7 CNA library built with ASan/UBSan}"

export ASAN_OPTIONS="${ASAN_OPTIONS:-detect_leaks=1:halt_on_error=1:strict_string_checks=1}"
export UBSAN_OPTIONS="${UBSAN_OPTIONS:-halt_on_error=1:print_stacktrace=1}"

cargo test --workspace --all-features --test native_stress -- --nocapture
