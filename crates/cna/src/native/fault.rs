//! Deliberate test-only failures around reviewed native calls.

use cna_sys as sys;

use crate::error::{CnaError, ErrorCategory};

pub(crate) fn check(point: &'static str) -> Result<(), CnaError> {
    if std::env::var("CNA_RUST_TEST_FAULT").as_deref() == Ok(point) {
        Err(CnaError::Native {
            code: sys::CNA_RESULT_INTERNAL,
            category: ErrorCategory::None,
            message: format!("injected Rust bridge fault at {point}"),
        })
    } else {
        Ok(())
    }
}
